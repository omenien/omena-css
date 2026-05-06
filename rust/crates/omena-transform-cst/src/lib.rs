//! Transform CST contract substrate for the post-v5 omena-css track.
//!
//! This crate intentionally starts at the contract layer: transform passes are
//! only valid when they declare which semantic/cascade facts they read and what
//! cascade-safety obligation they must preserve.

use serde::Serialize;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum TransformLayer {
    SemanticReadOnly,
    SemanticAware,
    Commodity,
    Emission,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum TransformPassKind {
    TreeShakeUnusedSelectors,
    ResolveCssModulesComposes,
    HashCssModuleClassNames,
    StaticVarSubstitution,
    CompressColorLiterals,
    StripWhitespaceTrivia,
    PrintCss,
}

impl TransformPassKind {
    pub const fn id(self) -> &'static str {
        match self {
            Self::TreeShakeUnusedSelectors => "tree-shake-unused-selectors",
            Self::ResolveCssModulesComposes => "resolve-css-modules-composes",
            Self::HashCssModuleClassNames => "hash-css-module-class-names",
            Self::StaticVarSubstitution => "static-var-substitution",
            Self::CompressColorLiterals => "compress-color-literals",
            Self::StripWhitespaceTrivia => "strip-whitespace-trivia",
            Self::PrintCss => "print-css",
        }
    }

    pub const fn layer(self) -> TransformLayer {
        match self {
            Self::TreeShakeUnusedSelectors
            | Self::ResolveCssModulesComposes
            | Self::HashCssModuleClassNames
            | Self::StaticVarSubstitution => TransformLayer::SemanticAware,
            Self::CompressColorLiterals | Self::StripWhitespaceTrivia => TransformLayer::Commodity,
            Self::PrintCss => TransformLayer::Emission,
        }
    }

    pub const fn reads_semantic_graph(self) -> bool {
        matches!(
            self,
            Self::TreeShakeUnusedSelectors
                | Self::ResolveCssModulesComposes
                | Self::HashCssModuleClassNames
                | Self::StaticVarSubstitution
        )
    }

    pub const fn reads_cascade_model(self) -> bool {
        matches!(
            self,
            Self::TreeShakeUnusedSelectors | Self::StaticVarSubstitution
        )
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TransformPassContractV0 {
    pub id: &'static str,
    pub kind: TransformPassKind,
    pub layer: TransformLayer,
    pub reads_semantic_graph: bool,
    pub reads_cascade_model: bool,
    pub writes_css: bool,
    pub cascade_safe_obligation: &'static str,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TransformDagEdgeV0 {
    pub from: &'static str,
    pub to: &'static str,
    pub reason: &'static str,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TransformCstBoundarySummaryV0 {
    pub schema_version: &'static str,
    pub product: &'static str,
    pub representation: &'static str,
    pub pass_contracts: Vec<TransformPassContractV0>,
    pub dag_edges: Vec<TransformDagEdgeV0>,
    pub semantic_aware_pass_count: usize,
    pub commodity_pass_count: usize,
    pub all_passes_declare_cascade_obligation: bool,
    pub provenance_preservation_required: bool,
    pub next_surfaces: Vec<&'static str>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TransformCstArtifactV0 {
    pub schema_version: &'static str,
    pub product: &'static str,
    pub source_byte_len: usize,
    pub semantic_signature: String,
    pub pass_ids: Vec<&'static str>,
    pub provenance_preserved: bool,
}

pub fn summarize_omena_transform_cst_boundary() -> TransformCstBoundarySummaryV0 {
    let pass_contracts = default_transform_pass_contracts();
    let semantic_aware_pass_count = pass_contracts
        .iter()
        .filter(|contract| contract.layer == TransformLayer::SemanticAware)
        .count();
    let commodity_pass_count = pass_contracts
        .iter()
        .filter(|contract| contract.layer == TransformLayer::Commodity)
        .count();
    let all_passes_declare_cascade_obligation = pass_contracts
        .iter()
        .all(|contract| !contract.cascade_safe_obligation.is_empty());

    TransformCstBoundarySummaryV0 {
        schema_version: "0",
        product: "omena-transform-cst.boundary",
        representation: "post-semantic-provenance-preserving-transform-cst",
        pass_contracts,
        dag_edges: default_transform_dag_edges(),
        semantic_aware_pass_count,
        commodity_pass_count,
        all_passes_declare_cascade_obligation,
        provenance_preservation_required: true,
        next_surfaces: vec![
            "omena-transform-passes",
            "omena-transform-print",
            "salsaTransformQueries",
            "sourceMapComposition",
        ],
    }
}

pub fn build_transform_cst_artifact(
    source: &str,
    semantic_signature: impl Into<String>,
    passes: &[TransformPassKind],
) -> TransformCstArtifactV0 {
    TransformCstArtifactV0 {
        schema_version: "0",
        product: "omena-transform-cst.artifact",
        source_byte_len: source.len(),
        semantic_signature: semantic_signature.into(),
        pass_ids: passes.iter().map(|pass| pass.id()).collect(),
        provenance_preserved: true,
    }
}

fn default_transform_pass_contracts() -> Vec<TransformPassContractV0> {
    [
        TransformPassKind::TreeShakeUnusedSelectors,
        TransformPassKind::ResolveCssModulesComposes,
        TransformPassKind::HashCssModuleClassNames,
        TransformPassKind::StaticVarSubstitution,
        TransformPassKind::CompressColorLiterals,
        TransformPassKind::StripWhitespaceTrivia,
        TransformPassKind::PrintCss,
    ]
    .into_iter()
    .map(transform_pass_contract)
    .collect()
}

fn transform_pass_contract(kind: TransformPassKind) -> TransformPassContractV0 {
    TransformPassContractV0 {
        id: kind.id(),
        kind,
        layer: kind.layer(),
        reads_semantic_graph: kind.reads_semantic_graph(),
        reads_cascade_model: kind.reads_cascade_model(),
        writes_css: true,
        cascade_safe_obligation: cascade_safe_obligation(kind),
    }
}

fn cascade_safe_obligation(kind: TransformPassKind) -> &'static str {
    match kind {
        TransformPassKind::TreeShakeUnusedSelectors => {
            "may remove selectors only when bridge reachability and cascade witnesses prove no reachable element can observe them"
        }
        TransformPassKind::ResolveCssModulesComposes => {
            "must preserve exported class set and composed class provenance"
        }
        TransformPassKind::HashCssModuleClassNames => {
            "must rewrite every source and style reference through the same selector identity map"
        }
        TransformPassKind::StaticVarSubstitution => {
            "must preserve custom-property fixed-point semantics or emit a provenance-backed blocked result"
        }
        TransformPassKind::CompressColorLiterals => {
            "may rewrite only literal-equivalent color tokens"
        }
        TransformPassKind::StripWhitespaceTrivia => {
            "may remove only trivia outside syntax-sensitive token boundaries"
        }
        TransformPassKind::PrintCss => {
            "must emit a source-map trace for every non-trivia transformed span"
        }
    }
}

fn default_transform_dag_edges() -> Vec<TransformDagEdgeV0> {
    vec![
        TransformDagEdgeV0 {
            from: "tree-shake-unused-selectors",
            to: "resolve-css-modules-composes",
            reason: "composes resolution must see the reachable selector set",
        },
        TransformDagEdgeV0 {
            from: "resolve-css-modules-composes",
            to: "hash-css-module-class-names",
            reason: "hashing must run after composed class expansion",
        },
        TransformDagEdgeV0 {
            from: "static-var-substitution",
            to: "compress-color-literals",
            reason: "literal compression can only run after safe variable substitution",
        },
        TransformDagEdgeV0 {
            from: "compress-color-literals",
            to: "strip-whitespace-trivia",
            reason: "token-preserving commodity rewrites precede trivia stripping",
        },
        TransformDagEdgeV0 {
            from: "strip-whitespace-trivia",
            to: "print-css",
            reason: "printer consumes the final transform CST",
        },
    ]
}

#[cfg(test)]
mod tests {
    use super::{
        TransformLayer, TransformPassKind, build_transform_cst_artifact,
        summarize_omena_transform_cst_boundary,
    };

    #[test]
    fn exposes_transform_cst_boundary_with_semantic_aware_pass_contracts() {
        let boundary = summarize_omena_transform_cst_boundary();

        assert_eq!(boundary.schema_version, "0");
        assert_eq!(boundary.product, "omena-transform-cst.boundary");
        assert_eq!(boundary.semantic_aware_pass_count, 4);
        assert_eq!(boundary.commodity_pass_count, 2);
        assert!(boundary.all_passes_declare_cascade_obligation);
        assert!(boundary.provenance_preservation_required);
        assert!(boundary.pass_contracts.iter().any(|contract| {
            contract.kind == TransformPassKind::TreeShakeUnusedSelectors
                && contract.layer == TransformLayer::SemanticAware
                && contract.reads_semantic_graph
                && contract.reads_cascade_model
        }));
        assert!(boundary.dag_edges.iter().any(|edge| {
            edge.from == "resolve-css-modules-composes"
                && edge.to == "hash-css-module-class-names"
        }));
    }

    #[test]
    fn transform_cst_artifact_preserves_semantic_signature_and_pass_ids() {
        let artifact = build_transform_cst_artifact(
            ".button { color: var(--brand); }",
            "semantic:button:brand",
            &[
                TransformPassKind::StaticVarSubstitution,
                TransformPassKind::CompressColorLiterals,
            ],
        );

        assert_eq!(artifact.product, "omena-transform-cst.artifact");
        assert_eq!(artifact.source_byte_len, 32);
        assert_eq!(artifact.semantic_signature, "semantic:button:brand");
        assert_eq!(
            artifact.pass_ids,
            vec!["static-var-substitution", "compress-color-literals"]
        );
        assert!(artifact.provenance_preserved);
    }
}

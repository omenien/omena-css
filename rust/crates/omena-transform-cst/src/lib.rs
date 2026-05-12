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
    WhitespaceStrip,
    CommentStrip,
    NumberCompression,
    UnitNormalization,
    ColorCompression,
    UrlQuoteStrip,
    StringQuoteNormalize,
    SelectorIsWhereCompression,
    ShorthandCombining,
    RuleDeduplication,
    RuleMerging,
    SelectorMerging,
    EmptyRuleRemoval,
    VendorPrefixing,
    LightDarkLowering,
    ColorMixLowering,
    OklchOklabLowering,
    ColorFunctionLowering,
    LogicalToPhysical,
    NestingUnwrap,
    ScopeFlatten,
    LayerFlatten,
    SupportsStaticEval,
    MediaStaticEval,
    CalcReduction,
    ImportInline,
    ScssModuleEvaluate,
    LessModuleEvaluate,
    HashCssModuleClassNames,
    ResolveCssModulesComposes,
    ValueResolution,
    StaticVarSubstitution,
    TreeShakeClass,
    TreeShakeKeyframes,
    TreeShakeValue,
    TreeShakeCustomProperty,
    DeadMediaBranchRemoval,
    DeadSupportsBranchRemoval,
    DesignTokenRouting,
    PrintCss,
}

pub const TRANSFORM_PASS_CATALOG_LEN: usize = 40;

pub const fn all_transform_pass_kinds() -> [TransformPassKind; TRANSFORM_PASS_CATALOG_LEN] {
    [
        TransformPassKind::WhitespaceStrip,
        TransformPassKind::CommentStrip,
        TransformPassKind::NumberCompression,
        TransformPassKind::UnitNormalization,
        TransformPassKind::ColorCompression,
        TransformPassKind::UrlQuoteStrip,
        TransformPassKind::StringQuoteNormalize,
        TransformPassKind::SelectorIsWhereCompression,
        TransformPassKind::ShorthandCombining,
        TransformPassKind::RuleDeduplication,
        TransformPassKind::RuleMerging,
        TransformPassKind::SelectorMerging,
        TransformPassKind::EmptyRuleRemoval,
        TransformPassKind::VendorPrefixing,
        TransformPassKind::LightDarkLowering,
        TransformPassKind::ColorMixLowering,
        TransformPassKind::OklchOklabLowering,
        TransformPassKind::ColorFunctionLowering,
        TransformPassKind::LogicalToPhysical,
        TransformPassKind::NestingUnwrap,
        TransformPassKind::ScopeFlatten,
        TransformPassKind::LayerFlatten,
        TransformPassKind::SupportsStaticEval,
        TransformPassKind::MediaStaticEval,
        TransformPassKind::CalcReduction,
        TransformPassKind::ImportInline,
        TransformPassKind::ScssModuleEvaluate,
        TransformPassKind::LessModuleEvaluate,
        TransformPassKind::HashCssModuleClassNames,
        TransformPassKind::ResolveCssModulesComposes,
        TransformPassKind::ValueResolution,
        TransformPassKind::StaticVarSubstitution,
        TransformPassKind::TreeShakeClass,
        TransformPassKind::TreeShakeKeyframes,
        TransformPassKind::TreeShakeValue,
        TransformPassKind::TreeShakeCustomProperty,
        TransformPassKind::DeadMediaBranchRemoval,
        TransformPassKind::DeadSupportsBranchRemoval,
        TransformPassKind::DesignTokenRouting,
        TransformPassKind::PrintCss,
    ]
}

impl TransformPassKind {
    pub const fn ordinal(self) -> u8 {
        match self {
            Self::WhitespaceStrip => 1,
            Self::CommentStrip => 2,
            Self::NumberCompression => 3,
            Self::UnitNormalization => 4,
            Self::ColorCompression => 5,
            Self::UrlQuoteStrip => 6,
            Self::StringQuoteNormalize => 7,
            Self::SelectorIsWhereCompression => 8,
            Self::ShorthandCombining => 9,
            Self::RuleDeduplication => 10,
            Self::RuleMerging => 11,
            Self::SelectorMerging => 12,
            Self::EmptyRuleRemoval => 13,
            Self::VendorPrefixing => 14,
            Self::LightDarkLowering => 15,
            Self::ColorMixLowering => 16,
            Self::OklchOklabLowering => 17,
            Self::ColorFunctionLowering => 18,
            Self::LogicalToPhysical => 19,
            Self::NestingUnwrap => 20,
            Self::ScopeFlatten => 21,
            Self::LayerFlatten => 22,
            Self::SupportsStaticEval => 23,
            Self::MediaStaticEval => 24,
            Self::CalcReduction => 25,
            Self::ImportInline => 26,
            Self::ScssModuleEvaluate => 27,
            Self::LessModuleEvaluate => 28,
            Self::HashCssModuleClassNames => 29,
            Self::ResolveCssModulesComposes => 30,
            Self::ValueResolution => 31,
            Self::StaticVarSubstitution => 32,
            Self::TreeShakeClass => 33,
            Self::TreeShakeKeyframes => 34,
            Self::TreeShakeValue => 35,
            Self::TreeShakeCustomProperty => 36,
            Self::DeadMediaBranchRemoval => 37,
            Self::DeadSupportsBranchRemoval => 38,
            Self::DesignTokenRouting => 39,
            Self::PrintCss => 40,
        }
    }

    pub const fn label(self) -> &'static str {
        self.id()
    }

    pub const fn title(self) -> &'static str {
        match self {
            Self::WhitespaceStrip => "whitespace strip",
            Self::CommentStrip => "comment strip",
            Self::NumberCompression => "number compression",
            Self::UnitNormalization => "unit normalization",
            Self::ColorCompression => "color compression",
            Self::UrlQuoteStrip => "url quote strip",
            Self::StringQuoteNormalize => "string quote normalize",
            Self::SelectorIsWhereCompression => "selector :is/:where compression",
            Self::ShorthandCombining => "shorthand combining",
            Self::RuleDeduplication => "rule deduplication",
            Self::RuleMerging => "rule merging",
            Self::SelectorMerging => "selector merging",
            Self::EmptyRuleRemoval => "empty rule removal",
            Self::VendorPrefixing => "vendor prefixing",
            Self::LightDarkLowering => "light-dark lowering",
            Self::ColorMixLowering => "color-mix lowering",
            Self::OklchOklabLowering => "oklch/oklab lowering",
            Self::ColorFunctionLowering => "color() lowering",
            Self::LogicalToPhysical => "logical to physical",
            Self::NestingUnwrap => "nesting unwrap",
            Self::ScopeFlatten => "@scope flatten",
            Self::LayerFlatten => "@layer flatten",
            Self::SupportsStaticEval => "@supports static eval",
            Self::MediaStaticEval => "@media static eval",
            Self::CalcReduction => "calc() reduction",
            Self::ImportInline => "@import inline",
            Self::ScssModuleEvaluate => "SCSS module evaluate",
            Self::LessModuleEvaluate => "Less module evaluate",
            Self::HashCssModuleClassNames => "CSS Modules class hashing",
            Self::ResolveCssModulesComposes => "composes resolution",
            Self::ValueResolution => "@value resolution",
            Self::StaticVarSubstitution => "custom property static resolve",
            Self::TreeShakeClass => "tree shaking class",
            Self::TreeShakeKeyframes => "tree shaking keyframes",
            Self::TreeShakeValue => "tree shaking value",
            Self::TreeShakeCustomProperty => "tree shaking custom-property",
            Self::DeadMediaBranchRemoval => "dead @media branch removal",
            Self::DeadSupportsBranchRemoval => "dead @supports branch removal",
            Self::DesignTokenRouting => "design-token routing",
            Self::PrintCss => "printer + sourcemap composer",
        }
    }

    pub const fn id(self) -> &'static str {
        match self {
            Self::WhitespaceStrip => "whitespace-strip",
            Self::CommentStrip => "comment-strip",
            Self::NumberCompression => "number-compression",
            Self::UnitNormalization => "unit-normalization",
            Self::ColorCompression => "color-compression",
            Self::UrlQuoteStrip => "url-quote-strip",
            Self::StringQuoteNormalize => "string-quote-normalize",
            Self::SelectorIsWhereCompression => "selector-is-where-compression",
            Self::ShorthandCombining => "shorthand-combining",
            Self::RuleDeduplication => "rule-deduplication",
            Self::RuleMerging => "rule-merging",
            Self::SelectorMerging => "selector-merging",
            Self::EmptyRuleRemoval => "empty-rule-removal",
            Self::VendorPrefixing => "vendor-prefixing",
            Self::LightDarkLowering => "light-dark-lowering",
            Self::ColorMixLowering => "color-mix-lowering",
            Self::OklchOklabLowering => "oklch-oklab-lowering",
            Self::ColorFunctionLowering => "color-function-lowering",
            Self::LogicalToPhysical => "logical-to-physical",
            Self::NestingUnwrap => "nesting-unwrap",
            Self::ScopeFlatten => "scope-flatten",
            Self::LayerFlatten => "layer-flatten",
            Self::SupportsStaticEval => "supports-static-eval",
            Self::MediaStaticEval => "media-static-eval",
            Self::CalcReduction => "calc-reduction",
            Self::ImportInline => "import-inline",
            Self::ScssModuleEvaluate => "scss-module-evaluate",
            Self::LessModuleEvaluate => "less-module-evaluate",
            Self::HashCssModuleClassNames => "css-modules-class-hashing",
            Self::ResolveCssModulesComposes => "composes-resolution",
            Self::ValueResolution => "value-resolution",
            Self::StaticVarSubstitution => "custom-property-static-resolve",
            Self::TreeShakeClass => "tree-shake-class",
            Self::TreeShakeKeyframes => "tree-shake-keyframes",
            Self::TreeShakeValue => "tree-shake-value",
            Self::TreeShakeCustomProperty => "tree-shake-custom-property",
            Self::DeadMediaBranchRemoval => "dead-media-branch-removal",
            Self::DeadSupportsBranchRemoval => "dead-supports-branch-removal",
            Self::DesignTokenRouting => "design-token-routing",
            Self::PrintCss => "print-css",
        }
    }

    pub const fn layer(self) -> TransformLayer {
        match self {
            Self::ImportInline
            | Self::ScssModuleEvaluate
            | Self::LessModuleEvaluate
            | Self::HashCssModuleClassNames
            | Self::ResolveCssModulesComposes
            | Self::ValueResolution
            | Self::StaticVarSubstitution
            | Self::TreeShakeClass
            | Self::TreeShakeKeyframes
            | Self::TreeShakeValue
            | Self::TreeShakeCustomProperty
            | Self::DeadMediaBranchRemoval
            | Self::DeadSupportsBranchRemoval
            | Self::DesignTokenRouting => TransformLayer::SemanticAware,
            Self::PrintCss => TransformLayer::Emission,
            _ => TransformLayer::Commodity,
        }
    }

    pub const fn reads_semantic_graph(self) -> bool {
        matches!(
            self,
            Self::ImportInline
                | Self::ScssModuleEvaluate
                | Self::LessModuleEvaluate
                | Self::HashCssModuleClassNames
                | Self::ResolveCssModulesComposes
                | Self::ValueResolution
                | Self::StaticVarSubstitution
                | Self::TreeShakeClass
                | Self::TreeShakeKeyframes
                | Self::TreeShakeValue
                | Self::TreeShakeCustomProperty
                | Self::DeadMediaBranchRemoval
                | Self::DeadSupportsBranchRemoval
                | Self::DesignTokenRouting
        )
    }

    pub const fn reads_cascade_model(self) -> bool {
        matches!(
            self,
            Self::ShorthandCombining
                | Self::RuleDeduplication
                | Self::RuleMerging
                | Self::SelectorMerging
                | Self::ScopeFlatten
                | Self::LayerFlatten
                | Self::StaticVarSubstitution
                | Self::DeadMediaBranchRemoval
                | Self::DeadSupportsBranchRemoval
        )
    }

    pub const fn read_model(self) -> TransformPassReadModel {
        match self {
            Self::VendorPrefixing
            | Self::LightDarkLowering
            | Self::ColorMixLowering
            | Self::OklchOklabLowering
            | Self::ColorFunctionLowering
            | Self::LogicalToPhysical
            | Self::NestingUnwrap => TransformPassReadModel::TargetData,
            Self::ShorthandCombining
            | Self::RuleDeduplication
            | Self::RuleMerging
            | Self::SelectorMerging
            | Self::ScopeFlatten
            | Self::LayerFlatten
            | Self::StaticVarSubstitution
            | Self::DeadMediaBranchRemoval
            | Self::DeadSupportsBranchRemoval => TransformPassReadModel::CascadeModel,
            Self::TreeShakeClass
            | Self::TreeShakeKeyframes
            | Self::TreeShakeValue
            | Self::TreeShakeCustomProperty
            | Self::DesignTokenRouting => TransformPassReadModel::BridgeReachability,
            Self::ImportInline
            | Self::ScssModuleEvaluate
            | Self::LessModuleEvaluate
            | Self::HashCssModuleClassNames
            | Self::ResolveCssModulesComposes
            | Self::ValueResolution => TransformPassReadModel::SemanticGraph,
            Self::PrintCss => TransformPassReadModel::Emission,
            _ => TransformPassReadModel::SyntaxOnly,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum TransformPassReadModel {
    SyntaxOnly,
    TargetData,
    CascadeModel,
    SemanticGraph,
    BridgeReachability,
    Emission,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TransformPassContractV0 {
    pub ordinal: u8,
    pub label: &'static str,
    pub id: &'static str,
    pub title: &'static str,
    pub kind: TransformPassKind,
    pub layer: TransformLayer,
    pub read_model: TransformPassReadModel,
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
    pub pass_catalog_count: usize,
    pub semantic_aware_pass_count: usize,
    pub commodity_pass_count: usize,
    pub emission_pass_count: usize,
    pub full_pass_catalog_covered: bool,
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
    let emission_pass_count = pass_contracts
        .iter()
        .filter(|contract| contract.layer == TransformLayer::Emission)
        .count();
    let all_passes_declare_cascade_obligation = pass_contracts
        .iter()
        .all(|contract| !contract.cascade_safe_obligation.is_empty());
    let pass_catalog_count = pass_contracts.len();

    TransformCstBoundarySummaryV0 {
        schema_version: "0",
        product: "omena-transform-cst.boundary",
        representation: "post-semantic-provenance-preserving-transform-cst",
        pass_contracts,
        dag_edges: default_transform_dag_edges(),
        pass_catalog_count,
        semantic_aware_pass_count,
        commodity_pass_count,
        emission_pass_count,
        full_pass_catalog_covered: pass_catalog_count == TRANSFORM_PASS_CATALOG_LEN,
        all_passes_declare_cascade_obligation,
        provenance_preservation_required: true,
        next_surfaces: Vec::new(),
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

pub fn default_transform_pass_contracts() -> Vec<TransformPassContractV0> {
    all_transform_pass_kinds()
        .into_iter()
        .map(transform_pass_contract)
        .collect()
}

fn transform_pass_contract(kind: TransformPassKind) -> TransformPassContractV0 {
    TransformPassContractV0 {
        ordinal: kind.ordinal(),
        label: kind.label(),
        id: kind.id(),
        title: kind.title(),
        kind,
        layer: kind.layer(),
        read_model: kind.read_model(),
        reads_semantic_graph: kind.reads_semantic_graph(),
        reads_cascade_model: kind.reads_cascade_model(),
        writes_css: true,
        cascade_safe_obligation: cascade_safe_obligation(kind),
    }
}

fn cascade_safe_obligation(kind: TransformPassKind) -> &'static str {
    match kind {
        TransformPassKind::WhitespaceStrip => {
            "may remove only whitespace outside string, url, attr, and calc-sensitive token boundaries"
        }
        TransformPassKind::CommentStrip => {
            "may remove comments only when source-map provenance preserves the removed span"
        }
        TransformPassKind::NumberCompression => {
            "may rewrite only numerically equivalent literal tokens"
        }
        TransformPassKind::UnitNormalization => {
            "may normalize only dimension values whose computed value is unchanged"
        }
        TransformPassKind::ColorCompression => "may rewrite only color-equivalent literal tokens",
        TransformPassKind::UrlQuoteStrip => {
            "may remove url quotes only when the unquoted token grammar remains equivalent"
        }
        TransformPassKind::StringQuoteNormalize => {
            "may normalize string quotes only when escaped contents remain byte-equivalent after decoding"
        }
        TransformPassKind::SelectorIsWhereCompression => {
            "must preserve selector specificity and matching semantics under the cascade model"
        }
        TransformPassKind::ShorthandCombining => {
            "must prove longhand and shorthand cascade outcomes are equivalent"
        }
        TransformPassKind::RuleDeduplication => {
            "must preserve origin, layer, specificity, and order for every surviving declaration"
        }
        TransformPassKind::RuleMerging => {
            "must prove merged rule order cannot change declaration winners"
        }
        TransformPassKind::SelectorMerging => {
            "must preserve selector identity and post-hash module semantics"
        }
        TransformPassKind::EmptyRuleRemoval => {
            "may remove rules only when no source-visible semantic marker is attached"
        }
        TransformPassKind::VendorPrefixing => {
            "must add target-required prefixed declarations without changing modern target outcomes"
        }
        TransformPassKind::LightDarkLowering => {
            "must lower only when target data requires fallback branches and provenance tracks both branches"
        }
        TransformPassKind::ColorMixLowering => {
            "must lower only when color-space conversion is target-equivalent"
        }
        TransformPassKind::OklchOklabLowering => {
            "must preserve color semantics within the configured target fallback precision"
        }
        TransformPassKind::ColorFunctionLowering => {
            "must preserve color semantics within the configured target fallback precision"
        }
        TransformPassKind::LogicalToPhysical => {
            "must run only under explicit directionality options"
        }
        TransformPassKind::NestingUnwrap => {
            "must preserve nested selector expansion and specificity"
        }
        TransformPassKind::ScopeFlatten => {
            "must preserve scoped matching semantics or emit a blocked result"
        }
        TransformPassKind::LayerFlatten => "must preserve layer order in CascadeKey comparison",
        TransformPassKind::SupportsStaticEval => {
            "may remove branches only when the target feature predicate is known"
        }
        TransformPassKind::MediaStaticEval => {
            "may remove branches only when the configured media predicate is known"
        }
        TransformPassKind::CalcReduction => {
            "may reduce only syntax-equivalent or computed-value-equivalent calc expressions"
        }
        TransformPassKind::ImportInline => {
            "must preserve import-site media, supports, layer wrappers, and source provenance"
        }
        TransformPassKind::ScssModuleEvaluate => {
            "must preserve SCSS namespace, show/hide, mixin, variable, and source provenance facts"
        }
        TransformPassKind::LessModuleEvaluate => {
            "must preserve Less variable, mixin, namespace, and source provenance facts"
        }
        TransformPassKind::HashCssModuleClassNames => {
            "must rewrite every source and style reference through the same selector identity map"
        }
        TransformPassKind::ResolveCssModulesComposes => {
            "must preserve exported class set and composed class provenance"
        }
        TransformPassKind::ValueResolution => {
            "must preserve @value graph resolution and cycle diagnostics"
        }
        TransformPassKind::StaticVarSubstitution => {
            "must preserve custom-property fixed-point semantics or emit a provenance-backed blocked result"
        }
        TransformPassKind::TreeShakeClass => {
            "may remove classes only when bridge reachability proves no reachable source expression observes them"
        }
        TransformPassKind::TreeShakeKeyframes => {
            "may remove keyframes only when animation-name reachability proves they are unobservable"
        }
        TransformPassKind::TreeShakeValue => {
            "may remove @value declarations only when value-graph traversal proves they are unreachable"
        }
        TransformPassKind::TreeShakeCustomProperty => {
            "may remove custom properties only when var() reachability proves they are unobservable"
        }
        TransformPassKind::DeadMediaBranchRemoval => {
            "may remove @media branches only when target and cascade witnesses prove deadness"
        }
        TransformPassKind::DeadSupportsBranchRemoval => {
            "may remove @supports branches only when target and cascade witnesses prove deadness"
        }
        TransformPassKind::DesignTokenRouting => {
            "must preserve design-token provenance while routing declarations across package boundaries"
        }
        TransformPassKind::PrintCss => {
            "must emit a source-map trace for every non-trivia transformed span"
        }
    }
}

pub fn default_transform_dag_edges() -> Vec<TransformDagEdgeV0> {
    vec![
        TransformDagEdgeV0 {
            from: "import-inline",
            to: "custom-property-static-resolve",
            reason: "var() resolution needs the full custom-property graph from inlined files",
        },
        TransformDagEdgeV0 {
            from: "scss-module-evaluate",
            to: "custom-property-static-resolve",
            reason: "SCSS evaluation can introduce custom-property declarations",
        },
        TransformDagEdgeV0 {
            from: "less-module-evaluate",
            to: "custom-property-static-resolve",
            reason: "Less evaluation can introduce custom-property declarations",
        },
        TransformDagEdgeV0 {
            from: "composes-resolution",
            to: "css-modules-class-hashing",
            reason: "hashing must run after composed class expansion",
        },
        TransformDagEdgeV0 {
            from: "css-modules-class-hashing",
            to: "selector-merging",
            reason: "selector merging must see post-hash selector identities",
        },
        TransformDagEdgeV0 {
            from: "custom-property-static-resolve",
            to: "calc-reduction",
            reason: "var() inside calc may resolve to numeric literals that enable reduction",
        },
        TransformDagEdgeV0 {
            from: "tree-shake-class",
            to: "rule-deduplication",
            reason: "tree shaking must run before rule deduplication can hide dead rules",
        },
        TransformDagEdgeV0 {
            from: "tree-shake-keyframes",
            to: "rule-deduplication",
            reason: "keyframe reachability must settle before rule deduplication",
        },
        TransformDagEdgeV0 {
            from: "tree-shake-value",
            to: "rule-deduplication",
            reason: "@value reachability must settle before rule deduplication",
        },
        TransformDagEdgeV0 {
            from: "tree-shake-custom-property",
            to: "rule-deduplication",
            reason: "custom-property reachability must settle before rule deduplication",
        },
        TransformDagEdgeV0 {
            from: "light-dark-lowering",
            to: "vendor-prefixing",
            reason: "prefixing runs after target lowering produces final declarations",
        },
        TransformDagEdgeV0 {
            from: "color-mix-lowering",
            to: "vendor-prefixing",
            reason: "prefixing runs after target lowering produces final declarations",
        },
        TransformDagEdgeV0 {
            from: "oklch-oklab-lowering",
            to: "vendor-prefixing",
            reason: "prefixing runs after target lowering produces final declarations",
        },
        TransformDagEdgeV0 {
            from: "color-function-lowering",
            to: "vendor-prefixing",
            reason: "prefixing runs after target lowering produces final declarations",
        },
        TransformDagEdgeV0 {
            from: "logical-to-physical",
            to: "vendor-prefixing",
            reason: "prefixing runs after target lowering produces final declarations",
        },
        TransformDagEdgeV0 {
            from: "nesting-unwrap",
            to: "vendor-prefixing",
            reason: "prefixing runs after target lowering produces final declarations",
        },
        TransformDagEdgeV0 {
            from: "scope-flatten",
            to: "vendor-prefixing",
            reason: "prefixing runs after target lowering produces final declarations",
        },
        TransformDagEdgeV0 {
            from: "layer-flatten",
            to: "vendor-prefixing",
            reason: "prefixing runs after target lowering produces final declarations",
        },
        TransformDagEdgeV0 {
            from: "supports-static-eval",
            to: "vendor-prefixing",
            reason: "prefixing runs after target branch evaluation produces final declarations",
        },
        TransformDagEdgeV0 {
            from: "media-static-eval",
            to: "vendor-prefixing",
            reason: "prefixing runs after target branch evaluation produces final declarations",
        },
        TransformDagEdgeV0 {
            from: "calc-reduction",
            to: "print-css",
            reason: "printer consumes the final reduced transform CST",
        },
        TransformDagEdgeV0 {
            from: "whitespace-strip",
            to: "print-css",
            reason: "printer consumes the final trivia policy",
        },
    ]
}

#[cfg(test)]
mod tests {
    use super::{
        TRANSFORM_PASS_CATALOG_LEN, TransformLayer, TransformPassKind,
        build_transform_cst_artifact, summarize_omena_transform_cst_boundary,
    };

    #[test]
    fn exposes_transform_cst_boundary_with_full_pass_catalog() {
        let boundary = summarize_omena_transform_cst_boundary();

        assert_eq!(boundary.schema_version, "0");
        assert_eq!(boundary.product, "omena-transform-cst.boundary");
        assert_eq!(boundary.pass_catalog_count, TRANSFORM_PASS_CATALOG_LEN);
        assert!(boundary.full_pass_catalog_covered);
        assert_eq!(boundary.semantic_aware_pass_count, 14);
        assert_eq!(boundary.commodity_pass_count, 25);
        assert_eq!(boundary.emission_pass_count, 1);
        assert!(boundary.all_passes_declare_cascade_obligation);
        assert!(boundary.provenance_preservation_required);
        assert!(!boundary.next_surfaces.contains(&"omena-transform-passes"));
        assert!(!boundary.next_surfaces.contains(&"omena-transform-print"));
        assert!(!boundary.next_surfaces.contains(&"salsaTransformQueries"));
        assert!(!boundary.next_surfaces.contains(&"sourceMapSpanPrecision"));
        assert!(boundary.pass_contracts.iter().any(|contract| {
            contract.kind == TransformPassKind::TreeShakeClass
                && contract.label == "tree-shake-class"
                && contract.layer == TransformLayer::SemanticAware
                && contract.reads_semantic_graph
        }));
        assert!(boundary.dag_edges.iter().any(|edge| {
            edge.from == "composes-resolution" && edge.to == "css-modules-class-hashing"
        }));
    }

    #[test]
    fn transform_cst_artifact_preserves_semantic_signature_and_pass_ids() {
        let artifact = build_transform_cst_artifact(
            ".button { color: var(--brand); }",
            "semantic:button:brand",
            &[
                TransformPassKind::StaticVarSubstitution,
                TransformPassKind::ColorCompression,
            ],
        );

        assert_eq!(artifact.product, "omena-transform-cst.artifact");
        assert_eq!(artifact.source_byte_len, 32);
        assert_eq!(artifact.semantic_signature, "semantic:button:brand");
        assert_eq!(
            artifact.pass_ids,
            vec!["custom-property-static-resolve", "color-compression"]
        );
        assert!(artifact.provenance_preserved);
    }
}

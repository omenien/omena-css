//! Transform CST contract substrate for the post-v5 omena-css track.
//!
//! This crate intentionally starts at the contract layer: transform passes are
//! only valid when they declare which semantic/cascade facts they read and what
//! cascade-safety obligation they must preserve.

pub use omena_parser::StyleDialect;
use omena_parser::{
    ParsedAnimationFactKind, ParsedCssModuleComposesFactKind, ParsedCssModuleValueFactKind,
    ParsedIcssFactKind, ParsedSassSymbolFactKind, ParsedSelectorFactKind, ParsedVariableFactKind,
    collect_style_facts,
};
use serde::Serialize;
use std::{borrow::Cow, collections::BTreeMap};

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
    StalePrefixRemoval,
    LightDarkLowering,
    ColorMixLowering,
    OklchOklabLowering,
    ColorFunctionLowering,
    RelativeColorLowering,
    LogicalToPhysical,
    NestingUnwrap,
    ScopeFlatten,
    LayerFlatten,
    SupportsStaticEval,
    MediaStaticEval,
    ContainerStaticEval,
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

pub const TRANSFORM_PASS_CATALOG_LEN: usize = 43;

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
        TransformPassKind::StalePrefixRemoval,
        TransformPassKind::LightDarkLowering,
        TransformPassKind::ColorMixLowering,
        TransformPassKind::OklchOklabLowering,
        TransformPassKind::ColorFunctionLowering,
        TransformPassKind::RelativeColorLowering,
        TransformPassKind::LogicalToPhysical,
        TransformPassKind::NestingUnwrap,
        TransformPassKind::ScopeFlatten,
        TransformPassKind::LayerFlatten,
        TransformPassKind::SupportsStaticEval,
        TransformPassKind::MediaStaticEval,
        TransformPassKind::ContainerStaticEval,
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
            Self::StalePrefixRemoval => 15,
            Self::LightDarkLowering => 16,
            Self::ColorMixLowering => 17,
            Self::OklchOklabLowering => 18,
            Self::ColorFunctionLowering => 19,
            Self::LogicalToPhysical => 20,
            Self::NestingUnwrap => 21,
            Self::ScopeFlatten => 22,
            Self::LayerFlatten => 23,
            Self::SupportsStaticEval => 24,
            Self::MediaStaticEval => 25,
            Self::CalcReduction => 26,
            Self::ImportInline => 27,
            Self::ScssModuleEvaluate => 28,
            Self::LessModuleEvaluate => 29,
            Self::HashCssModuleClassNames => 30,
            Self::ResolveCssModulesComposes => 31,
            Self::ValueResolution => 32,
            Self::StaticVarSubstitution => 33,
            Self::TreeShakeClass => 34,
            Self::TreeShakeKeyframes => 35,
            Self::TreeShakeValue => 36,
            Self::TreeShakeCustomProperty => 37,
            Self::DeadMediaBranchRemoval => 38,
            Self::DeadSupportsBranchRemoval => 39,
            Self::DesignTokenRouting => 40,
            Self::PrintCss => 41,
            Self::RelativeColorLowering => 42,
            Self::ContainerStaticEval => 43,
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
            Self::StringQuoteNormalize => "string and font value normalize",
            Self::SelectorIsWhereCompression => "selector alias compression",
            Self::ShorthandCombining => "shorthand combining",
            Self::RuleDeduplication => "rule deduplication",
            Self::RuleMerging => "rule merging",
            Self::SelectorMerging => "selector merging",
            Self::EmptyRuleRemoval => "empty rule removal",
            Self::VendorPrefixing => "vendor prefixing",
            Self::StalePrefixRemoval => "stale prefix removal",
            Self::LightDarkLowering => "light-dark lowering",
            Self::ColorMixLowering => "color-mix lowering",
            Self::OklchOklabLowering => "oklch/oklab lowering",
            Self::ColorFunctionLowering => "color() lowering",
            Self::RelativeColorLowering => "relative color lowering",
            Self::LogicalToPhysical => "logical to physical",
            Self::NestingUnwrap => "nesting unwrap",
            Self::ScopeFlatten => "@scope flatten",
            Self::LayerFlatten => "@layer flatten",
            Self::SupportsStaticEval => "@supports static eval",
            Self::MediaStaticEval => "@media static eval",
            Self::ContainerStaticEval => "@container static eval",
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
            Self::StalePrefixRemoval => "stale-prefix-removal",
            Self::LightDarkLowering => "light-dark-lowering",
            Self::ColorMixLowering => "color-mix-lowering",
            Self::OklchOklabLowering => "oklch-oklab-lowering",
            Self::ColorFunctionLowering => "color-function-lowering",
            Self::RelativeColorLowering => "relative-color-lowering",
            Self::LogicalToPhysical => "logical-to-physical",
            Self::NestingUnwrap => "nesting-unwrap",
            Self::ScopeFlatten => "scope-flatten",
            Self::LayerFlatten => "layer-flatten",
            Self::SupportsStaticEval => "supports-static-eval",
            Self::MediaStaticEval => "media-static-eval",
            Self::ContainerStaticEval => "container-static-eval",
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
            | Self::StalePrefixRemoval
            | Self::LightDarkLowering
            | Self::ColorMixLowering
            | Self::OklchOklabLowering
            | Self::ColorFunctionLowering
            | Self::RelativeColorLowering
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
    pub cascade_safe: bool,
    pub cascade_safety_witness: CascadeSafetyWitnessV0,
    pub cascade_safe_obligation: &'static str,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CascadeSafetyWitnessV0 {
    pub pass_id: &'static str,
    pub obligation: &'static str,
    pub enforced_at: &'static str,
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
    pub all_passes_have_compile_time_cascade_witness: bool,
    pub stable_transform_ir_ready: bool,
    pub provenance_derivation_forest_scaffold_ready: bool,
    pub provenance_preservation_required: bool,
    pub next_surfaces: Vec<&'static str>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum StableTransformIrNodeKindV0 {
    ClassSelector,
    IdSelector,
    PlaceholderSelector,
    CustomPropertyDeclaration,
    CustomPropertyReference,
    ScssVariableDeclaration,
    ScssVariableReference,
    LessVariableDeclaration,
    LessVariableReference,
    SassSymbolDeclaration,
    SassSymbolReference,
    SassModuleEdge,
    KeyframesDeclaration,
    AnimationNameReference,
    CssModuleValueDefinition,
    CssModuleValueReference,
    CssModuleValueImportSource,
    CssModuleComposesTarget,
    CssModuleComposesImportSource,
    IcssExportName,
    IcssImportLocalName,
    IcssImportRemoteName,
    IcssImportSource,
    AtRule,
}

impl StableTransformIrNodeKindV0 {
    pub const fn id(self) -> &'static str {
        match self {
            Self::ClassSelector => "class-selector",
            Self::IdSelector => "id-selector",
            Self::PlaceholderSelector => "placeholder-selector",
            Self::CustomPropertyDeclaration => "custom-property-declaration",
            Self::CustomPropertyReference => "custom-property-reference",
            Self::ScssVariableDeclaration => "scss-variable-declaration",
            Self::ScssVariableReference => "scss-variable-reference",
            Self::LessVariableDeclaration => "less-variable-declaration",
            Self::LessVariableReference => "less-variable-reference",
            Self::SassSymbolDeclaration => "sass-symbol-declaration",
            Self::SassSymbolReference => "sass-symbol-reference",
            Self::SassModuleEdge => "sass-module-edge",
            Self::KeyframesDeclaration => "keyframes-declaration",
            Self::AnimationNameReference => "animation-name-reference",
            Self::CssModuleValueDefinition => "css-module-value-definition",
            Self::CssModuleValueReference => "css-module-value-reference",
            Self::CssModuleValueImportSource => "css-module-value-import-source",
            Self::CssModuleComposesTarget => "css-module-composes-target",
            Self::CssModuleComposesImportSource => "css-module-composes-import-source",
            Self::IcssExportName => "icss-export-name",
            Self::IcssImportLocalName => "icss-import-local-name",
            Self::IcssImportRemoteName => "icss-import-remote-name",
            Self::IcssImportSource => "icss-import-source",
            Self::AtRule => "at-rule",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize)]
#[serde(transparent)]
pub struct StableNodeKeyV0(pub String);

impl StableNodeKeyV0 {
    pub fn as_str(&self) -> &str {
        self.0.as_str()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct StableTransformIrNodeV0 {
    pub node_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub node_key: Option<StableNodeKeyV0>,
    pub kind: StableTransformIrNodeKindV0,
    pub kind_id: &'static str,
    pub label: String,
    pub semantic_key: String,
    pub source_span_start: usize,
    pub source_span_end: usize,
    pub provenance_anchor_index: usize,
}

impl StableTransformIrNodeV0 {
    pub fn positional_node_id(&self) -> &str {
        self.node_id.as_str()
    }

    pub fn additive_node_key(&self) -> Option<&StableNodeKeyV0> {
        self.node_key.as_ref()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TransformCstProvenanceAnchorV0 {
    pub anchor_index: usize,
    pub node_id: String,
    pub semantic_key: String,
    pub source_span_start: usize,
    pub source_span_end: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct StableTransformIrV0 {
    pub schema_version: &'static str,
    pub product: &'static str,
    pub dialect: &'static str,
    pub source_byte_len: usize,
    pub semantic_signature: String,
    pub node_count: usize,
    pub parser_error_count: usize,
    pub contains_bogus_or_trivia: bool,
    pub stable_post_semantic_ir: bool,
    pub nodes: Vec<StableTransformIrNodeV0>,
    pub provenance_anchors: Vec<TransformCstProvenanceAnchorV0>,
}

pub const STABLE_TRANSFORM_IR_SCHEMA_VERSION_V0: &str = "0";

pub const STABLE_TRANSFORM_IR_NODE_IDENTITY_POLICY_V0: &str =
    "schema-v0-node-key-preferred-node-id-fallback";

impl StableTransformIrV0 {
    pub fn node_identity_policy(&self) -> &'static str {
        if self.schema_version == STABLE_TRANSFORM_IR_SCHEMA_VERSION_V0 {
            STABLE_TRANSFORM_IR_NODE_IDENTITY_POLICY_V0
        } else {
            "legacy-node-id-only"
        }
    }

    pub fn identity_key_for_node<'a>(&self, node: &'a StableTransformIrNodeV0) -> Cow<'a, str> {
        if self.schema_version == STABLE_TRANSFORM_IR_SCHEMA_VERSION_V0
            && let Some(node_key) = node.additive_node_key()
        {
            return Cow::Borrowed(node_key.as_str());
        }
        Cow::Borrowed(node.positional_node_id())
    }

    pub fn identity_key_at(&self, node_index: usize) -> Option<Cow<'_, str>> {
        self.nodes
            .get(node_index)
            .map(|node| self.identity_key_for_node(node))
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TransformCstArtifactV0 {
    pub schema_version: &'static str,
    pub product: &'static str,
    pub source_byte_len: usize,
    pub semantic_signature: String,
    pub stable_ir: StableTransformIrV0,
    pub stable_ir_node_count: usize,
    pub parser_error_count: usize,
    pub contains_bogus_or_trivia: bool,
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
    let all_passes_have_compile_time_cascade_witness = pass_contracts.iter().all(|contract| {
        contract.cascade_safe
            && contract.cascade_safety_witness.pass_id == contract.id
            && contract.cascade_safety_witness.obligation == contract.cascade_safe_obligation
            && contract.cascade_safety_witness.enforced_at == "compile-time-exhaustive-pass-catalog"
    });
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
        all_passes_have_compile_time_cascade_witness,
        stable_transform_ir_ready: true,
        provenance_derivation_forest_scaffold_ready: true,
        provenance_preservation_required: true,
        next_surfaces: Vec::new(),
    }
}

pub fn build_transform_cst_artifact(
    source: &str,
    semantic_signature: impl Into<String>,
    passes: &[TransformPassKind],
) -> TransformCstArtifactV0 {
    build_transform_cst_artifact_with_dialect(source, StyleDialect::Css, semantic_signature, passes)
}

pub fn build_transform_cst_artifact_with_dialect(
    source: &str,
    dialect: StyleDialect,
    semantic_signature: impl Into<String>,
    passes: &[TransformPassKind],
) -> TransformCstArtifactV0 {
    let semantic_signature = semantic_signature.into();
    let stable_ir =
        build_stable_transform_ir_from_source(source, dialect, semantic_signature.clone());
    let stable_ir_node_count = stable_ir.node_count;
    let parser_error_count = stable_ir.parser_error_count;
    let contains_bogus_or_trivia = stable_ir.contains_bogus_or_trivia;

    TransformCstArtifactV0 {
        schema_version: "0",
        product: "omena-transform-cst.artifact",
        source_byte_len: source.len(),
        semantic_signature,
        stable_ir,
        stable_ir_node_count,
        parser_error_count,
        contains_bogus_or_trivia,
        pass_ids: passes.iter().map(|pass| pass.id()).collect(),
        provenance_preserved: true,
    }
}

pub fn build_stable_transform_ir_from_source(
    source: &str,
    dialect: StyleDialect,
    semantic_signature: impl Into<String>,
) -> StableTransformIrV0 {
    let facts = collect_style_facts(source, dialect);
    let mut nodes = Vec::new();

    for selector in facts.selectors {
        push_ir_node(
            &mut nodes,
            stable_ir_selector_kind(selector.kind),
            selector.name,
            selector.range.start().into(),
            selector.range.end().into(),
        );
    }

    for variable in facts.variables {
        push_ir_node(
            &mut nodes,
            stable_ir_variable_kind(variable.kind),
            variable.name,
            variable.range.start().into(),
            variable.range.end().into(),
        );
    }

    for symbol in facts.sass_symbols {
        push_ir_node(
            &mut nodes,
            stable_ir_sass_symbol_kind(symbol.kind),
            format!("{}:{}", symbol.symbol_kind, symbol.name),
            symbol.range.start().into(),
            symbol.range.end().into(),
        );
    }

    for edge in facts.sass_module_edges {
        push_ir_node(
            &mut nodes,
            StableTransformIrNodeKindV0::SassModuleEdge,
            edge.source,
            edge.range.start().into(),
            edge.range.end().into(),
        );
    }

    for animation in facts.animations {
        push_ir_node(
            &mut nodes,
            stable_ir_animation_kind(animation.kind),
            animation.name,
            animation.range.start().into(),
            animation.range.end().into(),
        );
    }

    for value in facts.css_module_values {
        push_ir_node(
            &mut nodes,
            stable_ir_css_module_value_kind(value.kind),
            value.name,
            value.range.start().into(),
            value.range.end().into(),
        );
    }

    for composes in facts.css_module_composes {
        push_ir_node(
            &mut nodes,
            stable_ir_css_module_composes_kind(composes.kind),
            composes.name,
            composes.range.start().into(),
            composes.range.end().into(),
        );
    }

    for icss in facts.icss {
        push_ir_node(
            &mut nodes,
            stable_ir_icss_kind(icss.kind),
            icss.name,
            icss.range.start().into(),
            icss.range.end().into(),
        );
    }

    for at_rule in facts.at_rules {
        push_ir_node(
            &mut nodes,
            StableTransformIrNodeKindV0::AtRule,
            at_rule.name,
            at_rule.range.start().into(),
            at_rule.range.end().into(),
        );
    }

    nodes.sort_by(|left, right| {
        left.source_span_start
            .cmp(&right.source_span_start)
            .then_with(|| left.source_span_end.cmp(&right.source_span_end))
            .then_with(|| left.kind.cmp(&right.kind))
            .then_with(|| left.label.cmp(&right.label))
    });

    let mut provenance_anchors = Vec::with_capacity(nodes.len());
    let mut semantic_key_ordinals = BTreeMap::new();
    for (index, node) in nodes.iter_mut().enumerate() {
        let ordinal = semantic_key_ordinals
            .entry(node.semantic_key.clone())
            .and_modify(|count| *count += 1)
            .or_insert(0);
        node.node_id = format!("ir:{index}");
        node.node_key = Some(StableNodeKeyV0(format!(
            "{}#{}",
            node.semantic_key, *ordinal
        )));
        node.provenance_anchor_index = index;
        provenance_anchors.push(TransformCstProvenanceAnchorV0 {
            anchor_index: index,
            node_id: node.node_id.clone(),
            semantic_key: node.semantic_key.clone(),
            source_span_start: node.source_span_start,
            source_span_end: node.source_span_end,
        });
    }

    let node_count = nodes.len();
    let parser_error_count = facts.error_count;

    StableTransformIrV0 {
        schema_version: "0",
        product: "omena-transform-cst.stable-ir",
        dialect: transform_cst_style_dialect_label(dialect),
        source_byte_len: source.len(),
        semantic_signature: semantic_signature.into(),
        node_count,
        parser_error_count,
        contains_bogus_or_trivia: false,
        stable_post_semantic_ir: parser_error_count == 0,
        nodes,
        provenance_anchors,
    }
}

pub fn default_transform_pass_contracts() -> Vec<TransformPassContractV0> {
    all_transform_pass_kinds()
        .into_iter()
        .map(transform_pass_contract)
        .collect()
}

fn transform_pass_contract(kind: TransformPassKind) -> TransformPassContractV0 {
    let cascade_safety_witness = cascade_safety_witness(kind);

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
        cascade_safe: true,
        cascade_safety_witness,
        cascade_safe_obligation: cascade_safety_witness.obligation,
    }
}

pub const fn cascade_safety_witness(kind: TransformPassKind) -> CascadeSafetyWitnessV0 {
    CascadeSafetyWitnessV0 {
        pass_id: kind.id(),
        obligation: cascade_safe_obligation(kind),
        enforced_at: "compile-time-exhaustive-pass-catalog",
    }
}

pub const fn cascade_safe_obligation(kind: TransformPassKind) -> &'static str {
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
            "may normalize string quotes and font keyword aliases only when computed text and font values remain equivalent"
        }
        TransformPassKind::SelectorIsWhereCompression => {
            "must preserve selector specificity, keyframe timeline positions, and matching semantics under the cascade model"
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
        TransformPassKind::StalePrefixRemoval => {
            "may remove prefixed declarations only when an explicit mapping and exact unprefixed peer prove the prefix stale"
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
        TransformPassKind::RelativeColorLowering => {
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
        TransformPassKind::ContainerStaticEval => {
            "may remove @container branches only when the size condition is provably unsatisfiable regardless of container context"
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

fn push_ir_node(
    nodes: &mut Vec<StableTransformIrNodeV0>,
    kind: StableTransformIrNodeKindV0,
    label: impl Into<String>,
    source_span_start: usize,
    source_span_end: usize,
) {
    let label = label.into();
    let kind_id = kind.id();
    nodes.push(StableTransformIrNodeV0 {
        node_id: String::new(),
        node_key: None,
        kind,
        kind_id,
        semantic_key: format!("{kind_id}:{label}"),
        label,
        source_span_start,
        source_span_end,
        provenance_anchor_index: 0,
    });
}

const fn stable_ir_selector_kind(kind: ParsedSelectorFactKind) -> StableTransformIrNodeKindV0 {
    match kind {
        ParsedSelectorFactKind::Class => StableTransformIrNodeKindV0::ClassSelector,
        ParsedSelectorFactKind::Id => StableTransformIrNodeKindV0::IdSelector,
        ParsedSelectorFactKind::Placeholder => StableTransformIrNodeKindV0::PlaceholderSelector,
    }
}

const fn stable_ir_variable_kind(kind: ParsedVariableFactKind) -> StableTransformIrNodeKindV0 {
    match kind {
        ParsedVariableFactKind::ScssDeclaration => {
            StableTransformIrNodeKindV0::ScssVariableDeclaration
        }
        ParsedVariableFactKind::ScssReference => StableTransformIrNodeKindV0::ScssVariableReference,
        ParsedVariableFactKind::LessDeclaration => {
            StableTransformIrNodeKindV0::LessVariableDeclaration
        }
        ParsedVariableFactKind::LessReference => StableTransformIrNodeKindV0::LessVariableReference,
        ParsedVariableFactKind::CustomPropertyDeclaration => {
            StableTransformIrNodeKindV0::CustomPropertyDeclaration
        }
        ParsedVariableFactKind::CustomPropertyReference => {
            StableTransformIrNodeKindV0::CustomPropertyReference
        }
    }
}

const fn stable_ir_sass_symbol_kind(kind: ParsedSassSymbolFactKind) -> StableTransformIrNodeKindV0 {
    match kind {
        ParsedSassSymbolFactKind::VariableDeclaration
        | ParsedSassSymbolFactKind::MixinDeclaration
        | ParsedSassSymbolFactKind::FunctionDeclaration => {
            StableTransformIrNodeKindV0::SassSymbolDeclaration
        }
        ParsedSassSymbolFactKind::VariableReference
        | ParsedSassSymbolFactKind::MixinInclude
        | ParsedSassSymbolFactKind::FunctionCall => {
            StableTransformIrNodeKindV0::SassSymbolReference
        }
    }
}

const fn stable_ir_animation_kind(kind: ParsedAnimationFactKind) -> StableTransformIrNodeKindV0 {
    match kind {
        ParsedAnimationFactKind::KeyframesDeclaration => {
            StableTransformIrNodeKindV0::KeyframesDeclaration
        }
        ParsedAnimationFactKind::AnimationNameReference => {
            StableTransformIrNodeKindV0::AnimationNameReference
        }
    }
}

const fn stable_ir_css_module_value_kind(
    kind: ParsedCssModuleValueFactKind,
) -> StableTransformIrNodeKindV0 {
    match kind {
        ParsedCssModuleValueFactKind::Definition => {
            StableTransformIrNodeKindV0::CssModuleValueDefinition
        }
        ParsedCssModuleValueFactKind::Reference => {
            StableTransformIrNodeKindV0::CssModuleValueReference
        }
        ParsedCssModuleValueFactKind::ImportSource => {
            StableTransformIrNodeKindV0::CssModuleValueImportSource
        }
    }
}

const fn stable_ir_css_module_composes_kind(
    kind: ParsedCssModuleComposesFactKind,
) -> StableTransformIrNodeKindV0 {
    match kind {
        ParsedCssModuleComposesFactKind::Target => {
            StableTransformIrNodeKindV0::CssModuleComposesTarget
        }
        ParsedCssModuleComposesFactKind::ImportSource => {
            StableTransformIrNodeKindV0::CssModuleComposesImportSource
        }
    }
}

const fn stable_ir_icss_kind(kind: ParsedIcssFactKind) -> StableTransformIrNodeKindV0 {
    match kind {
        ParsedIcssFactKind::ExportName => StableTransformIrNodeKindV0::IcssExportName,
        ParsedIcssFactKind::ImportLocalName => StableTransformIrNodeKindV0::IcssImportLocalName,
        ParsedIcssFactKind::ImportRemoteName => StableTransformIrNodeKindV0::IcssImportRemoteName,
        ParsedIcssFactKind::ImportSource => StableTransformIrNodeKindV0::IcssImportSource,
    }
}

pub const fn transform_cst_style_dialect_label(dialect: StyleDialect) -> &'static str {
    match dialect {
        StyleDialect::Css => "css",
        StyleDialect::Scss => "scss",
        StyleDialect::Sass => "sass",
        StyleDialect::Less => "less",
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
            from: "nesting-unwrap",
            to: "css-modules-class-hashing",
            reason: "hashing must run after nested selectors are expanded into final selector branches",
        },
        TransformDagEdgeV0 {
            from: "tree-shake-class",
            to: "css-modules-class-hashing",
            reason: "class reachability is expressed in authored selector names and must run before hashing rewrites them",
        },
        TransformDagEdgeV0 {
            from: "css-modules-class-hashing",
            to: "selector-merging",
            reason: "selector merging must see post-hash selector identities",
        },
        TransformDagEdgeV0 {
            from: "number-compression",
            to: "selector-merging",
            reason: "selector merging must see canonical declaration numeric values",
        },
        TransformDagEdgeV0 {
            from: "unit-normalization",
            to: "selector-merging",
            reason: "selector merging must see canonical declaration unit values",
        },
        TransformDagEdgeV0 {
            from: "color-compression",
            to: "selector-merging",
            reason: "selector merging must see canonical declaration color values",
        },
        TransformDagEdgeV0 {
            from: "url-quote-strip",
            to: "selector-merging",
            reason: "selector merging must see canonical url() values",
        },
        TransformDagEdgeV0 {
            from: "string-quote-normalize",
            to: "selector-merging",
            reason: "selector merging must see canonical string values",
        },
        TransformDagEdgeV0 {
            from: "shorthand-combining",
            to: "selector-merging",
            reason: "selector merging must see canonical shorthand declaration blocks",
        },
        TransformDagEdgeV0 {
            from: "shorthand-combining",
            to: "rule-merging",
            reason: "rule merging must see shorthand-combined declaration blocks before comparing adjacent rules",
        },
        TransformDagEdgeV0 {
            from: "calc-reduction",
            to: "selector-merging",
            reason: "selector merging must see reduced calc() declaration values",
        },
        TransformDagEdgeV0 {
            from: "selector-merging",
            to: "whitespace-strip",
            reason: "whitespace stripping must run after selector merging emits final selector lists",
        },
        TransformDagEdgeV0 {
            from: "custom-property-static-resolve",
            to: "calc-reduction",
            reason: "var() inside calc may resolve to numeric literals that enable reduction",
        },
        TransformDagEdgeV0 {
            from: "value-resolution",
            to: "supports-static-eval",
            reason: "@value references inside @supports preludes must resolve before static branch evaluation",
        },
        TransformDagEdgeV0 {
            from: "value-resolution",
            to: "media-static-eval",
            reason: "@value references inside @media preludes must resolve before static media normalization",
        },
        TransformDagEdgeV0 {
            from: "custom-property-static-resolve",
            to: "supports-static-eval",
            reason: "var() references inside @supports preludes must resolve before static branch evaluation",
        },
        TransformDagEdgeV0 {
            from: "custom-property-static-resolve",
            to: "media-static-eval",
            reason: "var() references inside @media preludes must resolve before static media normalization",
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
            from: "tree-shake-class",
            to: "empty-rule-removal",
            reason: "class tree shaking can leave ordinary and group rules empty",
        },
        TransformDagEdgeV0 {
            from: "tree-shake-keyframes",
            to: "empty-rule-removal",
            reason: "keyframe tree shaking can leave enclosing group rules empty",
        },
        TransformDagEdgeV0 {
            from: "tree-shake-value",
            to: "empty-rule-removal",
            reason: "@value tree shaking can leave module-only wrappers empty",
        },
        TransformDagEdgeV0 {
            from: "tree-shake-custom-property",
            to: "empty-rule-removal",
            reason: "custom-property tree shaking can leave declaration-only rules empty",
        },
        TransformDagEdgeV0 {
            from: "comment-strip",
            to: "empty-rule-removal",
            reason: "comment-only rules become removable empty rules after comment stripping",
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
            from: "relative-color-lowering",
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
            from: "vendor-prefixing",
            to: "stale-prefix-removal",
            reason: "stale-prefix removal must inspect the final vendor-prefix declaration set",
        },
        TransformDagEdgeV0 {
            from: "stale-prefix-removal",
            to: "print-css",
            reason: "printer consumes the final prefix-removal decisions",
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
        STABLE_TRANSFORM_IR_NODE_IDENTITY_POLICY_V0, StableTransformIrNodeKindV0, StyleDialect,
        TRANSFORM_PASS_CATALOG_LEN, TransformLayer, TransformPassKind,
        build_stable_transform_ir_from_source, build_transform_cst_artifact,
        summarize_omena_transform_cst_boundary,
    };

    #[test]
    fn exposes_transform_cst_boundary_with_full_pass_catalog() {
        let boundary = summarize_omena_transform_cst_boundary();

        assert_eq!(boundary.schema_version, "0");
        assert_eq!(boundary.product, "omena-transform-cst.boundary");
        assert_eq!(boundary.pass_catalog_count, TRANSFORM_PASS_CATALOG_LEN);
        assert!(boundary.full_pass_catalog_covered);
        assert_eq!(boundary.semantic_aware_pass_count, 14);
        assert_eq!(boundary.commodity_pass_count, 28);
        assert_eq!(boundary.emission_pass_count, 1);
        assert!(boundary.all_passes_declare_cascade_obligation);
        assert!(boundary.all_passes_have_compile_time_cascade_witness);
        assert!(boundary.stable_transform_ir_ready);
        assert!(boundary.provenance_derivation_forest_scaffold_ready);
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
                && contract.cascade_safe
                && contract.cascade_safety_witness.pass_id == "tree-shake-class"
                && contract.cascade_safety_witness.enforced_at
                    == "compile-time-exhaustive-pass-catalog"
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
        assert_eq!(artifact.stable_ir.product, "omena-transform-cst.stable-ir");
        assert_eq!(artifact.stable_ir.dialect, "css");
        assert_eq!(artifact.parser_error_count, 0);
        assert!(!artifact.contains_bogus_or_trivia);
        assert!(artifact.stable_ir.stable_post_semantic_ir);
        assert_eq!(
            artifact.stable_ir_node_count,
            artifact.stable_ir.provenance_anchors.len()
        );
        assert_eq!(
            artifact.pass_ids,
            vec!["custom-property-static-resolve", "color-compression"]
        );
        assert!(artifact.provenance_preserved);
    }

    #[test]
    fn stable_transform_ir_consumes_parser_semantic_facts_without_trivia_or_bogus_nodes() {
        let ir = build_stable_transform_ir_from_source(
            r#"
@use "./tokens" as tokens;
@value primary from "./colors.module.css";
.button {
  composes: reset from "./reset.module.css";
  --brand: tokens.$brand;
  color: var(--brand);
}
"#,
            StyleDialect::Scss,
            "semantic:scss-button",
        );

        assert_eq!(ir.product, "omena-transform-cst.stable-ir");
        assert_eq!(ir.dialect, "scss");
        assert_eq!(ir.parser_error_count, 0);
        assert!(!ir.contains_bogus_or_trivia);
        assert!(ir.stable_post_semantic_ir);
        assert_eq!(ir.node_count, ir.nodes.len());
        assert_eq!(ir.node_count, ir.provenance_anchors.len());
        assert!(ir.nodes.iter().any(|node| {
            node.kind == StableTransformIrNodeKindV0::ClassSelector && node.label == "button"
        }));
        assert!(ir.nodes.iter().any(|node| {
            node.kind == StableTransformIrNodeKindV0::CustomPropertyDeclaration
                && node.label == "--brand"
        }));
        assert!(ir.nodes.iter().any(|node| {
            node.kind == StableTransformIrNodeKindV0::CustomPropertyReference
                && node.label == "--brand"
        }));
        assert!(ir.nodes.iter().any(|node| {
            node.kind == StableTransformIrNodeKindV0::SassModuleEdge && node.label == "./tokens"
        }));
        assert!(
            ir.nodes
                .windows(2)
                .all(|pair| pair[0].source_span_start <= pair[1].source_span_start)
        );
    }

    #[test]
    fn stable_transform_ir_mints_additive_source_order_node_keys() {
        let ir = build_stable_transform_ir_from_source(
            ".button { color: red; }\n.button { color: blue; }",
            StyleDialect::Css,
            "semantic:duplicate-button",
        );

        let button_nodes = ir
            .nodes
            .iter()
            .filter(|node| {
                node.kind == StableTransformIrNodeKindV0::ClassSelector && node.label == "button"
            })
            .collect::<Vec<_>>();

        assert_eq!(button_nodes.len(), 2);
        assert_eq!(button_nodes[0].node_id, "ir:0");
        assert_eq!(button_nodes[1].node_id, "ir:1");
        assert_eq!(
            button_nodes[0].node_key.as_ref().map(|key| key.as_str()),
            Some("class-selector:button#0")
        );
        assert_eq!(
            button_nodes[1].node_key.as_ref().map(|key| key.as_str()),
            Some("class-selector:button#1")
        );
        assert!(ir.nodes.iter().enumerate().all(|(index, node)| {
            node.node_id == format!("ir:{index}") && node.node_key.is_some()
        }));
    }

    #[test]
    fn stable_transform_ir_identity_reader_prefers_key_with_positional_fallback() {
        let mut ir = build_stable_transform_ir_from_source(
            ".button { color: red; }\n.button { color: blue; }",
            StyleDialect::Css,
            "semantic:duplicate-button",
        );

        assert_eq!(
            ir.node_identity_policy(),
            STABLE_TRANSFORM_IR_NODE_IDENTITY_POLICY_V0
        );
        assert_eq!(
            ir.identity_key_at(0).as_deref(),
            Some("class-selector:button#0")
        );
        assert_eq!(
            ir.identity_key_at(1).as_deref(),
            Some("class-selector:button#1")
        );

        ir.nodes[0].node_key = None;
        assert_eq!(ir.identity_key_at(0).as_deref(), Some("ir:0"));
        assert_eq!(
            ir.identity_key_at(1).as_deref(),
            Some("class-selector:button#1")
        );

        ir.schema_version = "future";
        assert_eq!(ir.node_identity_policy(), "legacy-node-id-only");
        assert_eq!(ir.identity_key_at(1).as_deref(), Some("ir:1"));
    }

    #[test]
    fn stable_transform_ir_identity_reader_preserves_serialized_node_shape()
    -> Result<(), serde_json::Error> {
        let mut ir = build_stable_transform_ir_from_source(
            ".button { color: red; }",
            StyleDialect::Css,
            "semantic:button",
        );
        let with_key_json = serde_json::to_string(&ir.nodes[0])?;
        assert!(with_key_json.contains("\"nodeId\":\"ir:0\""));
        assert!(with_key_json.contains("\"nodeKey\":\"class-selector:button#0\""));

        ir.nodes[0].node_key = None;
        let fallback_json = serde_json::to_string(&ir.nodes[0])?;
        assert!(fallback_json.contains("\"nodeId\":\"ir:0\""));
        assert!(!fallback_json.contains("nodeKey"));
        assert_eq!(ir.identity_key_at(0).as_deref(), Some("ir:0"));
        Ok(())
    }
}

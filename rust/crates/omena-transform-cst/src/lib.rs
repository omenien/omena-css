//! Transform CST contract substrate for the post-v5 omena-css track.
//!
//! This crate intentionally starts at the contract layer: transform passes are
//! only valid when they declare which semantic/cascade facts they read and what
//! cascade-safety obligation they must preserve.

use omena_cascade_proof::{
    CascadeSMTProofV0, SmtBackendV0, SmtVerdictV0, StubSmtBackendV0, TransformRewriteProofInputV0,
    smt_verify_transform_rewrite_candidate_v0,
};
use omena_evidence_graph::{
    EvidenceDemandEdgeV0, EvidenceGraphBuildErrorV0, EvidenceGraphV0, EvidenceNodeKeyV0,
    EvidenceNodeSeedV0, GuaranteeKindV0, ObligationFamilyIdV0, build_evidence_graph_from_edges_v0,
};
pub use omena_parser::StyleDialect;
use omena_parser::{
    ClosedWorldBundleV0, ParsedAnimationFactKind, ParsedCssModuleComposesFactKind,
    ParsedCssModuleValueFactKind, ParsedIcssFactKind, ParsedSassSymbolFactKind,
    ParsedSelectorFactKind, ParsedVariableFactKind, collect_style_facts,
};
use serde::{Serialize, ser::SerializeStruct};
use std::{borrow::Cow, collections::BTreeMap, sync::OnceLock};

mod pass_descriptor;
mod transform_ir;
pub use pass_descriptor::{
    TransformBuildProfileV0, TransformPassClassV0, TransformPassDescriptorV0,
    default_transform_pass_descriptors, transform_build_profile_from_passes, transform_pass_class,
    transform_pass_descriptor,
};
pub use transform_ir::{
    IrEditRegionV0, IrNodeIdV0, IrNodeKindV0, IrNodeV0, IrTargetV0, IrTransactionErrorV0,
    IrTransactionV0, IrTransactionValidationErrorV0, NodeTextOriginV0,
    TransformIrIdentityRoundTripV0, TransformIrIndexesV0, TransformIrKindIndexV0,
    TransformIrParentIndexV0, TransformIrParseErrorSpanV0, TransformIrPrintErrorV0, TransformIrV0,
    lower_transform_ir_from_source, materialize_transform_ir_printed_source,
    print_transform_ir_css, summarize_transform_ir_identity_round_trip,
};

use std::cell::RefCell;

const CASCADE_WITNESS_EVIDENCE_QUERY_V0: &str = "omena-transform-cst.cascade-safety-witness";
const CASCADE_WITNESS_EVIDENCE_EDGE_KIND_V0: &str = "cascade-safety-evidence";
pub const STABLE_NODE_KEY_STRING_ARM_EXPIRY_UTC_DATE_V0: &str = "2026-10-01";
pub const STABLE_NODE_KEY_TYPE_LABEL_V0: &str = "StableNodeKeyV0";

#[cfg(stable_node_key_string_arm_expired)]
compile_error!(
    "StableNodeKeyV0 string arm has passed its expiry date; migrate consumers to StableNodeKeyU64V0 or extend the expiry with a tracked decision."
);

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
    NativeCssStaticEval,
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

pub const TRANSFORM_PASS_CATALOG_LEN: usize = 44;
pub const NATIVE_CSS_STATIC_EVAL_SPEC_SNAPSHOT_V0: &str =
    "css-values-5-if-css-mixins-1-function-ed-2026-06-22";
pub const NATIVE_CSS_STATIC_EVAL_OPT_IN_POLICY_V0: &str =
    "explicit-pass-id-required-default-consumer-build-excludes";
pub const NATIVE_CSS_STATIC_EVAL_DIALECT_RESTRICTION_V0: &str = "css-only";

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
        TransformPassKind::NativeCssStaticEval,
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
            Self::NativeCssStaticEval => 44,
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
            Self::NativeCssStaticEval => "native CSS static eval",
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
            Self::NativeCssStaticEval => "native-css-static-eval",
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
            | Self::NestingUnwrap
            | Self::NativeCssStaticEval => TransformPassReadModel::TargetData,
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

    pub const fn explicit_opt_in_required(self) -> bool {
        matches!(self, Self::NativeCssStaticEval)
    }

    pub const fn dialect_restriction(self) -> Option<&'static str> {
        match self {
            Self::NativeCssStaticEval => Some(NATIVE_CSS_STATIC_EVAL_DIALECT_RESTRICTION_V0),
            _ => None,
        }
    }

    pub const fn spec_snapshot(self) -> Option<&'static str> {
        match self {
            Self::NativeCssStaticEval => Some(NATIVE_CSS_STATIC_EVAL_SPEC_SNAPSHOT_V0),
            _ => None,
        }
    }

    pub const fn opt_in_policy(self) -> Option<&'static str> {
        match self {
            Self::NativeCssStaticEval => Some(NATIVE_CSS_STATIC_EVAL_OPT_IN_POLICY_V0),
            _ => None,
        }
    }
}

thread_local! {
    static TRANSFORM_PASS_SORT_ORDINAL_OVERRIDES: RefCell<Option<[u8; TRANSFORM_PASS_CATALOG_LEN]>> =
        const { RefCell::new(None) };
    static STABLE_NODE_KEY_STAMP_COUNT: RefCell<usize> = const { RefCell::new(0) };
}

pub fn transform_pass_sort_ordinal(kind: TransformPassKind) -> u8 {
    TRANSFORM_PASS_SORT_ORDINAL_OVERRIDES.with(|overrides| {
        overrides
            .borrow()
            .as_ref()
            .map(|values| values[(kind.ordinal() - 1) as usize])
            .unwrap_or_else(|| kind.ordinal())
    })
}

#[doc(hidden)]
pub fn with_transform_pass_sort_ordinal_overrides_for_test<R>(
    overrides: [u8; TRANSFORM_PASS_CATALOG_LEN],
    run: impl FnOnce() -> R,
) -> R {
    struct ResetOrdinalOverrides(Option<[u8; TRANSFORM_PASS_CATALOG_LEN]>);

    impl Drop for ResetOrdinalOverrides {
        fn drop(&mut self) {
            let previous = self.0.take();
            TRANSFORM_PASS_SORT_ORDINAL_OVERRIDES.with(|overrides| {
                overrides.replace(previous);
            });
        }
    }

    let previous =
        TRANSFORM_PASS_SORT_ORDINAL_OVERRIDES.with(|values| values.replace(Some(overrides)));
    let _reset = ResetOrdinalOverrides(previous);
    run()
}

#[doc(hidden)]
pub fn reset_stable_node_key_stamp_count_for_test() {
    STABLE_NODE_KEY_STAMP_COUNT.with(|count| {
        *count.borrow_mut() = 0;
    });
}

#[doc(hidden)]
pub fn stable_node_key_stamp_count_for_test() -> usize {
    STABLE_NODE_KEY_STAMP_COUNT.with(|count| *count.borrow())
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
    pub family: &'static str,
    pub execution_phase: u8,
    pub executes_mutation: bool,
    pub layer: TransformLayer,
    pub read_model: TransformPassReadModel,
    pub reads_semantic_graph: bool,
    pub reads_cascade_model: bool,
    pub writes_css: bool,
    pub cascade_safety_witness: CascadeSafetyWitnessV0,
    pub cascade_obligation: &'static str,
    pub explicit_opt_in_required: bool,
    pub dialect_restriction: Option<&'static str>,
    pub spec_snapshot: Option<&'static str>,
    pub opt_in_policy: Option<&'static str>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CascadeSafetyWitnessV0 {
    pub pass_id: &'static str,
    pub obligation: &'static str,
    pub enforced_at: &'static str,
}

impl CascadeSafetyWitnessV0 {
    pub fn evidence_node_key(&self) -> EvidenceNodeKeyV0 {
        EvidenceNodeKeyV0::new(CASCADE_WITNESS_EVIDENCE_QUERY_V0, self.pass_id)
    }

    pub fn evidence_node_seed(&self) -> EvidenceNodeSeedV0 {
        EvidenceNodeSeedV0::new(
            self.evidence_node_key(),
            vec![
                ["pass:", self.pass_id].concat(),
                ["obligation:", self.obligation].concat(),
                ["enforcedAt:", self.enforced_at].concat(),
            ],
            GuaranteeKindV0::for_label_less_family(),
        )
    }

    pub fn evidence_graph(&self) -> Result<EvidenceGraphV0, EvidenceGraphBuildErrorV0> {
        build_evidence_graph_from_edges_v0(
            [self.evidence_node_seed()],
            [EvidenceDemandEdgeV0::new(
                CASCADE_WITNESS_EVIDENCE_QUERY_V0,
                self.evidence_node_key(),
                CASCADE_WITNESS_EVIDENCE_EDGE_KIND_V0,
            )],
        )
    }
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
    pub pass_descriptors: Vec<TransformPassDescriptorV0>,
    pub dag_edges: Vec<TransformDagEdgeV0>,
    pub pass_catalog_count: usize,
    pub semantic_aware_pass_count: usize,
    pub commodity_pass_count: usize,
    pub emission_pass_count: usize,
    pub structural_pass_count: usize,
    pub text_local_pass_count: usize,
    pub module_evaluation_pass_count: usize,
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize)]
#[serde(transparent)]
pub struct StableNodeKeyU64V0(pub u64);

impl StableNodeKeyU64V0 {
    pub const fn as_u64(self) -> u64 {
        self.0
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct StableNodeKeySeedV0 {
    semantic_key: String,
    ordinal: usize,
}

impl StableNodeKeySeedV0 {
    fn new(semantic_key: String, ordinal: usize) -> Self {
        Self {
            semantic_key,
            ordinal,
        }
    }

    fn materialize(&self) -> StableNodeKeyV0 {
        STABLE_NODE_KEY_STAMP_COUNT.with(|count| {
            *count.borrow_mut() += 1;
        });
        StableNodeKeyV0(format!("{}#{}", self.semantic_key, self.ordinal))
    }

    fn materialize_u64(&self) -> StableNodeKeyU64V0 {
        let mut hash = StableNodeKeyFnv64::new();
        hash.piece("omena-transform-cst.stable-node-key");
        hash.piece(&self.semantic_key);
        hash.piece("#");
        let ordinal = self.ordinal.to_string();
        hash.piece(&ordinal);
        StableNodeKeyU64V0(hash.finish())
    }
}

struct StableNodeKeyFnv64(u64);

impl StableNodeKeyFnv64 {
    const OFFSET: u64 = 0xcbf29ce484222325;
    const PRIME: u64 = 0x00000100000001b3;

    const fn new() -> Self {
        Self(Self::OFFSET)
    }

    fn piece(&mut self, value: &str) {
        for byte in value.as_bytes() {
            self.0 = (self.0 ^ u64::from(*byte)).wrapping_mul(Self::PRIME);
        }
        self.0 = (self.0 ^ 0xff).wrapping_mul(Self::PRIME);
    }

    const fn finish(self) -> u64 {
        self.0
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StableTransformIrNodeV0 {
    pub node_id: String,
    node_key: OnceLock<StableNodeKeyV0>,
    node_key_u64: OnceLock<StableNodeKeyU64V0>,
    node_key_seed: Option<StableNodeKeySeedV0>,
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
        if let Some(key) = self.node_key.get() {
            return Some(key);
        }
        let seed = self.node_key_seed.as_ref()?;
        Some(self.node_key.get_or_init(|| seed.materialize()))
    }

    pub fn additive_node_key_u64(&self) -> Option<StableNodeKeyU64V0> {
        if let Some(key) = self.node_key_u64.get() {
            return Some(*key);
        }
        let seed = self.node_key_seed.as_ref()?;
        Some(*self.node_key_u64.get_or_init(|| seed.materialize_u64()))
    }

    fn set_additive_node_key_seed(&mut self, ordinal: usize) {
        self.node_key = OnceLock::new();
        self.node_key_u64 = OnceLock::new();
        self.node_key_seed = Some(StableNodeKeySeedV0::new(self.semantic_key.clone(), ordinal));
    }

    #[doc(hidden)]
    pub fn clear_additive_node_key_for_test(&mut self) {
        self.node_key = OnceLock::new();
        self.node_key_u64 = OnceLock::new();
        self.node_key_seed = None;
    }
}

impl Serialize for StableTransformIrNodeV0 {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let node_key = self.additive_node_key();
        let node_key_u64 = self.additive_node_key_u64();
        let field_count = 8 + usize::from(node_key.is_some()) + usize::from(node_key_u64.is_some());
        let mut state = serializer.serialize_struct("StableTransformIrNodeV0", field_count)?;
        state.serialize_field("nodeId", &self.node_id)?;
        if let Some(node_key) = node_key {
            state.serialize_field("nodeKey", node_key)?;
        }
        if let Some(node_key_u64) = node_key_u64 {
            state.serialize_field("nodeKeyU64", &node_key_u64)?;
        }
        state.serialize_field("kind", &self.kind)?;
        state.serialize_field("kindId", &self.kind_id)?;
        state.serialize_field("label", &self.label)?;
        state.serialize_field("semanticKey", &self.semantic_key)?;
        state.serialize_field("sourceSpanStart", &self.source_span_start)?;
        state.serialize_field("sourceSpanEnd", &self.source_span_end)?;
        state.serialize_field("provenanceAnchorIndex", &self.provenance_anchor_index)?;
        state.end()
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

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TransformPassSpecV0 {
    pub schema_version: &'static str,
    pub product: &'static str,
    pub pass_id: &'static str,
    pub pass_kind: TransformPassKind,
    pub cascade_obligation: &'static str,
    pub cascade_safety_witness: CascadeSafetyWitnessV0,
}

impl TransformPassSpecV0 {
    pub fn from_pass(pass_kind: TransformPassKind) -> Self {
        let cascade_safety_witness = cascade_safety_witness(pass_kind);
        Self {
            schema_version: "0",
            product: "omena-transform-cst.pass-spec",
            pass_id: pass_kind.id(),
            pass_kind,
            cascade_obligation: cascade_safety_witness.obligation,
            cascade_safety_witness,
        }
    }

    pub fn declares_cascade_obligation(&self) -> bool {
        !self.cascade_obligation.is_empty()
            && self.cascade_safety_witness.pass_id == self.pass_id
            && self.cascade_safety_witness.obligation == self.cascade_obligation
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RewriteCandidateV0 {
    schema_version: &'static str,
    product: &'static str,
    pass_spec: TransformPassSpecV0,
    semantic_signature: String,
    input_source_byte_len: usize,
    output_source_byte_len: usize,
    input_stable_ir: StableTransformIrV0,
    output_stable_ir: StableTransformIrV0,
}

impl RewriteCandidateV0 {
    pub fn from_sources(
        pass_kind: TransformPassKind,
        input_source: &str,
        output_source: &str,
        dialect: StyleDialect,
        semantic_signature: impl Into<String>,
    ) -> Self {
        let semantic_signature = semantic_signature.into();
        Self {
            schema_version: "0",
            product: "omena-transform-cst.rewrite-candidate",
            pass_spec: TransformPassSpecV0::from_pass(pass_kind),
            semantic_signature: semantic_signature.clone(),
            input_source_byte_len: input_source.len(),
            output_source_byte_len: output_source.len(),
            input_stable_ir: build_stable_transform_ir_from_source(
                input_source,
                dialect,
                semantic_signature.clone(),
            ),
            output_stable_ir: build_stable_transform_ir_from_source(
                output_source,
                dialect,
                semantic_signature,
            ),
        }
    }

    pub fn pass_spec(&self) -> &TransformPassSpecV0 {
        &self.pass_spec
    }

    pub fn output_stable_ir(&self) -> &StableTransformIrV0 {
        &self.output_stable_ir
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct VerificationReportV0 {
    schema_version: &'static str,
    product: &'static str,
    pass_id: &'static str,
    closed_world_bundle_hash: Option<String>,
    cascade_obligation_declared: bool,
    provenance_recomputed: bool,
    provenance_preserved: bool,
    contains_bogus_or_trivia: bool,
    stable_post_semantic_ir: bool,
    cascade_proof: CascadeSMTProofV0,
}

impl VerificationReportV0 {
    pub fn provenance_preserved(&self) -> bool {
        self.provenance_preserved
    }

    pub fn contains_bogus_or_trivia(&self) -> bool {
        self.contains_bogus_or_trivia
    }

    pub fn cascade_proof(&self) -> &CascadeSMTProofV0 {
        &self.cascade_proof
    }

    pub fn closed_world_bundle_hash(&self) -> Option<&str> {
        self.closed_world_bundle_hash.as_deref()
    }

    pub fn cascade_safe(&self) -> bool {
        self.cascade_proof.verdict == SmtVerdictV0::Accepted
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct VerifiedRewriteV0 {
    schema_version: &'static str,
    product: &'static str,
    candidate: RewriteCandidateV0,
    verification_report: VerificationReportV0,
}

impl VerifiedRewriteV0 {
    pub fn candidate(&self) -> &RewriteCandidateV0 {
        &self.candidate
    }

    pub fn verification_report(&self) -> &VerificationReportV0 {
        &self.verification_report
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum TransformVerificationErrorV0 {
    CascadeProofRejected {
        pass_id: &'static str,
        verdict: SmtVerdictV0,
    },
    CascadeObligationMissing {
        pass_id: &'static str,
    },
    ProvenanceNotPreserved {
        pass_id: &'static str,
    },
    ClosedWorldBundleRequired {
        pass_id: &'static str,
    },
}

pub fn summarize_omena_transform_cst_boundary() -> TransformCstBoundarySummaryV0 {
    let pass_contracts = default_transform_pass_contracts();
    let pass_descriptors = default_transform_pass_descriptors();
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
    let structural_pass_count = pass_descriptors
        .iter()
        .filter(|descriptor| descriptor.pass_class == TransformPassClassV0::Structural)
        .count();
    let text_local_pass_count = pass_descriptors
        .iter()
        .filter(|descriptor| descriptor.pass_class == TransformPassClassV0::TextLocal)
        .count();
    let module_evaluation_pass_count = pass_descriptors
        .iter()
        .filter(|descriptor| descriptor.pass_class == TransformPassClassV0::ModuleEvaluation)
        .count();
    let all_passes_declare_cascade_obligation = pass_contracts
        .iter()
        .all(|contract| !contract.cascade_obligation.is_empty());
    let all_passes_have_compile_time_cascade_witness = pass_contracts.iter().all(|contract| {
        contract.cascade_safety_witness.pass_id == contract.id
            && contract.cascade_safety_witness.obligation == contract.cascade_obligation
            && contract.cascade_safety_witness.enforced_at == "compile-time-exhaustive-pass-catalog"
    });
    let pass_catalog_count = pass_contracts.len();

    TransformCstBoundarySummaryV0 {
        schema_version: "0",
        product: "omena-transform-cst.boundary",
        representation: "post-semantic-provenance-preserving-transform-cst",
        pass_contracts,
        pass_descriptors,
        dag_edges: default_transform_dag_edges(),
        pass_catalog_count,
        semantic_aware_pass_count,
        commodity_pass_count,
        emission_pass_count,
        structural_pass_count,
        text_local_pass_count,
        module_evaluation_pass_count,
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
    let verified_rewrites =
        verify_rewrite_plan_with_backend(source, dialect, semantic_signature.clone(), passes);
    let provenance_preserved = verified_rewrites.as_ref().is_ok_and(|rewrites| {
        rewrites
            .iter()
            .all(|rewrite| rewrite.verification_report.provenance_preserved)
    });

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
        provenance_preserved,
    }
}

pub fn build_verified_transform_cst_artifact_with_dialect(
    source: &str,
    dialect: StyleDialect,
    semantic_signature: impl Into<String>,
    passes: &[TransformPassKind],
) -> Result<TransformCstArtifactV0, TransformVerificationErrorV0> {
    let semantic_signature = semantic_signature.into();
    let verified_rewrites =
        verify_rewrite_plan_with_backend(source, dialect, semantic_signature.clone(), passes)?;
    let stable_ir =
        build_stable_transform_ir_from_source(source, dialect, semantic_signature.clone());
    Ok(transform_cst_artifact_from_verified_plan(
        source.len(),
        semantic_signature,
        stable_ir,
        passes,
        &verified_rewrites,
    ))
}

pub fn verify_rewrite_candidate_with_backend<B: SmtBackendV0>(
    candidate: RewriteCandidateV0,
    backend: &B,
) -> Result<VerifiedRewriteV0, TransformVerificationErrorV0> {
    verify_rewrite_candidate_inner(candidate, backend, None)
}

pub fn verify_rewrite_candidate_with_backend_and_closed_world_bundle<B: SmtBackendV0>(
    candidate: RewriteCandidateV0,
    backend: &B,
    closed_world_bundle: &ClosedWorldBundleV0,
) -> Result<VerifiedRewriteV0, TransformVerificationErrorV0> {
    verify_rewrite_candidate_inner(candidate, backend, Some(closed_world_bundle))
}

fn verify_rewrite_candidate_inner<B: SmtBackendV0>(
    candidate: RewriteCandidateV0,
    backend: &B,
    closed_world_bundle: Option<&ClosedWorldBundleV0>,
) -> Result<VerifiedRewriteV0, TransformVerificationErrorV0> {
    let pass_id = candidate.pass_spec.pass_id;
    if closed_world_bundle.is_none()
        && transform_pass_requires_closed_world_bundle(candidate.pass_spec.pass_kind)
    {
        return Err(TransformVerificationErrorV0::ClosedWorldBundleRequired { pass_id });
    }
    let obligation_family = obligation_family_for_transform_pass(candidate.pass_spec.pass_kind);
    let cascade_obligation_declared = obligation_family.declares_cascade_obligation();
    let provenance_recomputed = candidate_recomputes_provenance(&candidate);
    let contains_bogus_or_trivia = candidate.input_stable_ir.contains_bogus_or_trivia
        || candidate.output_stable_ir.contains_bogus_or_trivia;
    let stable_post_semantic_ir = candidate.input_stable_ir.stable_post_semantic_ir
        && candidate.output_stable_ir.stable_post_semantic_ir;
    let provenance_preserved =
        provenance_recomputed && stable_post_semantic_ir && !contains_bogus_or_trivia;
    let proof_input = TransformRewriteProofInputV0::new(
        pass_id,
        obligation_family,
        provenance_recomputed,
        provenance_preserved,
        contains_bogus_or_trivia,
        stable_post_semantic_ir,
    );
    let cascade_proof = smt_verify_transform_rewrite_candidate_v0(&proof_input, backend);
    let verdict = cascade_proof.verdict;
    let verification_report = VerificationReportV0 {
        schema_version: "0",
        product: "omena-transform-cst.verification-report",
        pass_id,
        closed_world_bundle_hash: closed_world_bundle
            .map(|bundle| bundle.closure_hash().to_string()),
        cascade_obligation_declared,
        provenance_recomputed,
        provenance_preserved,
        contains_bogus_or_trivia,
        stable_post_semantic_ir,
        cascade_proof,
    };

    if verdict != SmtVerdictV0::Accepted {
        return Err(TransformVerificationErrorV0::CascadeProofRejected { pass_id, verdict });
    }
    if !cascade_obligation_declared {
        return Err(TransformVerificationErrorV0::CascadeObligationMissing { pass_id });
    }
    if !provenance_preserved {
        return Err(TransformVerificationErrorV0::ProvenanceNotPreserved { pass_id });
    }

    Ok(VerifiedRewriteV0 {
        schema_version: "0",
        product: "omena-transform-cst.verified-rewrite",
        candidate,
        verification_report,
    })
}

pub fn verify_rewrite_candidate(
    candidate: RewriteCandidateV0,
) -> Result<VerifiedRewriteV0, TransformVerificationErrorV0> {
    verify_rewrite_candidate_with_backend(candidate, &StubSmtBackendV0::default())
}

pub fn verify_rewrite_candidate_with_closed_world_bundle(
    candidate: RewriteCandidateV0,
    closed_world_bundle: &ClosedWorldBundleV0,
) -> Result<VerifiedRewriteV0, TransformVerificationErrorV0> {
    verify_rewrite_candidate_with_backend_and_closed_world_bundle(
        candidate,
        &StubSmtBackendV0::default(),
        closed_world_bundle,
    )
}

pub fn apply_verified_rewrite(verified_rewrite: &VerifiedRewriteV0) -> TransformCstArtifactV0 {
    let candidate = verified_rewrite.candidate();
    transform_cst_artifact_from_verified_plan(
        candidate.output_source_byte_len,
        candidate.semantic_signature.clone(),
        candidate.output_stable_ir.clone(),
        &[candidate.pass_spec.pass_kind],
        std::slice::from_ref(verified_rewrite),
    )
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
        node.set_additive_node_key_seed(*ordinal);
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
    let contains_bogus_or_trivia = parser_error_count > 0;

    StableTransformIrV0 {
        schema_version: "0",
        product: "omena-transform-cst.stable-ir",
        dialect: transform_cst_style_dialect_label(dialect),
        source_byte_len: source.len(),
        semantic_signature: semantic_signature.into(),
        node_count,
        parser_error_count,
        contains_bogus_or_trivia,
        stable_post_semantic_ir: parser_error_count == 0,
        nodes,
        provenance_anchors,
    }
}

fn verify_rewrite_plan_with_backend(
    source: &str,
    dialect: StyleDialect,
    semantic_signature: String,
    passes: &[TransformPassKind],
) -> Result<Vec<VerifiedRewriteV0>, TransformVerificationErrorV0> {
    let backend = StubSmtBackendV0::default();
    passes
        .iter()
        .map(|pass| {
            verify_rewrite_candidate_with_backend(
                RewriteCandidateV0::from_sources(
                    *pass,
                    source,
                    source,
                    dialect,
                    semantic_signature.clone(),
                ),
                &backend,
            )
        })
        .collect()
}

fn transform_cst_artifact_from_verified_plan(
    source_byte_len: usize,
    semantic_signature: String,
    stable_ir: StableTransformIrV0,
    passes: &[TransformPassKind],
    verified_rewrites: &[VerifiedRewriteV0],
) -> TransformCstArtifactV0 {
    let stable_ir_node_count = stable_ir.node_count;
    let parser_error_count = stable_ir.parser_error_count;
    let contains_bogus_or_trivia = verified_rewrites
        .iter()
        .any(|rewrite| rewrite.verification_report.contains_bogus_or_trivia());
    let provenance_preserved = verified_rewrites
        .iter()
        .all(|rewrite| rewrite.verification_report.provenance_preserved());

    TransformCstArtifactV0 {
        schema_version: "0",
        product: "omena-transform-cst.artifact",
        source_byte_len,
        semantic_signature,
        stable_ir,
        stable_ir_node_count,
        parser_error_count,
        contains_bogus_or_trivia,
        pass_ids: passes.iter().map(|pass| pass.id()).collect(),
        provenance_preserved,
    }
}

fn candidate_recomputes_provenance(candidate: &RewriteCandidateV0) -> bool {
    stable_ir_has_consistent_provenance(&candidate.input_stable_ir)
        && stable_ir_has_consistent_provenance(&candidate.output_stable_ir)
}

fn transform_pass_requires_closed_world_bundle(kind: TransformPassKind) -> bool {
    matches!(
        kind,
        TransformPassKind::LayerFlatten
            | TransformPassKind::TreeShakeClass
            | TransformPassKind::TreeShakeKeyframes
            | TransformPassKind::TreeShakeValue
            | TransformPassKind::TreeShakeCustomProperty
    )
}

fn stable_ir_has_consistent_provenance(ir: &StableTransformIrV0) -> bool {
    ir.node_count == ir.nodes.len()
        && ir.node_count == ir.provenance_anchors.len()
        && ir.nodes.iter().enumerate().all(|(index, node)| {
            let Some(anchor) = ir.provenance_anchors.get(index) else {
                return false;
            };
            node.provenance_anchor_index == index
                && anchor.anchor_index == index
                && anchor.node_id == node.node_id
                && anchor.semantic_key == node.semantic_key
                && anchor.source_span_start == node.source_span_start
                && anchor.source_span_end == node.source_span_end
        })
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
        family: transform_pass_family(kind),
        execution_phase: transform_pass_execution_phase(kind),
        executes_mutation: transform_pass_executes_mutation(kind),
        layer: kind.layer(),
        read_model: kind.read_model(),
        reads_semantic_graph: kind.reads_semantic_graph(),
        reads_cascade_model: kind.reads_cascade_model(),
        writes_css: true,
        cascade_safety_witness,
        cascade_obligation: cascade_safety_witness.obligation,
        explicit_opt_in_required: kind.explicit_opt_in_required(),
        dialect_restriction: kind.dialect_restriction(),
        spec_snapshot: kind.spec_snapshot(),
        opt_in_policy: kind.opt_in_policy(),
    }
}

const fn transform_pass_family(kind: TransformPassKind) -> &'static str {
    match kind.ordinal() {
        1..=7 => "commodity-token",
        8 | 26 => "egg-backed",
        9..=13 => "cascade-proven-structural",
        14..=25 | 42..=44 => "target-lowering",
        27..=29 => "module-bundle",
        30..=33 => "css-modules-resolution",
        34..=40 => "semantic-reachability",
        41 => "emission",
        _ => "unknown",
    }
}

const fn transform_pass_execution_phase(kind: TransformPassKind) -> u8 {
    match kind.ordinal() {
        27..=29 => 10,
        30..=40 => 20,
        14..=25 | 42..=44 => 30,
        8..=13 | 26 => 40,
        1..=7 => 50,
        41 => 60,
        _ => 70,
    }
}

const fn transform_pass_executes_mutation(_kind: TransformPassKind) -> bool {
    true
}

pub const fn cascade_safety_witness(kind: TransformPassKind) -> CascadeSafetyWitnessV0 {
    CascadeSafetyWitnessV0 {
        pass_id: kind.id(),
        obligation: cascade_safe_obligation(kind),
        enforced_at: "compile-time-exhaustive-pass-catalog",
    }
}

pub const fn obligation_family_for_transform_pass(kind: TransformPassKind) -> ObligationFamilyIdV0 {
    match kind {
        TransformPassKind::WhitespaceStrip => ObligationFamilyIdV0::WhitespaceBoundary,
        TransformPassKind::CommentStrip => ObligationFamilyIdV0::CommentSourceMapProvenance,
        TransformPassKind::NumberCompression => ObligationFamilyIdV0::NumericLiteralEquivalence,
        TransformPassKind::UnitNormalization => ObligationFamilyIdV0::DimensionComputedValue,
        TransformPassKind::ColorCompression => ObligationFamilyIdV0::ColorLiteralEquivalence,
        TransformPassKind::UrlQuoteStrip => ObligationFamilyIdV0::UrlTokenGrammar,
        TransformPassKind::StringQuoteNormalize => ObligationFamilyIdV0::StringTextAndFontValue,
        TransformPassKind::SelectorIsWhereCompression => {
            ObligationFamilyIdV0::SelectorSpecificityAndCascade
        }
        TransformPassKind::ShorthandCombining => {
            ObligationFamilyIdV0::LonghandShorthandCascadeOutcome
        }
        TransformPassKind::RuleDeduplication => ObligationFamilyIdV0::DeclarationCascadeOrder,
        TransformPassKind::RuleMerging => ObligationFamilyIdV0::RuleMergeWinnerOrder,
        TransformPassKind::SelectorMerging => {
            ObligationFamilyIdV0::SelectorIdentityAndModuleSemantics
        }
        TransformPassKind::EmptyRuleRemoval => ObligationFamilyIdV0::SemanticMarkerRetention,
        TransformPassKind::VendorPrefixing => ObligationFamilyIdV0::TargetPrefixAddition,
        TransformPassKind::StalePrefixRemoval => ObligationFamilyIdV0::StalePrefixRemovalMapping,
        TransformPassKind::LightDarkLowering => ObligationFamilyIdV0::TargetFallbackBranch,
        TransformPassKind::ColorMixLowering => ObligationFamilyIdV0::ColorSpaceTargetEquivalence,
        TransformPassKind::OklchOklabLowering
        | TransformPassKind::ColorFunctionLowering
        | TransformPassKind::RelativeColorLowering => ObligationFamilyIdV0::TargetColorPrecision,
        TransformPassKind::LogicalToPhysical => ObligationFamilyIdV0::DirectionalityOption,
        TransformPassKind::NestingUnwrap => ObligationFamilyIdV0::NestedSelectorSpecificity,
        TransformPassKind::ScopeFlatten => ObligationFamilyIdV0::ScopedMatching,
        TransformPassKind::LayerFlatten => ObligationFamilyIdV0::LayerOrderComparison,
        TransformPassKind::SupportsStaticEval => ObligationFamilyIdV0::TargetFeaturePredicate,
        TransformPassKind::MediaStaticEval => ObligationFamilyIdV0::MediaPredicate,
        TransformPassKind::ContainerStaticEval => ObligationFamilyIdV0::ContainerPredicate,
        TransformPassKind::NativeCssStaticEval => ObligationFamilyIdV0::NativeCssStaticValue,
        TransformPassKind::CalcReduction => ObligationFamilyIdV0::CalcExpressionEquivalence,
        TransformPassKind::ImportInline => ObligationFamilyIdV0::ImportWrapperProvenance,
        TransformPassKind::ScssModuleEvaluate => ObligationFamilyIdV0::ScssNamespaceProvenance,
        TransformPassKind::LessModuleEvaluate => ObligationFamilyIdV0::LessNamespaceProvenance,
        TransformPassKind::HashCssModuleClassNames => ObligationFamilyIdV0::SelectorIdentityMap,
        TransformPassKind::ResolveCssModulesComposes => {
            ObligationFamilyIdV0::ComposedClassProvenance
        }
        TransformPassKind::ValueResolution => ObligationFamilyIdV0::ValueGraphResolution,
        TransformPassKind::StaticVarSubstitution => ObligationFamilyIdV0::CustomPropertyFixedPoint,
        TransformPassKind::TreeShakeClass => ObligationFamilyIdV0::SourceClassReachability,
        TransformPassKind::TreeShakeKeyframes => ObligationFamilyIdV0::AnimationNameReachability,
        TransformPassKind::TreeShakeValue => ObligationFamilyIdV0::ValueGraphReachability,
        TransformPassKind::TreeShakeCustomProperty => ObligationFamilyIdV0::VarReachability,
        TransformPassKind::DeadMediaBranchRemoval => ObligationFamilyIdV0::DeadMediaWitness,
        TransformPassKind::DeadSupportsBranchRemoval => ObligationFamilyIdV0::DeadSupportsWitness,
        TransformPassKind::DesignTokenRouting => ObligationFamilyIdV0::DesignTokenPackageProvenance,
        TransformPassKind::PrintCss => ObligationFamilyIdV0::SourceMapTransformTrace,
    }
}

pub const fn cascade_safe_obligation(kind: TransformPassKind) -> &'static str {
    obligation_family_for_transform_pass(kind)
        .descriptor()
        .obligation
}

#[cfg(test)]
fn cascade_safe_obligation_reference(kind: TransformPassKind) -> &'static str {
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
        TransformPassKind::NativeCssStaticEval => {
            "may fold native CSS if() and function calls only when the evaluator proves a concrete static value and preserves runtime-dependent constructs verbatim"
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
        node_key: OnceLock::new(),
        node_key_u64: OnceLock::new(),
        node_key_seed: None,
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
            from: "value-resolution",
            to: "native-css-static-eval",
            reason: "@value references inside native CSS conditional values and function arguments must resolve before static native evaluation",
        },
        TransformDagEdgeV0 {
            from: "custom-property-static-resolve",
            to: "native-css-static-eval",
            reason: "var() references inside native CSS conditional values and function arguments must resolve before static native evaluation",
        },
        TransformDagEdgeV0 {
            from: "native-css-static-eval",
            to: "calc-reduction",
            reason: "native CSS static evaluation can expose calc() values that should reduce after folding",
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
            from: "native-css-static-eval",
            to: "vendor-prefixing",
            reason: "prefixing runs after native CSS static evaluation produces final declarations",
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
        NATIVE_CSS_STATIC_EVAL_DIALECT_RESTRICTION_V0, NATIVE_CSS_STATIC_EVAL_OPT_IN_POLICY_V0,
        NATIVE_CSS_STATIC_EVAL_SPEC_SNAPSHOT_V0, RewriteCandidateV0,
        STABLE_TRANSFORM_IR_NODE_IDENTITY_POLICY_V0, StableTransformIrNodeKindV0, StyleDialect,
        TRANSFORM_PASS_CATALOG_LEN, TransformLayer, TransformPassClassV0, TransformPassKind,
        TransformVerificationErrorV0, all_transform_pass_kinds, apply_verified_rewrite,
        build_stable_transform_ir_from_source, build_transform_cst_artifact,
        build_verified_transform_cst_artifact_with_dialect, cascade_safe_obligation,
        cascade_safe_obligation_reference, cascade_safety_witness,
        default_transform_pass_descriptors, obligation_family_for_transform_pass,
        summarize_omena_transform_cst_boundary, transform_build_profile_from_passes,
        verify_rewrite_candidate, verify_rewrite_candidate_with_backend,
        verify_rewrite_candidate_with_closed_world_bundle,
    };
    use omena_cascade_proof::{
        CanonicalSmtInputV0, SMT_FEATURE_GATE_V0, SMT_LAYER_MARKER_V0, SMT_SCHEMA_VERSION_V0,
        SmtBackendCheckV0, SmtBackendKindV0, SmtBackendSatResultV0, SmtBackendV0, SmtVerdictV0,
    };
    use omena_evidence_graph::GuaranteeKindV0;
    use omena_parser::{
        ClosedWorldBundleV0, ClosedWorldLinkedModuleV0, ConfigurationHashV0, ModuleIdV0,
        ModuleInstanceKeyV0,
    };

    struct RejectingBackend;

    impl SmtBackendV0 for RejectingBackend {
        fn backend_kind(&self) -> SmtBackendKindV0 {
            SmtBackendKindV0::Stub
        }

        fn check_canonical_input_v0(&self, input: &CanonicalSmtInputV0) -> SmtBackendCheckV0 {
            SmtBackendCheckV0 {
                schema_version: SMT_SCHEMA_VERSION_V0,
                product: "omena-smt.backend-check",
                layer_marker: SMT_LAYER_MARKER_V0,
                feature_gate: SMT_FEATURE_GATE_V0,
                backend: self.backend_kind(),
                obligation_id: input.obligation_id.clone(),
                formula_count: input.canonical_terms.len(),
                sat_result: SmtBackendSatResultV0::Unsat,
                model_available: false,
            }
        }
    }

    #[test]
    fn exposes_transform_cst_boundary_with_full_pass_catalog() {
        let boundary = summarize_omena_transform_cst_boundary();

        assert_eq!(boundary.schema_version, "0");
        assert_eq!(boundary.product, "omena-transform-cst.boundary");
        assert_eq!(boundary.pass_catalog_count, TRANSFORM_PASS_CATALOG_LEN);
        assert!(boundary.full_pass_catalog_covered);
        assert_eq!(boundary.semantic_aware_pass_count, 14);
        assert_eq!(boundary.commodity_pass_count, 29);
        assert_eq!(boundary.emission_pass_count, 1);
        assert_eq!(boundary.pass_descriptors.len(), TRANSFORM_PASS_CATALOG_LEN);
        assert_eq!(boundary.structural_pass_count, 21);
        assert_eq!(boundary.text_local_pass_count, 20);
        assert_eq!(boundary.module_evaluation_pass_count, 2);
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
                && !contract.cascade_obligation.is_empty()
                && contract.cascade_safety_witness.pass_id == "tree-shake-class"
                && contract.cascade_safety_witness.obligation == contract.cascade_obligation
                && contract.cascade_safety_witness.enforced_at
                    == "compile-time-exhaustive-pass-catalog"
        }));
        assert!(boundary.pass_contracts.iter().any(|contract| {
            contract.kind == TransformPassKind::NativeCssStaticEval
                && contract.label == "native-css-static-eval"
                && contract.layer == TransformLayer::Commodity
                && contract.read_model == super::TransformPassReadModel::TargetData
                && contract.cascade_safety_witness.pass_id == "native-css-static-eval"
                && contract.explicit_opt_in_required
                && contract.dialect_restriction
                    == Some(NATIVE_CSS_STATIC_EVAL_DIALECT_RESTRICTION_V0)
                && contract.spec_snapshot == Some(NATIVE_CSS_STATIC_EVAL_SPEC_SNAPSHOT_V0)
                && contract.opt_in_policy == Some(NATIVE_CSS_STATIC_EVAL_OPT_IN_POLICY_V0)
        }));
        assert!(boundary.dag_edges.iter().any(|edge| {
            edge.from == "composes-resolution" && edge.to == "css-modules-class-hashing"
        }));
        assert!(boundary.pass_descriptors.iter().any(|descriptor| {
            descriptor.kind == TransformPassKind::NestingUnwrap
                && descriptor.pass_class == TransformPassClassV0::Structural
        }));
        assert!(boundary.pass_descriptors.iter().any(|descriptor| {
            descriptor.kind == TransformPassKind::StaticVarSubstitution
                && descriptor.pass_class == TransformPassClassV0::TextLocal
        }));
        assert!(boundary.pass_descriptors.iter().any(|descriptor| {
            descriptor.kind == TransformPassKind::ScssModuleEvaluate
                && descriptor.pass_class == TransformPassClassV0::ModuleEvaluation
        }));
    }

    #[test]
    fn pass_descriptors_pin_classification_phase_and_dependency_contracts() -> Result<(), String> {
        let descriptors = default_transform_pass_descriptors();

        assert_eq!(descriptors.len(), TRANSFORM_PASS_CATALOG_LEN);
        assert!(
            descriptors
                .iter()
                .all(|descriptor| descriptor.schema_version == "0"
                    && descriptor.product == "omena-transform-cst.pass-descriptor"
                    && descriptor.id == descriptor.kind.id())
        );
        assert_eq!(
            descriptors
                .iter()
                .filter(|descriptor| descriptor.pass_class == TransformPassClassV0::Structural)
                .count(),
            21
        );
        assert_eq!(
            descriptors
                .iter()
                .filter(|descriptor| descriptor.pass_class == TransformPassClassV0::TextLocal)
                .count(),
            20
        );
        assert_eq!(
            descriptors
                .iter()
                .filter(|descriptor| descriptor.pass_class == TransformPassClassV0::ModuleEvaluation)
                .count(),
            2
        );
        assert_eq!(
            descriptors
                .iter()
                .filter(|descriptor| descriptor.pass_class == TransformPassClassV0::Emission)
                .count(),
            1
        );

        let hash_descriptor = descriptors
            .iter()
            .find(|descriptor| descriptor.kind == TransformPassKind::HashCssModuleClassNames)
            .ok_or_else(|| "missing hash descriptor".to_string())?;
        assert_eq!(hash_descriptor.pass_class, TransformPassClassV0::Structural);
        assert!(
            hash_descriptor.depends_on.contains(&"composes-resolution")
                && hash_descriptor.depends_on.contains(&"nesting-unwrap")
                && hash_descriptor.depends_on.contains(&"tree-shake-class")
        );

        let profile = transform_build_profile_from_passes(
            "requested-transform-plan",
            &[
                TransformPassKind::CommentStrip,
                TransformPassKind::WhitespaceStrip,
            ],
        );
        assert_eq!(profile.schema_version, "0");
        assert_eq!(profile.profile_id, "requested-transform-plan");
        assert_eq!(profile.pass_ids, vec!["comment-strip", "whitespace-strip"]);
        assert_ne!(profile.pass_ids.len(), TRANSFORM_PASS_CATALOG_LEN);
        Ok(())
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
    fn verified_rewrite_requires_accepted_cascade_proof() -> Result<(), String> {
        let candidate = RewriteCandidateV0::from_sources(
            TransformPassKind::RuleDeduplication,
            ".button { color: red; }",
            ".button { color: red; }",
            StyleDialect::Css,
            "semantic:button",
        );
        let err = match verify_rewrite_candidate_with_backend(candidate, &RejectingBackend) {
            Ok(_) => {
                return Err(
                    "rejecting backend must prevent verified rewrite construction".to_string(),
                );
            }
            Err(err) => err,
        };

        assert_eq!(
            err,
            TransformVerificationErrorV0::CascadeProofRejected {
                pass_id: "rule-deduplication",
                verdict: SmtVerdictV0::Rejected,
            }
        );
        Ok(())
    }

    #[test]
    fn verified_rewrite_token_is_the_artifact_apply_input() -> Result<(), String> {
        let candidate = RewriteCandidateV0::from_sources(
            TransformPassKind::ColorCompression,
            ".button { color: #ffffff; }",
            ".button { color: #ffffff; }",
            StyleDialect::Css,
            "semantic:button",
        );
        let verified = match verify_rewrite_candidate(candidate) {
            Ok(verified) => verified,
            Err(err) => {
                return Err(format!(
                    "default proof backend should accept recomputed stable IR: {err:?}"
                ));
            }
        };
        let artifact = apply_verified_rewrite(&verified);

        assert!(verified.verification_report().provenance_preserved());
        assert!(verified.verification_report().cascade_safe());
        assert_eq!(
            verified.verification_report().cascade_proof().verdict,
            SmtVerdictV0::Accepted
        );
        assert_eq!(artifact.pass_ids, vec!["color-compression"]);
        assert_eq!(artifact.source_byte_len, 27);
        assert!(artifact.provenance_preserved);
        assert!(!artifact.contains_bogus_or_trivia);
        Ok(())
    }

    #[test]
    fn verified_rewrite_requires_closed_world_bundle_for_reachability_pass() -> Result<(), String> {
        let candidate = RewriteCandidateV0::from_sources(
            TransformPassKind::TreeShakeClass,
            ".used { color: red; }",
            ".used { color: red; }",
            StyleDialect::Css,
            "semantic:used",
        );
        let err = verify_rewrite_candidate(candidate.clone());
        assert_eq!(
            err,
            Err(TransformVerificationErrorV0::ClosedWorldBundleRequired {
                pass_id: "tree-shake-class"
            })
        );

        let instance = ModuleInstanceKeyV0::new(
            ModuleIdV0::new("verified-rewrite.css"),
            ConfigurationHashV0::none(),
        );
        let bundle = ClosedWorldBundleV0::try_from_linked_modules(
            vec![instance.clone()],
            vec![ClosedWorldLinkedModuleV0::new(instance).with_class_name("used")],
        )
        .map_err(|err| format!("closed-world bundle should be constructible: {err:?}"))?;
        let verified = verify_rewrite_candidate_with_closed_world_bundle(candidate, &bundle)
            .map_err(|err| format!("bundle-backed rewrite should verify: {err:?}"))?;

        assert_eq!(
            verified.verification_report().closed_world_bundle_hash(),
            Some(bundle.closure_hash())
        );

        let open_candidate = RewriteCandidateV0::from_sources(
            TransformPassKind::ColorCompression,
            ".button { color: #ffffff; }",
            ".button { color: #fff; }",
            StyleDialect::Css,
            "semantic:button",
        );
        let open_verified = verify_rewrite_candidate(open_candidate)
            .map_err(|err| format!("open rewrite should stay bundle-free: {err:?}"))?;
        assert_eq!(
            open_verified
                .verification_report()
                .closed_world_bundle_hash(),
            None
        );
        Ok(())
    }

    #[test]
    fn verified_artifact_builder_routes_through_typestate_report() -> Result<(), String> {
        let artifact = match build_verified_transform_cst_artifact_with_dialect(
            ".button { color: red; }",
            StyleDialect::Css,
            "semantic:button",
            &[
                TransformPassKind::RuleDeduplication,
                TransformPassKind::ColorCompression,
            ],
        ) {
            Ok(artifact) => artifact,
            Err(err) => {
                return Err(format!(
                    "valid stable IR should produce verified artifact: {err:?}"
                ));
            }
        };

        assert_eq!(
            artifact.pass_ids,
            vec!["rule-deduplication", "color-compression"]
        );
        assert!(artifact.provenance_preserved);
        Ok(())
    }

    #[test]
    fn transform_cst_boolean_fields_are_not_direct_literal_assignments() -> Result<(), String> {
        let source = match std::fs::read_to_string(
            std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
                .join("src")
                .join("lib.rs"),
        ) {
            Ok(source) => source,
            Err(err) => return Err(format!("test source should be readable: {err:?}")),
        };
        for forbidden in [
            ["cascade_safe", ": true"].concat(),
            ["provenance_preserved", ": true"].concat(),
            ["contains_bogus_or_trivia", ": false"].concat(),
        ] {
            assert!(
                !source.contains(&forbidden),
                "{forbidden} must be derived through verification instead of assigned directly"
            );
        }
        assert!(
            !source.contains(&["pub ", "cascade_safe", ": bool"].concat()),
            "pass contracts must not expose static cascade safety as a catalog field"
        );
        Ok(())
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
    fn stable_transform_ir_lazily_materializes_source_order_node_keys() {
        super::reset_stable_node_key_stamp_count_for_test();
        let ir = build_stable_transform_ir_from_source(
            ".button { color: red; }\n.button { color: blue; }",
            StyleDialect::Css,
            "semantic:duplicate-button",
        );
        assert_eq!(super::stable_node_key_stamp_count_for_test(), 0);

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
            button_nodes[0].additive_node_key().map(|key| key.as_str()),
            Some("class-selector:button#0")
        );
        assert_eq!(super::stable_node_key_stamp_count_for_test(), 1);
        assert_eq!(
            button_nodes[0].additive_node_key().map(|key| key.as_str()),
            Some("class-selector:button#0")
        );
        assert_eq!(super::stable_node_key_stamp_count_for_test(), 1);
        assert_eq!(
            button_nodes[1].additive_node_key().map(|key| key.as_str()),
            Some("class-selector:button#1")
        );
        assert!(button_nodes[0].additive_node_key_u64() != button_nodes[1].additive_node_key_u64());
        assert!(ir.nodes.iter().enumerate().all(|(index, node)| {
            node.node_id == format!("ir:{index}") && node.additive_node_key().is_some()
        }));
    }

    #[test]
    fn stable_transform_ir_u64_keys_preserve_string_key_equivalence_classes() {
        let ir = build_stable_transform_ir_from_source(
            ".button { color: red; }\n.button { color: blue; }\n.card { color: red; }",
            StyleDialect::Css,
            "semantic:key-equivalence",
        );
        let keyed_nodes = ir
            .nodes
            .iter()
            .map(|node| {
                (
                    node.semantic_key.as_str(),
                    node.additive_node_key()
                        .map(|key| key.as_str())
                        .unwrap_or_default(),
                    node.additive_node_key_u64()
                        .map(|key| key.as_u64())
                        .unwrap_or_default(),
                )
            })
            .collect::<Vec<_>>();

        assert!(
            keyed_nodes
                .iter()
                .filter(|(semantic_key, _, _)| *semantic_key == "class-selector:button")
                .count()
                >= 2
        );
        for (left_index, (_, left_string, left_u64)) in keyed_nodes.iter().enumerate() {
            for (_, right_string, right_u64) in keyed_nodes.iter().skip(left_index + 1) {
                assert_eq!(left_string == right_string, left_u64 == right_u64);
            }
        }
        let button_keys = keyed_nodes
            .iter()
            .filter(|(semantic_key, _, _)| *semantic_key == "class-selector:button")
            .map(|(_, string_key, u64_key)| (*string_key, *u64_key))
            .collect::<Vec<_>>();
        assert_ne!(button_keys[0].0, button_keys[1].0);
        assert_ne!(button_keys[0].1, button_keys[1].1);
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

        ir.nodes[0].clear_additive_node_key_for_test();
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
        assert!(with_key_json.contains("\"nodeKeyU64\":"));

        ir.nodes[0].clear_additive_node_key_for_test();
        let fallback_json = serde_json::to_string(&ir.nodes[0])?;
        assert!(fallback_json.contains("\"nodeId\":\"ir:0\""));
        assert!(!fallback_json.contains("nodeKey"));
        assert!(!fallback_json.contains("nodeKeyU64"));
        assert_eq!(ir.identity_key_at(0).as_deref(), Some("ir:0"));
        Ok(())
    }

    #[test]
    fn cascade_safety_witness_evidence_graph_preserves_public_shape()
    -> Result<(), serde_json::Error> {
        let witness = cascade_safety_witness(TransformPassKind::NumberCompression);

        let before = serde_json::to_value(witness)?;
        let graph = witness
            .evidence_graph()
            .map_err(|_| serde::ser::Error::custom("witness edge must target its node"))?;
        let after = serde_json::to_value(witness)?;

        assert_eq!(before, after);
        assert_eq!(graph.nodes.len(), 1);
        assert_eq!(graph.nodes[0].key.input_identity, "number-compression");
        assert_eq!(graph.nodes[0].guarantee, GuaranteeKindV0::Floor);
        assert!(
            graph.nodes[0]
                .provenance
                .iter()
                .any(|item| item == "enforcedAt:compile-time-exhaustive-pass-catalog")
        );
        Ok(())
    }

    #[test]
    fn transform_pass_obligation_families_preserve_catalog_obligation_text() {
        for kind in all_transform_pass_kinds() {
            assert_eq!(
                cascade_safe_obligation(kind),
                cascade_safe_obligation_reference(kind),
                "obligation text changed for {}",
                kind.id()
            );
            assert_eq!(
                obligation_family_for_transform_pass(kind)
                    .descriptor()
                    .obligation,
                cascade_safe_obligation_reference(kind),
                "family descriptor text changed for {}",
                kind.id()
            );
        }
    }
}

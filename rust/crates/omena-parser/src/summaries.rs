//! Parser boundary and evidence summaries.
//!
//! These V0 records are consumed by check gates, CLIs, and higher-level crates
//! to prove parser coverage without coupling to private parser internals.

use cstree::Syntax;
use omena_interner::{
    NameKind, intern_class_name, intern_css_ident, intern_custom_property_name, intern_file_path,
    intern_keyframes_name, intern_mixin_name, intern_property_name, intern_selector_key,
};
use omena_syntax::{StyleDialect, SyntaxKind};
use serde::Serialize;
use std::collections::{BTreeMap, BTreeSet};

use crate::value_names::{CSS_COLOR_FUNCTION_NAMES, VALUES_L4_MATH_FUNCTION_NAMES};
use crate::{
    BuiltinDialectExtension, ParsedAnimationFactKind, ParsedCssModuleComposesEdgeKind,
    ParsedCssModuleComposesFactKind, ParsedCssModuleValueFactKind, ParsedIcssFactKind,
    ParsedSassModuleEdgeFactKind, ParsedSassSymbolFact, ParsedSassSymbolFactKind,
    ParsedSelectorFactKind, ParsedStyleFacts, ParsedVariableFactKind, SelectorBranch, Token,
    collect_class_selector_names_from_header, collect_style_facts,
    css_module_block_scope_marker_in_header, css_module_value_statement_end,
    declaration_colon_index, find_block_after_header, lex, matching_right_brace,
    next_non_trivia_token_index_until, parse, previous_non_trivia_token_index,
    resolve_selector_header, skip_statement, skip_trivia_tokens, split_selector_groups,
    style_wrapper_at_rule, tokenize,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParserBoundarySummary {
    pub product: &'static str,
    pub tree_model: &'static str,
    pub parser_track: &'static str,
    pub dialect_count: usize,
    pub shared_name_kind_count: usize,
    pub ready_surfaces: Vec<&'static str>,
    pub not_ready_surfaces: Vec<&'static str>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParserSemanticNameConsumptionSummaryV0 {
    pub product: &'static str,
    pub dialect: StyleDialect,
    pub semantic_name_count: usize,
    pub interned_name_count: usize,
    pub invalid_name_count: usize,
    pub class_name_count: usize,
    pub css_ident_count: usize,
    pub property_name_count: usize,
    pub selector_key_count: usize,
    pub custom_property_name_count: usize,
    pub keyframes_name_count: usize,
    pub mixin_name_count: usize,
    pub file_path_count: usize,
    pub ready_surfaces: Vec<&'static str>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParserCstEquivalenceSummaryV0 {
    pub product: &'static str,
    pub dialect: StyleDialect,
    pub root_kind: SyntaxKind,
    pub parser_node_count: usize,
    pub parser_token_count: usize,
    pub typed_wrapper_count: usize,
    pub source_text_round_trip_ready: bool,
    pub syntax_kind_round_trip_ready: bool,
    pub zero_unknown_kind_ready: bool,
    pub typed_cst_wrapper_ready: bool,
    pub ready_surfaces: Vec<&'static str>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParserPrattValueCoverageSummaryV0 {
    pub product: &'static str,
    pub infix_operator_kinds: Vec<SyntaxKind>,
    pub prefix_operator_kinds: Vec<SyntaxKind>,
    pub value_expression_node_kinds: Vec<SyntaxKind>,
    pub specialized_function_family_count: usize,
    pub css_values_l4_math_function_count: usize,
    pub css_color_function_count: usize,
    pub ready_surfaces: Vec<&'static str>,
    pub next_surfaces: Vec<&'static str>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParserRecursiveDescentCoverageSummaryV0 {
    pub product: &'static str,
    pub dialect_count: usize,
    pub entry_point_count: usize,
    pub selector_surface_count: usize,
    pub at_rule_surface_count: usize,
    pub dialect_extension_surface_count: usize,
    pub recovery_surface_count: usize,
    pub ready_surfaces: Vec<&'static str>,
    pub next_surfaces: Vec<&'static str>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ParserSemanticNameCandidateV0 {
    pub(crate) kind: NameKind,
    pub(crate) text: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OmenaParserStyleFactsSummaryV0 {
    pub schema_version: &'static str,
    pub product: &'static str,
    pub dialect: &'static str,
    pub class_selector_names: Vec<String>,
    pub id_selector_names: Vec<String>,
    pub placeholder_selector_names: Vec<String>,
    pub keyframe_names: Vec<String>,
    pub animation_reference_names: Vec<String>,
    pub css_module_value_definition_names: Vec<String>,
    pub css_module_value_reference_names: Vec<String>,
    pub css_module_value_import_sources: Vec<String>,
    pub css_module_value_import_edges: Vec<OmenaParserCssModuleValueImportEdgeFactV0>,
    pub css_module_value_definition_edges: Vec<OmenaParserCssModuleValueDefinitionEdgeFactV0>,
    pub css_module_composes_target_names: Vec<String>,
    pub css_module_composes_import_sources: Vec<String>,
    pub css_module_composes_edges: Vec<OmenaParserCssModuleComposesEdgeFactV0>,
    pub icss_export_names: Vec<String>,
    pub icss_import_local_names: Vec<String>,
    pub icss_import_remote_names: Vec<String>,
    pub icss_import_sources: Vec<String>,
    pub icss_import_edges: Vec<OmenaParserIcssImportEdgeFactV0>,
    pub icss_export_edges: Vec<OmenaParserIcssExportEdgeFactV0>,
    pub variable_names: Vec<String>,
    pub sass_symbol_declaration_names: Vec<String>,
    pub sass_symbol_reference_names: Vec<String>,
    pub sass_symbol_facts: Vec<OmenaParserSassSymbolFactV0>,
    pub sass_symbol_resolution: OmenaParserSassSymbolResolutionV0,
    pub sass_module_use_sources: Vec<String>,
    pub sass_module_forward_sources: Vec<String>,
    pub sass_module_import_sources: Vec<String>,
    pub sass_module_edges: Vec<OmenaParserSassModuleEdgeFactV0>,
    pub custom_property_names: Vec<String>,
    pub custom_property_decl_names: Vec<String>,
    pub custom_property_ref_names: Vec<String>,
    pub at_rule_names: Vec<String>,
    pub parser_error_count: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OmenaParserCssModuleValueImportEdgeFactV0 {
    pub remote_name: String,
    pub local_name: String,
    pub import_source: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OmenaParserCssModuleValueDefinitionEdgeFactV0 {
    pub definition_name: String,
    pub reference_names: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OmenaParserCssModuleComposesEdgeFactV0 {
    pub kind: &'static str,
    pub owner_selector_names: Vec<String>,
    pub target_names: Vec<String>,
    pub import_source: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OmenaParserIcssImportEdgeFactV0 {
    pub local_name: String,
    pub remote_name: String,
    pub import_source: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OmenaParserIcssExportEdgeFactV0 {
    pub export_name: String,
    pub reference_names: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OmenaParserSassSymbolFactV0 {
    pub kind: &'static str,
    pub symbol_kind: &'static str,
    pub name: String,
    pub role: &'static str,
    pub namespace: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OmenaParserSassModuleEdgeFactV0 {
    pub kind: &'static str,
    pub source: String,
    pub namespace_kind: Option<&'static str>,
    pub namespace: Option<String>,
    pub visibility_filter_kind: Option<&'static str>,
    pub visibility_filter_names: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OmenaParserSassSymbolResolutionV0 {
    pub schema_version: &'static str,
    pub product: &'static str,
    pub resolution_scope: &'static str,
    pub declaration_count: usize,
    pub reference_count: usize,
    pub resolved_reference_count: usize,
    pub unresolved_reference_count: usize,
    pub edges: Vec<OmenaParserSassSymbolResolutionEdgeV0>,
    pub capabilities: OmenaParserSassSymbolResolutionCapabilitiesV0,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OmenaParserSassSymbolResolutionEdgeV0 {
    pub symbol_kind: &'static str,
    pub name: String,
    pub namespace: Option<String>,
    pub reference_kind: &'static str,
    pub reference_role: &'static str,
    pub reference_source_order: usize,
    pub declaration_kind: Option<&'static str>,
    pub declaration_source_order: Option<usize>,
    pub status: &'static str,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OmenaParserSassSymbolResolutionCapabilitiesV0 {
    pub same_file_lexical_resolution_ready: bool,
    pub declaration_before_reference_ready: bool,
    pub unresolved_reference_reporting_ready: bool,
    pub cross_file_module_resolution_ready: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OmenaParserLexSummaryV0 {
    pub schema_version: &'static str,
    pub product: &'static str,
    pub dialect: &'static str,
    pub tokens: Vec<OmenaParserLexTokenV0>,
    pub parser_error_count: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OmenaParserLexTokenV0 {
    pub kind: String,
    pub text: String,
    pub start: usize,
    pub end: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OmenaParserParityLiteSummaryV0 {
    pub schema_version: &'static str,
    pub language: &'static str,
    pub selector_names: Vec<String>,
    pub keyframes_names: Vec<String>,
    pub value_decl_names: Vec<String>,
    pub diagnostic_count: usize,
    pub rule_count: usize,
    pub declaration_count: usize,
    pub grouped_selector_count: usize,
    pub max_nesting_depth: usize,
    pub at_rule_kind_counts: OmenaParserAtRuleKindCountsV0,
    pub declaration_kind_counts: OmenaParserDeclarationKindCountsV0,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OmenaParserAtRuleKindCountsV0 {
    pub media: usize,
    pub supports: usize,
    pub layer: usize,
    pub keyframes: usize,
    pub value: usize,
    pub at_root: usize,
    pub generic: usize,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OmenaParserDeclarationKindCountsV0 {
    pub composes: usize,
    pub animation: usize,
    pub animation_name: usize,
    pub generic: usize,
}

pub fn summarize_pratt_value_parser_coverage() -> ParserPrattValueCoverageSummaryV0 {
    ParserPrattValueCoverageSummaryV0 {
        product: "omena-parser.pratt-value-coverage",
        infix_operator_kinds: vec![
            SyntaxKind::Plus,
            SyntaxKind::Minus,
            SyntaxKind::Star,
            SyntaxKind::Slash,
            SyntaxKind::Percent,
        ],
        prefix_operator_kinds: vec![SyntaxKind::Plus, SyntaxKind::Minus],
        value_expression_node_kinds: vec![
            SyntaxKind::UnaryExpression,
            SyntaxKind::BinaryExpression,
            SyntaxKind::ParenthesizedExpression,
            SyntaxKind::FunctionCall,
            SyntaxKind::FunctionArguments,
            SyntaxKind::ValueList,
            SyntaxKind::ComponentValueList,
            SyntaxKind::SimpleBlock,
            SyntaxKind::BogusValue,
        ],
        specialized_function_family_count: 10,
        css_values_l4_math_function_count: VALUES_L4_MATH_FUNCTION_NAMES.len(),
        css_color_function_count: CSS_COLOR_FUNCTION_NAMES.len(),
        ready_surfaces: vec![
            "prattValueParserCore",
            "prefixUnaryExpressions",
            "additiveMultiplicativePrecedence",
            "parenthesizedValueExpressions",
            "functionArgumentValueLists",
            "specializedCssValueFunctionFamilies",
            "valuesL4MathFunctionArityChecks",
            "varEnvAttrFunctionHeadChecks",
            "dynamicInterpolationEscapeHatches",
            "valueBogusRecovery",
        ],
        next_surfaces: vec!["fullPropertyValueGrammarRegistry"],
    }
}

pub fn summarize_recursive_descent_parser_coverage() -> ParserRecursiveDescentCoverageSummaryV0 {
    ParserRecursiveDescentCoverageSummaryV0 {
        product: "omena-parser.recursive-descent-coverage",
        dialect_count: 4,
        entry_point_count: 10,
        selector_surface_count: 12,
        at_rule_surface_count: 19,
        dialect_extension_surface_count: 17,
        recovery_surface_count: 8,
        ready_surfaces: vec![
            "recursiveDescentParserCore",
            "stylesheetRuleDeclarationEntryPoints",
            "selectorsLevelFourCstNodes",
            "registeredAtRulePreludeParsers",
            "cssNestingRuleItems",
            "scssDialectStatements",
            "sassIndentedBlocks",
            "lessDialectStatements",
            "bogusRecoverySkeleton",
            "styleFactExtractionSurface",
        ],
        next_surfaces: vec!["completeExternalSpecMirror"],
    }
}

pub fn summarize_parser_boundary() -> ParserBoundarySummary {
    ParserBoundarySummary {
        product: "omena-parser.boundary",
        tree_model: "cstree-green-root",
        parser_track: "greenFieldNextToEngineStyleParser",
        dialect_count: 4,
        shared_name_kind_count: NameKind::ALL.len(),
        ready_surfaces: vec![
            "lexResult",
            "lexedTokenTextSurface",
            "parseResult",
            "panicFreeTokenizer",
            "cstreeGreenBuilder",
            "tokenSetRecoveryScaffold",
            "dialectExtensionScaffold",
            "recursiveDescentParserCore",
            "recursiveDescentCoverageSummary",
            "selectorCstSkeleton",
            "atRuleRegistrySkeleton",
            "prattValueExpressionSkeleton",
            "prattValueParserCore",
            "prattValueCoverageSummary",
            "attributeMatcherTokenization",
            "attributeMatcherCstNodes",
            "attributeNameValueModifierCstNodes",
            "specializedValueFunctionCstNodes",
            "caseInsensitiveFunctionRegistry",
            "caseInsensitiveAtRuleRegistry",
            "valueAtomCstNodes",
            "identifierValueCstNodes",
            "stringValueCstNodes",
            "unicodeRangeValueCstNodes",
            "functionArgumentValueLists",
            "cssModuleScopeFunctionCstNodes",
            "cssModuleGlobalSelectorFactFiltering",
            "cssModuleLocalIdSelectorFacts",
            "cssModuleValueStyleFacts",
            "cssModuleValueDeclarationReferenceFacts",
            "cssModuleComposesStyleFacts",
            "icssStyleFacts",
            "animationNameStyleFacts",
            "animationShorthandStyleFacts",
            "scssStructuredBlockAtRules",
            "scssControlPreludeValidation",
            "scssControlStyleFactExtraction",
            "scssIncludeContentBlockStyleFacts",
            "scssSassModuleEdgeStyleFacts",
            "scssSassSymbolStyleFacts",
            "scssUtilityAtRules",
            "scssVariableFlagCstNodes",
            "scssNestedPropertyCstNodes",
            "scssModulePreludeSourceValidation",
            "scssModulePreludeClauseValidation",
            "scssModuleConfigCstNodes",
            "scssModuleConfigBogusRecovery",
            "scssPlaceholderSelectorCstNodes",
            "lessMixinDeclarationCstNodes",
            "lessMixinCallCstNodes",
            "lessMixinGuardCstNodes",
            "lessExtendPseudoCstNodes",
            "lessDetachedRulesetCstNodes",
            "lessNamespaceAccessCstNodes",
            "lessPropertyVariableTokenization",
            "lessPropertyVariableCstNodes",
            "lessEscapedStringTokenization",
            "lessEscapedStringValueCstNodes",
            "importantAnnotationTokenization",
            "urlTokenization",
            "urlValueCstNodes",
            "quotedUrlFunctionValueCstNodes",
            "conditionalAtRulePreludeCstNodes",
            "supportsAtRulePreludeValidation",
            "conditionalLevel5AtRuleCstNodes",
            "mediaQueryCstNodes",
            "mediaQueryListValidation",
            "importPreludeCstNodes",
            "importSourcePreludeValidation",
            "importTailPreludeValidation",
            "customMediaPreludeValidation",
            "propertyAtRuleNameValidation",
            "namedAtRulePreludeValidation",
            "containerAtRulePreludeValidation",
            "charsetNamespaceAtRulePreludeValidation",
            "keyframesAtRuleNameValidation",
            "emptyBlockAtRulePreludeValidation",
            "layerScopePreludeCstNodes",
            "layerAtRulePreludeValidation",
            "scopeAtRulePreludeValidation",
            "pageAtRulePreludeValidation",
            "pageMarginAtRuleCstNodes",
            "modernDeclarationAtRuleCstNodes",
            "fontFeatureValuesAtRuleCstNodes",
            "fontFeatureValuesPreludeValidation",
            "keyframeSelectorListValidation",
            "viewTransitionAtRuleCstNodes",
            "genericAtRulePreludeCstNodes",
            "bogusAtRulePreludeCstNodes",
            "nestingAtRuleCstNodes",
            "customMediaAtRuleCstNodes",
            "cssColorFunctionCstNodes",
            "colorFunctionArgumentChecks",
            "gradientFunctionCstNodes",
            "transformFunctionCstNodes",
            "filterFunctionCstNodes",
            "imageFunctionCstNodes",
            "shapeFunctionCstNodes",
            "envAttrFunctionCstNodes",
            "mathFunctionCstNodes",
            "mathFunctionArityChecks",
            "mathFunctionEmptyArgumentChecks",
            "varEnvAttrFunctionHeadChecks",
            "scssInterpolationTokenization",
            "scssInterpolationCstNodes",
            "lessInterpolationTokenization",
            "lessInterpolationCstNodes",
            "interpolationBogusRecovery",
            "unicodeRangeTokenization",
            "badStringTokenRecovery",
            "badStringValueBogusNodes",
            "emptyDeclarationValueRecovery",
            "emptyVariableValueRecovery",
            "missingSemicolonDeclarationRecovery",
            "coreBogusPopulationSlice",
            "dialectBogusPopulationSlice",
            "cssModuleValueCstNodes",
            "cssModuleComposesCstNodes",
            "icssModuleBlockCstNodes",
            "icssImportSourceValidation",
            "cssModuleFromClauseSourceValidation",
            "cssModuleComposesMultipleFromValidation",
            "cssModuleGlobalComposesValidation",
            "cssModuleBogusRecovery",
            "valueListCstNodes",
            "valueListBogusRecovery",
            "genericRecoveryBogusNodes",
            "sassIndentedTokenization",
            "sassIndentedBlockCstNodes",
            "sassIndentedStyleFacts",
            "differentialCorpusSeed",
            "differentialCorpus",
            "lightningCssDifferentialCorpusSlice",
            "lightningCssSelectorIdAndAtRuleDifferentialSlice",
            "midTypingNoPanicPropertySlice",
            "deterministicPanicFreeCorpus",
            "losslessCstTextRoundTripSmoke",
            "parseResultSourceTextSurface",
            "parseSourceParseRoundTripSmoke",
            "typedNumericValueAtomCstNodes",
            "bracketedValueCstNodes",
            "importantAnnotationCstNodes",
            "splitImportantAnnotationCstNodes",
            "unexpectedValueTokenBogusNodes",
            "cdoCdcTokenization",
            "cssIdentifierEscapeTokenization",
            "nullAndBomInputPreprocessingSlice",
            "hashDelimiterTokenization",
            "cssDashIdentTokenization",
            "signedNumericTokenization",
            "exponentNumericTokenization",
            "badUrlWhitespaceRecovery",
            "parserEntryPointApiSlice",
            "ruleListEntryPointApiSlice",
            "componentValueEntryPointApiSlice",
            "componentValueListEntryPointApiSlice",
            "commaSeparatedComponentValueListEntryPointApiSlice",
            "simpleBlockEntryPointApiSlice",
            "typedCstWrapperSlice",
            "parserCstEquivalence",
            "typedBogusCstWrapperSlice",
            "componentValueCstNodes",
            "simpleBlockCstNodes",
            "fullBogusPopulation",
            "componentValueListCstNodes",
            "commaSeparatedComponentValueListCstNodes",
            "customPropertyAnyValueComponentList",
            "customPropertyValueCstNodes",
            "functionalPseudoSelectorListCstNodes",
            "strictNotPseudoSelectorListCstNodes",
            "nthSelectorOfSelectorListCstNodes",
            "nthSelectorFormulaCstNodes",
            "hasRelativeSelectorListCstNodes",
            "langDirSelectorArgumentCstNodes",
            "namespaceQualifiedSelectorCstNodes",
            "selectorFunctionArgumentFactExclusion",
            "missingBlockCloseBogusTrivia",
            "initialDialectStatementNodes",
            "recoveryBogusSkeleton",
            "styleFactExtractionSurface",
            "parserSemanticNameConsumption",
            "productCutoverGate",
        ],
        not_ready_surfaces: vec![
            "completeExternalSpecMirror",
            "fullPropertyValueGrammarRegistry",
        ],
    }
}

pub fn summarize_omena_parser_style_facts(
    style_source: &str,
    dialect: StyleDialect,
) -> OmenaParserStyleFactsSummaryV0 {
    let facts = collect_style_facts(style_source, dialect);
    let sass_symbol_resolution = summarize_omena_parser_sass_symbol_resolution(&facts.sass_symbols);
    let mut class_selector_names = Vec::new();
    let mut id_selector_names = Vec::new();
    let mut placeholder_selector_names = Vec::new();
    let mut keyframe_names = Vec::new();
    let mut animation_reference_names = Vec::new();
    let mut css_module_value_definition_names = BTreeSet::new();
    let mut css_module_value_reference_names = BTreeSet::new();
    let mut css_module_value_import_sources = BTreeSet::new();
    let mut css_module_composes_target_names = BTreeSet::new();
    let mut css_module_composes_import_sources = BTreeSet::new();
    let mut icss_export_names = BTreeSet::new();
    let mut icss_import_local_names = BTreeSet::new();
    let mut icss_import_remote_names = BTreeSet::new();
    let mut icss_import_sources = BTreeSet::new();
    let mut variable_names = BTreeSet::new();
    let mut sass_symbol_declaration_names = BTreeSet::new();
    let mut sass_symbol_reference_names = BTreeSet::new();
    let mut sass_module_use_sources = BTreeSet::new();
    let mut sass_module_forward_sources = BTreeSet::new();
    let mut sass_module_import_sources = BTreeSet::new();
    let mut custom_property_names = BTreeSet::new();
    let mut custom_property_decl_names = BTreeSet::new();
    let mut custom_property_ref_names = BTreeSet::new();

    for selector in facts.selectors {
        match selector.kind {
            ParsedSelectorFactKind::Class => class_selector_names.push(selector.name),
            ParsedSelectorFactKind::Id => id_selector_names.push(selector.name),
            ParsedSelectorFactKind::Placeholder => placeholder_selector_names.push(selector.name),
        }
    }

    for variable in facts.variables {
        match variable.kind {
            ParsedVariableFactKind::ScssDeclaration
            | ParsedVariableFactKind::ScssReference
            | ParsedVariableFactKind::LessDeclaration
            | ParsedVariableFactKind::LessReference => {
                variable_names.insert(variable.name);
            }
            ParsedVariableFactKind::CustomPropertyDeclaration
            | ParsedVariableFactKind::CustomPropertyReference => {
                custom_property_names.insert(variable.name.clone());
                match variable.kind {
                    ParsedVariableFactKind::CustomPropertyDeclaration => {
                        custom_property_decl_names.insert(variable.name);
                    }
                    ParsedVariableFactKind::CustomPropertyReference => {
                        custom_property_ref_names.insert(variable.name);
                    }
                    _ => {}
                }
            }
        }
    }

    for symbol in &facts.sass_symbols {
        match symbol.role {
            "declaration" => {
                sass_symbol_declaration_names.insert(symbol.name.clone());
            }
            _ => {
                sass_symbol_reference_names.insert(symbol.name.clone());
            }
        }
    }

    for edge in &facts.sass_module_edges {
        match edge.kind {
            ParsedSassModuleEdgeFactKind::Use => {
                sass_module_use_sources.insert(edge.source.clone());
            }
            ParsedSassModuleEdgeFactKind::Forward => {
                sass_module_forward_sources.insert(edge.source.clone());
            }
            ParsedSassModuleEdgeFactKind::Import => {
                sass_module_import_sources.insert(edge.source.clone());
            }
        }
    }

    for animation in facts.animations {
        match animation.kind {
            ParsedAnimationFactKind::KeyframesDeclaration => keyframe_names.push(animation.name),
            ParsedAnimationFactKind::AnimationNameReference => {
                animation_reference_names.push(animation.name);
            }
        }
    }

    for value in facts.css_module_values {
        match value.kind {
            ParsedCssModuleValueFactKind::Definition => {
                css_module_value_definition_names.insert(value.name);
            }
            ParsedCssModuleValueFactKind::Reference => {
                css_module_value_reference_names.insert(value.name);
            }
            ParsedCssModuleValueFactKind::ImportSource => {
                css_module_value_import_sources.insert(value.name);
            }
        }
    }

    for composes in facts.css_module_composes {
        match composes.kind {
            ParsedCssModuleComposesFactKind::Target => {
                css_module_composes_target_names.insert(composes.name);
            }
            ParsedCssModuleComposesFactKind::ImportSource => {
                css_module_composes_import_sources.insert(composes.name);
            }
        }
    }

    for icss in facts.icss {
        match icss.kind {
            ParsedIcssFactKind::ExportName => {
                icss_export_names.insert(icss.name);
            }
            ParsedIcssFactKind::ImportLocalName => {
                icss_import_local_names.insert(icss.name);
            }
            ParsedIcssFactKind::ImportRemoteName => {
                icss_import_remote_names.insert(icss.name);
            }
            ParsedIcssFactKind::ImportSource => {
                icss_import_sources.insert(icss.name);
            }
        }
    }

    OmenaParserStyleFactsSummaryV0 {
        schema_version: "0",
        product: "omena-parser.style-facts",
        dialect: style_dialect_label(dialect),
        class_selector_names,
        id_selector_names,
        placeholder_selector_names,
        keyframe_names,
        animation_reference_names,
        css_module_value_definition_names: css_module_value_definition_names.into_iter().collect(),
        css_module_value_reference_names: css_module_value_reference_names.into_iter().collect(),
        css_module_value_import_sources: css_module_value_import_sources.into_iter().collect(),
        css_module_value_import_edges: facts
            .css_module_value_import_edges
            .into_iter()
            .map(|edge| OmenaParserCssModuleValueImportEdgeFactV0 {
                remote_name: edge.remote_name,
                local_name: edge.local_name,
                import_source: edge.import_source,
            })
            .collect(),
        css_module_value_definition_edges: facts
            .css_module_value_definition_edges
            .into_iter()
            .map(|edge| OmenaParserCssModuleValueDefinitionEdgeFactV0 {
                definition_name: edge.definition_name,
                reference_names: edge.reference_names,
            })
            .collect(),
        css_module_composes_target_names: css_module_composes_target_names.into_iter().collect(),
        css_module_composes_import_sources: css_module_composes_import_sources
            .into_iter()
            .collect(),
        css_module_composes_edges: facts
            .css_module_composes_edges
            .into_iter()
            .map(|edge| OmenaParserCssModuleComposesEdgeFactV0 {
                kind: css_module_composes_edge_kind_label(edge.kind),
                owner_selector_names: edge.owner_selector_names,
                target_names: edge.target_names,
                import_source: edge.import_source,
            })
            .collect(),
        icss_export_names: icss_export_names.into_iter().collect(),
        icss_import_local_names: icss_import_local_names.into_iter().collect(),
        icss_import_remote_names: icss_import_remote_names.into_iter().collect(),
        icss_import_sources: icss_import_sources.into_iter().collect(),
        icss_import_edges: facts
            .icss_import_edges
            .into_iter()
            .map(|edge| OmenaParserIcssImportEdgeFactV0 {
                local_name: edge.local_name,
                remote_name: edge.remote_name,
                import_source: edge.import_source,
            })
            .collect(),
        icss_export_edges: facts
            .icss_export_edges
            .into_iter()
            .map(|edge| OmenaParserIcssExportEdgeFactV0 {
                export_name: edge.export_name,
                reference_names: edge.reference_names,
            })
            .collect(),
        variable_names: variable_names.into_iter().collect(),
        sass_symbol_declaration_names: sass_symbol_declaration_names.into_iter().collect(),
        sass_symbol_reference_names: sass_symbol_reference_names.into_iter().collect(),
        sass_symbol_facts: facts
            .sass_symbols
            .into_iter()
            .map(|symbol| OmenaParserSassSymbolFactV0 {
                kind: sass_symbol_fact_kind_label(symbol.kind),
                symbol_kind: symbol.symbol_kind,
                name: symbol.name,
                role: symbol.role,
                namespace: symbol.namespace,
            })
            .collect(),
        sass_symbol_resolution,
        sass_module_use_sources: sass_module_use_sources.into_iter().collect(),
        sass_module_forward_sources: sass_module_forward_sources.into_iter().collect(),
        sass_module_import_sources: sass_module_import_sources.into_iter().collect(),
        sass_module_edges: facts
            .sass_module_edges
            .into_iter()
            .map(|edge| OmenaParserSassModuleEdgeFactV0 {
                kind: sass_module_edge_fact_kind_label(edge.kind),
                source: edge.source,
                namespace_kind: edge.namespace_kind,
                namespace: edge.namespace,
                visibility_filter_kind: edge.visibility_filter_kind,
                visibility_filter_names: edge.visibility_filter_names,
            })
            .collect(),
        custom_property_names: custom_property_names.into_iter().collect(),
        custom_property_decl_names: custom_property_decl_names.into_iter().collect(),
        custom_property_ref_names: custom_property_ref_names.into_iter().collect(),
        at_rule_names: facts
            .at_rules
            .into_iter()
            .map(|at_rule| at_rule.name)
            .collect(),
        parser_error_count: facts.error_count,
    }
}

pub fn summarize_omena_parser_lex(source: &str, dialect: StyleDialect) -> OmenaParserLexSummaryV0 {
    let result = lex(source, dialect);
    OmenaParserLexSummaryV0 {
        schema_version: "0",
        product: "omena-parser.lex-result",
        dialect: style_dialect_label(result.dialect()),
        tokens: result
            .tokens()
            .iter()
            .map(|token| OmenaParserLexTokenV0 {
                kind: format!("{:?}", token.kind),
                text: token.text.clone(),
                start: token.range.start().into(),
                end: token.range.end().into(),
            })
            .collect(),
        parser_error_count: result.errors().len(),
    }
}

pub fn summarize_omena_parser_parity_lite(
    source: &str,
    dialect: StyleDialect,
) -> OmenaParserParityLiteSummaryV0 {
    let facts = collect_style_facts(source, dialect);
    let result = parse(source, dialect);
    let (tokens, _) = tokenize(source, &BuiltinDialectExtension::new(dialect));
    let mut structural = ParserStructuralSummary::default();
    summarize_parser_structural_range(&tokens, 0, tokens.len(), 0, &mut structural);
    let mut selector_names = collect_parity_lite_selector_names_from_tokens(&tokens);
    selector_names.sort();

    OmenaParserParityLiteSummaryV0 {
        schema_version: "0",
        language: style_dialect_label(dialect),
        selector_names,
        keyframes_names: sorted_unique(
            facts
                .animations
                .iter()
                .filter(|animation| animation.kind == ParsedAnimationFactKind::KeyframesDeclaration)
                .map(|animation| animation.name.clone()),
        ),
        value_decl_names: sorted_unique(
            facts
                .css_module_values
                .iter()
                .filter(|value| value.kind == ParsedCssModuleValueFactKind::Definition)
                .map(|value| value.name.clone()),
        ),
        diagnostic_count: result.errors().len(),
        rule_count: structural.rule_count,
        declaration_count: structural.declaration_count,
        grouped_selector_count: structural.grouped_selector_count,
        max_nesting_depth: structural.max_nesting_depth,
        at_rule_kind_counts: structural.at_rule_kind_counts,
        declaration_kind_counts: structural.declaration_kind_counts,
    }
}

fn style_dialect_label(dialect: StyleDialect) -> &'static str {
    match dialect {
        StyleDialect::Css => "css",
        StyleDialect::Scss => "scss",
        StyleDialect::Sass => "sass",
        StyleDialect::Less => "less",
    }
}

#[derive(Default)]
struct ParserStructuralSummary {
    rule_count: usize,
    declaration_count: usize,
    grouped_selector_count: usize,
    max_nesting_depth: usize,
    at_rule_kind_counts: OmenaParserAtRuleKindCountsV0,
    declaration_kind_counts: OmenaParserDeclarationKindCountsV0,
}

fn summarize_parser_structural_range(
    tokens: &[Token<'_>],
    start: usize,
    end: usize,
    depth: usize,
    summary: &mut ParserStructuralSummary,
) {
    let mut index = start;
    while index < end {
        index = skip_trivia_tokens(tokens, index, end);
        if index >= end {
            break;
        }

        if tokens[index].kind == SyntaxKind::AtKeyword {
            increment_omena_parser_at_rule_kind_count(
                &mut summary.at_rule_kind_counts,
                classify_omena_parser_at_rule_kind(tokens[index].text),
            );
            let next_depth = depth + 1;
            summary.max_nesting_depth = summary.max_nesting_depth.max(next_depth);
            if let Some((open, close)) = find_block_after_header(tokens, index, end) {
                summarize_parser_structural_range(tokens, open + 1, close, next_depth, summary);
                index = close + 1;
            } else {
                index = skip_statement(tokens, index, end);
            }
            continue;
        }

        let statement_end = css_module_value_statement_end(tokens, index);
        if is_root_less_variable_statement(tokens, index, statement_end.min(end), depth) {
            increment_omena_parser_at_rule_kind_count(
                &mut summary.at_rule_kind_counts,
                keyof_omena_parser_at_rule_kind_counts::Kind::Generic,
            );
            if statement_end >= end || tokens[statement_end].kind == SyntaxKind::RightBrace {
                break;
            }
            index = statement_end + 1;
            continue;
        }

        if statement_end < end && tokens[statement_end].kind == SyntaxKind::LeftBrace {
            summary.rule_count += 1;
            let next_depth = depth + 1;
            summary.max_nesting_depth = summary.max_nesting_depth.max(next_depth);
            let group_count = count_omena_parser_selector_groups(tokens, index, statement_end);
            if group_count > 1 {
                summary.grouped_selector_count += group_count;
            }
            if let Some(close) = matching_right_brace(tokens, statement_end, end) {
                summarize_parser_structural_range(
                    tokens,
                    statement_end + 1,
                    close,
                    next_depth,
                    summary,
                );
                index = close + 1;
            } else {
                index = statement_end + 1;
            }
            continue;
        }

        if let Some(colon_index) = declaration_colon_index(tokens, index, statement_end.min(end)) {
            summary.declaration_count += 1;
            let property = previous_non_trivia_token_index(tokens, colon_index, index)
                .map(|property| tokens[property].text)
                .unwrap_or_default();
            increment_omena_parser_declaration_kind_count(
                &mut summary.declaration_kind_counts,
                classify_omena_parser_declaration_kind(property),
            );
        }

        if statement_end >= end || tokens[statement_end].kind == SyntaxKind::RightBrace {
            break;
        }
        index = statement_end + 1;
    }
}

fn is_root_less_variable_statement(
    tokens: &[Token<'_>],
    start: usize,
    end: usize,
    depth: usize,
) -> bool {
    if depth != 0 {
        return false;
    }
    let Some(first) = next_non_trivia_token_index_until(tokens, start, end) else {
        return false;
    };
    tokens[first].kind == SyntaxKind::LessVariable
        && declaration_colon_index(tokens, first, end).is_some()
}

fn count_omena_parser_selector_groups(tokens: &[Token<'_>], start: usize, end: usize) -> usize {
    split_selector_groups(tokens, start, end)
        .into_iter()
        .filter(|(group_start, group_end)| {
            *group_start < *group_end
                && next_non_trivia_token_index_until(tokens, *group_start, *group_end).is_some()
        })
        .count()
}

fn collect_parity_lite_selector_names_from_tokens(tokens: &[Token<'_>]) -> Vec<String> {
    let mut names = Vec::new();
    collect_parity_lite_selector_names_in_range(tokens, 0, tokens.len(), &[], None, &mut names);
    names
}

fn collect_parity_lite_selector_names_in_range(
    tokens: &[Token<'_>],
    start: usize,
    end: usize,
    parent_branches: &[SelectorBranch],
    css_module_scope: Option<&'static str>,
    names: &mut Vec<String>,
) {
    let mut index = start;
    while index < end {
        index = skip_trivia_tokens(tokens, index, end);
        if index >= end {
            break;
        }

        if tokens[index].kind == SyntaxKind::AtKeyword {
            let block = find_block_after_header(tokens, index, end);
            if let Some((open, close)) = block {
                if tokens[index].text == "@nest" {
                    if css_module_scope == Some("global") {
                        collect_parity_lite_selector_names_in_range(
                            tokens,
                            open + 1,
                            close,
                            &[],
                            css_module_scope,
                            names,
                        );
                    } else {
                        let branches =
                            resolve_selector_header(tokens, index + 1, open, parent_branches);
                        names.extend(branches.iter().map(|branch| branch.name.clone()));
                        collect_grouped_ampersand_compound_selector_duplicates(
                            tokens,
                            index + 1,
                            open,
                            parent_branches.len(),
                            names,
                        );
                        collect_parity_lite_selector_names_in_range(
                            tokens,
                            open + 1,
                            close,
                            &branches,
                            css_module_scope,
                            names,
                        );
                    }
                } else if style_wrapper_at_rule(tokens[index].text) {
                    collect_parity_lite_selector_names_in_range(
                        tokens,
                        open + 1,
                        close,
                        parent_branches,
                        css_module_scope,
                        names,
                    );
                }
                index = close + 1;
            } else {
                index = skip_statement(tokens, index, end);
            }
            continue;
        }

        let Some((open, close)) = find_block_after_header(tokens, index, end) else {
            index = skip_statement(tokens, index, end);
            continue;
        };

        let effective_scope = css_module_scope
            .or_else(|| css_module_block_scope_marker_in_header(tokens, index, open));
        if effective_scope == Some("global") {
            collect_parity_lite_selector_names_in_range(
                tokens,
                open + 1,
                close,
                &[],
                effective_scope,
                names,
            );
        } else {
            let branches = resolve_selector_header(tokens, index, open, parent_branches);
            names.extend(branches.iter().map(|branch| branch.name.clone()));
            collect_grouped_ampersand_compound_selector_duplicates(
                tokens,
                index,
                open,
                parent_branches.len(),
                names,
            );
            collect_parity_lite_selector_names_in_range(
                tokens,
                open + 1,
                close,
                &branches,
                effective_scope,
                names,
            );
        }
        index = close + 1;
    }
}

fn collect_grouped_ampersand_compound_selector_duplicates(
    tokens: &[Token<'_>],
    start: usize,
    end: usize,
    parent_branch_count: usize,
    names: &mut Vec<String>,
) {
    if parent_branch_count <= 1 || !header_contains_ampersand(tokens, start, end) {
        return;
    }
    for (name, _) in collect_class_selector_names_from_header(tokens, start, end) {
        names.extend(std::iter::repeat_n(name, parent_branch_count - 1));
    }
}

fn header_contains_ampersand(tokens: &[Token<'_>], start: usize, end: usize) -> bool {
    tokens[start..end]
        .iter()
        .any(|token| token.kind == SyntaxKind::Ampersand)
}

fn classify_omena_parser_at_rule_kind(text: &str) -> keyof_omena_parser_at_rule_kind_counts::Kind {
    match text.trim_start_matches('@').to_ascii_lowercase().as_str() {
        "media" => keyof_omena_parser_at_rule_kind_counts::Kind::Media,
        "supports" => keyof_omena_parser_at_rule_kind_counts::Kind::Supports,
        "layer" => keyof_omena_parser_at_rule_kind_counts::Kind::Layer,
        "keyframes" | "-webkit-keyframes" => {
            keyof_omena_parser_at_rule_kind_counts::Kind::Keyframes
        }
        "value" => keyof_omena_parser_at_rule_kind_counts::Kind::Value,
        "at-root" => keyof_omena_parser_at_rule_kind_counts::Kind::AtRoot,
        _ => keyof_omena_parser_at_rule_kind_counts::Kind::Generic,
    }
}

fn increment_omena_parser_at_rule_kind_count(
    counts: &mut OmenaParserAtRuleKindCountsV0,
    kind: keyof_omena_parser_at_rule_kind_counts::Kind,
) {
    match kind {
        keyof_omena_parser_at_rule_kind_counts::Kind::Media => counts.media += 1,
        keyof_omena_parser_at_rule_kind_counts::Kind::Supports => counts.supports += 1,
        keyof_omena_parser_at_rule_kind_counts::Kind::Layer => counts.layer += 1,
        keyof_omena_parser_at_rule_kind_counts::Kind::Keyframes => counts.keyframes += 1,
        keyof_omena_parser_at_rule_kind_counts::Kind::Value => counts.value += 1,
        keyof_omena_parser_at_rule_kind_counts::Kind::AtRoot => counts.at_root += 1,
        keyof_omena_parser_at_rule_kind_counts::Kind::Generic => counts.generic += 1,
    }
}

fn classify_omena_parser_declaration_kind(
    property: &str,
) -> keyof_omena_parser_declaration_kind_counts::Kind {
    match property.trim().to_ascii_lowercase().as_str() {
        "composes" => keyof_omena_parser_declaration_kind_counts::Kind::Composes,
        "animation" => keyof_omena_parser_declaration_kind_counts::Kind::Animation,
        "animation-name" => keyof_omena_parser_declaration_kind_counts::Kind::AnimationName,
        _ => keyof_omena_parser_declaration_kind_counts::Kind::Generic,
    }
}

fn increment_omena_parser_declaration_kind_count(
    counts: &mut OmenaParserDeclarationKindCountsV0,
    kind: keyof_omena_parser_declaration_kind_counts::Kind,
) {
    match kind {
        keyof_omena_parser_declaration_kind_counts::Kind::Composes => counts.composes += 1,
        keyof_omena_parser_declaration_kind_counts::Kind::Animation => counts.animation += 1,
        keyof_omena_parser_declaration_kind_counts::Kind::AnimationName => {
            counts.animation_name += 1
        }
        keyof_omena_parser_declaration_kind_counts::Kind::Generic => counts.generic += 1,
    }
}

mod keyof_omena_parser_at_rule_kind_counts {
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub enum Kind {
        Media,
        Supports,
        Layer,
        Keyframes,
        Value,
        AtRoot,
        Generic,
    }
}

mod keyof_omena_parser_declaration_kind_counts {
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub enum Kind {
        Composes,
        Animation,
        AnimationName,
        Generic,
    }
}

fn sorted_unique(values: impl IntoIterator<Item = String>) -> Vec<String> {
    values
        .into_iter()
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect()
}

fn css_module_composes_edge_kind_label(kind: ParsedCssModuleComposesEdgeKind) -> &'static str {
    match kind {
        ParsedCssModuleComposesEdgeKind::Local => "local",
        ParsedCssModuleComposesEdgeKind::Global => "global",
        ParsedCssModuleComposesEdgeKind::External => "external",
    }
}

fn sass_symbol_fact_kind_label(kind: ParsedSassSymbolFactKind) -> &'static str {
    match kind {
        ParsedSassSymbolFactKind::VariableDeclaration => "sassVariableDeclaration",
        ParsedSassSymbolFactKind::VariableReference => "sassVariableReference",
        ParsedSassSymbolFactKind::MixinDeclaration => "sassMixinDeclaration",
        ParsedSassSymbolFactKind::MixinInclude => "sassMixinInclude",
        ParsedSassSymbolFactKind::FunctionDeclaration => "sassFunctionDeclaration",
        ParsedSassSymbolFactKind::FunctionCall => "sassFunctionCall",
    }
}

fn sass_module_edge_fact_kind_label(kind: ParsedSassModuleEdgeFactKind) -> &'static str {
    match kind {
        ParsedSassModuleEdgeFactKind::Use => "sassUse",
        ParsedSassModuleEdgeFactKind::Forward => "sassForward",
        ParsedSassModuleEdgeFactKind::Import => "sassImport",
    }
}

fn summarize_omena_parser_sass_symbol_resolution(
    symbols: &[ParsedSassSymbolFact],
) -> OmenaParserSassSymbolResolutionV0 {
    let mut declaration_by_symbol: BTreeMap<
        (&'static str, Option<String>, String),
        (usize, &'static str),
    > = BTreeMap::new();
    let mut declaration_count = 0usize;
    let mut reference_count = 0usize;
    let mut edges = Vec::new();

    for (source_order, symbol) in symbols.iter().enumerate() {
        let kind = sass_symbol_fact_kind_label(symbol.kind);
        if sass_symbol_fact_kind_is_declaration(symbol.kind) {
            declaration_count += 1;
            declaration_by_symbol.insert(
                (
                    symbol.symbol_kind,
                    symbol.namespace.clone(),
                    symbol.name.clone(),
                ),
                (source_order, kind),
            );
            continue;
        }
        if !sass_symbol_fact_kind_is_reference(symbol.kind) {
            continue;
        }

        reference_count += 1;
        let declaration = declaration_by_symbol.get(&(
            symbol.symbol_kind,
            symbol.namespace.clone(),
            symbol.name.clone(),
        ));
        edges.push(OmenaParserSassSymbolResolutionEdgeV0 {
            symbol_kind: symbol.symbol_kind,
            name: symbol.name.clone(),
            namespace: symbol.namespace.clone(),
            reference_kind: kind,
            reference_role: symbol.role,
            reference_source_order: source_order,
            declaration_kind: declaration.map(|(_, declaration_kind)| *declaration_kind),
            declaration_source_order: declaration.map(|(declaration_order, _)| *declaration_order),
            status: if declaration.is_some() {
                "resolved"
            } else {
                "unresolved"
            },
        });
    }

    let resolved_reference_count = edges
        .iter()
        .filter(|edge| edge.status == "resolved")
        .count();

    OmenaParserSassSymbolResolutionV0 {
        schema_version: "0",
        product: "omena-parser.sass-symbol-same-file-resolution",
        resolution_scope: "same-file",
        declaration_count,
        reference_count,
        resolved_reference_count,
        unresolved_reference_count: reference_count.saturating_sub(resolved_reference_count),
        edges,
        capabilities: OmenaParserSassSymbolResolutionCapabilitiesV0 {
            same_file_lexical_resolution_ready: true,
            declaration_before_reference_ready: true,
            unresolved_reference_reporting_ready: true,
            cross_file_module_resolution_ready: false,
        },
    }
}

fn sass_symbol_fact_kind_is_declaration(kind: ParsedSassSymbolFactKind) -> bool {
    matches!(
        kind,
        ParsedSassSymbolFactKind::VariableDeclaration
            | ParsedSassSymbolFactKind::MixinDeclaration
            | ParsedSassSymbolFactKind::FunctionDeclaration
    )
}

fn sass_symbol_fact_kind_is_reference(kind: ParsedSassSymbolFactKind) -> bool {
    matches!(
        kind,
        ParsedSassSymbolFactKind::VariableReference
            | ParsedSassSymbolFactKind::MixinInclude
            | ParsedSassSymbolFactKind::FunctionCall
    )
}

pub fn summarize_parser_cst_equivalence(
    text: &str,
    dialect: StyleDialect,
) -> ParserCstEquivalenceSummaryV0 {
    let result = parse(text, dialect);
    let syntax = result.syntax();
    let cst = result.cst();

    let mut node_count = 0;
    let mut token_count = 0;
    let mut syntax_kind_round_trip_ready = true;
    let mut zero_unknown_kind_ready = true;

    for node in syntax.descendants() {
        node_count += 1;
        let kind = node.kind();
        syntax_kind_round_trip_ready &= SyntaxKind::from_raw(kind.into_raw()) == kind;
        zero_unknown_kind_ready &= SyntaxKind::ALL.contains(&kind);
    }

    for token in syntax
        .descendants_with_tokens()
        .filter_map(|element| element.into_token())
    {
        token_count += 1;
        let kind = token.kind();
        syntax_kind_round_trip_ready &= SyntaxKind::from_raw(kind.into_raw()) == kind;
        zero_unknown_kind_ready &= SyntaxKind::ALL.contains(&kind);
    }

    let typed_wrapper_count = usize::from(cst.stylesheet().is_some())
        + cst.rules().len()
        + cst.selectors().len()
        + cst.declarations().len()
        + cst.declaration_lists().len()
        + cst.values().len()
        + cst.component_values().len()
        + cst.simple_blocks().len()
        + cst.component_value_lists().len()
        + cst.comma_separated_component_value_lists().len()
        + cst.custom_property_values().len()
        + cst.at_rules().len()
        + cst.bogus_nodes().len();

    ParserCstEquivalenceSummaryV0 {
        product: "omena-parser.cst-equivalence",
        dialect,
        root_kind: syntax.kind(),
        parser_node_count: node_count,
        parser_token_count: token_count,
        typed_wrapper_count,
        source_text_round_trip_ready: result.source_text().as_deref() == Some(text),
        syntax_kind_round_trip_ready,
        zero_unknown_kind_ready,
        typed_cst_wrapper_ready: cst.stylesheet().is_some() && typed_wrapper_count > 1,
        ready_surfaces: vec![
            "parserCstEquivalence",
            "parserUsesOmenaSyntaxKind",
            "parserCstSourceTextRoundTrip",
            "typedCstWrapperEquivalence",
        ],
    }
}

pub fn summarize_parser_semantic_name_consumption(
    text: &str,
    dialect: StyleDialect,
    db: &dyn salsa::Database,
) -> ParserSemanticNameConsumptionSummaryV0 {
    let facts = collect_style_facts(text, dialect);
    let candidates = parser_semantic_name_candidates(&facts);
    let interned_name_count = candidates
        .iter()
        .filter(|candidate| intern_parser_semantic_name(db, candidate.kind, &candidate.text))
        .count();
    let invalid_name_count = candidates.len().saturating_sub(interned_name_count);

    ParserSemanticNameConsumptionSummaryV0 {
        product: "omena-parser.semantic-name-consumption",
        dialect,
        semantic_name_count: candidates.len(),
        interned_name_count,
        invalid_name_count,
        class_name_count: count_parser_semantic_name_kind(&candidates, NameKind::ClassName),
        css_ident_count: count_parser_semantic_name_kind(&candidates, NameKind::CssIdent),
        property_name_count: count_parser_semantic_name_kind(&candidates, NameKind::PropertyName),
        selector_key_count: count_parser_semantic_name_kind(&candidates, NameKind::SelectorKey),
        custom_property_name_count: count_parser_semantic_name_kind(
            &candidates,
            NameKind::CustomPropertyName,
        ),
        keyframes_name_count: count_parser_semantic_name_kind(&candidates, NameKind::KeyframesName),
        mixin_name_count: count_parser_semantic_name_kind(&candidates, NameKind::MixinName),
        file_path_count: count_parser_semantic_name_kind(&candidates, NameKind::FilePath),
        ready_surfaces: vec![
            "parserSemanticNameConsumption",
            "typedInternerValidation",
            "styleFactNameKindProjection",
        ],
    }
}

fn parser_semantic_name_candidates(facts: &ParsedStyleFacts) -> Vec<ParserSemanticNameCandidateV0> {
    let mut candidates = Vec::new();

    for selector in &facts.selectors {
        let kind = match selector.kind {
            ParsedSelectorFactKind::Class => NameKind::ClassName,
            ParsedSelectorFactKind::Id | ParsedSelectorFactKind::Placeholder => {
                NameKind::SelectorKey
            }
        };
        push_parser_semantic_name_candidate(&mut candidates, kind, &selector.name);
    }

    for variable in &facts.variables {
        let kind = match variable.kind {
            ParsedVariableFactKind::CustomPropertyDeclaration
            | ParsedVariableFactKind::CustomPropertyReference => NameKind::CustomPropertyName,
            ParsedVariableFactKind::ScssDeclaration
            | ParsedVariableFactKind::ScssReference
            | ParsedVariableFactKind::LessDeclaration
            | ParsedVariableFactKind::LessReference => NameKind::CssIdent,
        };
        push_parser_semantic_name_candidate(&mut candidates, kind, &variable.name);
    }

    for symbol in &facts.sass_symbols {
        let kind = match symbol.kind {
            ParsedSassSymbolFactKind::MixinDeclaration | ParsedSassSymbolFactKind::MixinInclude => {
                NameKind::MixinName
            }
            ParsedSassSymbolFactKind::VariableDeclaration
            | ParsedSassSymbolFactKind::VariableReference
            | ParsedSassSymbolFactKind::FunctionDeclaration
            | ParsedSassSymbolFactKind::FunctionCall => NameKind::CssIdent,
        };
        push_parser_semantic_name_candidate(&mut candidates, kind, &symbol.name);
        if let Some(namespace) = &symbol.namespace {
            push_parser_semantic_name_candidate(&mut candidates, NameKind::CssIdent, namespace);
        }
    }

    for include in &facts.sass_includes {
        push_parser_semantic_name_candidate(&mut candidates, NameKind::MixinName, &include.name);
        if let Some(namespace) = &include.namespace {
            push_parser_semantic_name_candidate(&mut candidates, NameKind::CssIdent, namespace);
        }
    }

    for edge in &facts.sass_module_edges {
        push_parser_semantic_name_candidate(&mut candidates, NameKind::FilePath, &edge.source);
        if let Some(namespace) = &edge.namespace {
            push_parser_semantic_name_candidate(&mut candidates, NameKind::CssIdent, namespace);
        }
    }

    for animation in &facts.animations {
        push_parser_semantic_name_candidate(
            &mut candidates,
            NameKind::KeyframesName,
            &animation.name,
        );
    }

    for value in &facts.css_module_values {
        let kind = match value.kind {
            ParsedCssModuleValueFactKind::Definition | ParsedCssModuleValueFactKind::Reference => {
                NameKind::CssIdent
            }
            ParsedCssModuleValueFactKind::ImportSource => NameKind::FilePath,
        };
        push_parser_semantic_name_candidate(&mut candidates, kind, &value.name);
    }

    for edge in &facts.css_module_value_import_edges {
        push_parser_semantic_name_candidate(&mut candidates, NameKind::CssIdent, &edge.local_name);
        push_parser_semantic_name_candidate(&mut candidates, NameKind::CssIdent, &edge.remote_name);
        push_parser_semantic_name_candidate(
            &mut candidates,
            NameKind::FilePath,
            &edge.import_source,
        );
    }

    for edge in &facts.css_module_value_definition_edges {
        push_parser_semantic_name_candidate(
            &mut candidates,
            NameKind::CssIdent,
            &edge.definition_name,
        );
        for reference_name in &edge.reference_names {
            push_parser_semantic_name_candidate(
                &mut candidates,
                NameKind::CssIdent,
                reference_name,
            );
        }
    }

    for composes in &facts.css_module_composes {
        let kind = match composes.kind {
            ParsedCssModuleComposesFactKind::Target => NameKind::ClassName,
            ParsedCssModuleComposesFactKind::ImportSource => NameKind::FilePath,
        };
        push_parser_semantic_name_candidate(&mut candidates, kind, &composes.name);
    }

    for edge in &facts.css_module_composes_edges {
        for owner_selector_name in &edge.owner_selector_names {
            push_parser_semantic_name_candidate(
                &mut candidates,
                NameKind::ClassName,
                owner_selector_name,
            );
        }
        for target_name in &edge.target_names {
            push_parser_semantic_name_candidate(&mut candidates, NameKind::ClassName, target_name);
        }
        if let Some(import_source) = &edge.import_source {
            push_parser_semantic_name_candidate(&mut candidates, NameKind::FilePath, import_source);
        }
    }

    for icss in &facts.icss {
        let kind = match icss.kind {
            ParsedIcssFactKind::ImportSource => NameKind::FilePath,
            ParsedIcssFactKind::ExportName
            | ParsedIcssFactKind::ImportLocalName
            | ParsedIcssFactKind::ImportRemoteName => NameKind::CssIdent,
        };
        push_parser_semantic_name_candidate(&mut candidates, kind, &icss.name);
    }

    for edge in &facts.icss_import_edges {
        push_parser_semantic_name_candidate(&mut candidates, NameKind::CssIdent, &edge.local_name);
        push_parser_semantic_name_candidate(&mut candidates, NameKind::CssIdent, &edge.remote_name);
        push_parser_semantic_name_candidate(
            &mut candidates,
            NameKind::FilePath,
            &edge.import_source,
        );
    }

    for edge in &facts.icss_export_edges {
        push_parser_semantic_name_candidate(&mut candidates, NameKind::CssIdent, &edge.export_name);
        for reference_name in &edge.reference_names {
            push_parser_semantic_name_candidate(
                &mut candidates,
                NameKind::CssIdent,
                reference_name,
            );
        }
    }

    for at_rule in &facts.at_rules {
        push_parser_semantic_name_candidate(&mut candidates, NameKind::CssIdent, &at_rule.name);
    }

    candidates
}

fn push_parser_semantic_name_candidate(
    candidates: &mut Vec<ParserSemanticNameCandidateV0>,
    kind: NameKind,
    text: &str,
) {
    candidates.push(ParserSemanticNameCandidateV0 {
        kind,
        text: text.to_string(),
    });
}

fn count_parser_semantic_name_kind(
    candidates: &[ParserSemanticNameCandidateV0],
    kind: NameKind,
) -> usize {
    candidates
        .iter()
        .filter(|candidate| candidate.kind == kind)
        .count()
}

fn intern_parser_semantic_name(db: &dyn salsa::Database, kind: NameKind, text: &str) -> bool {
    match kind {
        NameKind::ClassName => intern_class_name(db, text).is_ok(),
        NameKind::CssIdent => intern_css_ident(db, text).is_ok(),
        NameKind::PropertyName => intern_property_name(db, text).is_ok(),
        NameKind::SelectorKey => intern_selector_key(db, text).is_ok(),
        NameKind::CustomPropertyName => intern_custom_property_name(db, text).is_ok(),
        NameKind::KeyframesName => intern_keyframes_name(db, text).is_ok(),
        NameKind::MixinName => intern_mixin_name(db, text).is_ok(),
        NameKind::FilePath => intern_file_path(db, text).is_ok(),
    }
}

//! Borrowed typed CST projection over the parser-owned syntax tree.

use omena_parser::{
    ParseError, ParseResult, ParseTreeNodeV0, ParserByteSpanV0, ParserPositionV0, ParserRangeV0,
};
use omena_syntax::{StyleDialect, SyntaxElementRef, SyntaxKind, SyntaxNode, SyntaxToken};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct NamedChildAccessor {
    pub name: &'static str,
    pub accepted_kinds: &'static [SyntaxKind],
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct NamedTokenGetter {
    pub name: &'static str,
    pub accepted_kinds: &'static [SyntaxKind],
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct NodeGrammarRow {
    pub kind: SyntaxKind,
    pub name: &'static str,
    pub named_child_accessors: &'static [NamedChildAccessor],
    pub named_token_getters: &'static [NamedTokenGetter],
}

const STRUCTURAL_MANIFEST_KINDS: &[SyntaxKind] = &[
    SyntaxKind::Stylesheet,
    SyntaxKind::Rule,
    SyntaxKind::QualifiedRule,
    SyntaxKind::Declaration,
    SyntaxKind::DeclarationList,
    SyntaxKind::RuleList,
    SyntaxKind::SelectorList,
    SyntaxKind::Selector,
    SyntaxKind::ComplexSelector,
    SyntaxKind::CompoundSelector,
    SyntaxKind::ClassSelector,
    SyntaxKind::IdSelector,
    SyntaxKind::TypeSelector,
    SyntaxKind::UniversalSelector,
    SyntaxKind::AttributeSelector,
    SyntaxKind::AttributeMatcher,
    SyntaxKind::PseudoClassSelector,
    SyntaxKind::PseudoElementSelector,
    SyntaxKind::PseudoSelectorArgument,
    SyntaxKind::NestingSelectorNode,
    SyntaxKind::Combinator,
    SyntaxKind::SelectorValue,
    SyntaxKind::PropertyName,
    SyntaxKind::CustomPropertyDeclaration,
    SyntaxKind::Value,
    SyntaxKind::ValueList,
    SyntaxKind::FunctionCall,
    SyntaxKind::FunctionArguments,
    SyntaxKind::BinaryExpression,
    SyntaxKind::UnaryExpression,
    SyntaxKind::ParenthesizedExpression,
    SyntaxKind::Interpolation,
    SyntaxKind::DimensionValue,
    SyntaxKind::ColorValue,
    SyntaxKind::UrlValue,
    SyntaxKind::VarFunction,
    SyntaxKind::CalcFunction,
    SyntaxKind::AtRule,
    SyntaxKind::MediaRule,
    SyntaxKind::SupportsRule,
    SyntaxKind::ContainerRule,
    SyntaxKind::LayerRule,
    SyntaxKind::ScopeRule,
    SyntaxKind::KeyframesRule,
    SyntaxKind::KeyframeBlock,
    SyntaxKind::FontFaceRule,
    SyntaxKind::PageRule,
    SyntaxKind::NamespaceRule,
    SyntaxKind::ImportRule,
    SyntaxKind::CharsetRule,
    SyntaxKind::PropertyRule,
    SyntaxKind::StartingStyleRule,
    SyntaxKind::MediaQueryList,
    SyntaxKind::MediaQuery,
    SyntaxKind::MediaFeature,
    SyntaxKind::SupportsCondition,
    SyntaxKind::ContainerCondition,
    SyntaxKind::LayerName,
    SyntaxKind::ScopeRange,
    SyntaxKind::CssModuleLocalBlock,
    SyntaxKind::CssModuleGlobalBlock,
    SyntaxKind::CssModuleExportBlock,
    SyntaxKind::CssModuleImportBlock,
    SyntaxKind::CssModuleComposesDeclaration,
    SyntaxKind::CssModuleComposesTarget,
    SyntaxKind::CssModuleFromClause,
    SyntaxKind::TokenDefinition,
    SyntaxKind::TokenReference,
    SyntaxKind::Comment,
    SyntaxKind::ErrorNode,
    SyntaxKind::EnvFunction,
    SyntaxKind::AttrFunction,
    SyntaxKind::MathFunction,
    SyntaxKind::PageMarginRule,
    SyntaxKind::WhenRule,
    SyntaxKind::ElseRule,
    SyntaxKind::CounterStyleRule,
    SyntaxKind::FontPaletteValuesRule,
    SyntaxKind::ColorProfileRule,
    SyntaxKind::PositionTryRule,
    SyntaxKind::FontFeatureValuesRule,
    SyntaxKind::FontFeatureValuesStylisticRule,
    SyntaxKind::FontFeatureValuesStylesetRule,
    SyntaxKind::FontFeatureValuesCharacterVariantRule,
    SyntaxKind::FontFeatureValuesSwashRule,
    SyntaxKind::FontFeatureValuesOrnamentsRule,
    SyntaxKind::FontFeatureValuesAnnotationRule,
    SyntaxKind::FontFeatureValuesHistoricalFormsRule,
    SyntaxKind::ViewTransitionRule,
    SyntaxKind::GradientFunction,
    SyntaxKind::TransformFunction,
    SyntaxKind::FilterFunction,
    SyntaxKind::ImageFunction,
    SyntaxKind::ShapeFunction,
    SyntaxKind::AtRulePrelude,
    SyntaxKind::NestRule,
    SyntaxKind::CustomMediaRule,
    SyntaxKind::IdentifierValue,
    SyntaxKind::StringValue,
    SyntaxKind::UnicodeRangeValue,
    SyntaxKind::NumberValue,
    SyntaxKind::PercentageValue,
    SyntaxKind::BracketedValue,
    SyntaxKind::ImportantAnnotation,
    SyntaxKind::ComponentValue,
    SyntaxKind::SimpleBlock,
    SyntaxKind::ComponentValueList,
    SyntaxKind::CommaSeparatedComponentValueList,
    SyntaxKind::CustomPropertyValue,
    SyntaxKind::AttributeName,
    SyntaxKind::AttributeValue,
    SyntaxKind::AttributeModifier,
    SyntaxKind::NthSelectorArgument,
    SyntaxKind::NthSelectorFormula,
    SyntaxKind::NthSelectorOfSelectorList,
    SyntaxKind::RelativeSelectorList,
    SyntaxKind::RelativeSelector,
    SyntaxKind::LanguageSelectorArgument,
    SyntaxKind::LanguageTag,
    SyntaxKind::DirectionalitySelectorArgument,
    SyntaxKind::NamespacePrefix,
    SyntaxKind::FunctionRule,
    SyntaxKind::IfFunction,
    SyntaxKind::IfRule,
    SyntaxKind::ScssStylesheet,
    SyntaxKind::ScssUseRule,
    SyntaxKind::ScssForwardRule,
    SyntaxKind::ScssMixinDeclaration,
    SyntaxKind::ScssIncludeRule,
    SyntaxKind::ScssFunctionDeclaration,
    SyntaxKind::ScssReturnRule,
    SyntaxKind::ScssVariableDeclaration,
    SyntaxKind::ScssVariableReference,
    SyntaxKind::ScssPlaceholderSelector,
    SyntaxKind::ScssExtendRule,
    SyntaxKind::ScssControlIf,
    SyntaxKind::ScssControlElse,
    SyntaxKind::ScssControlEach,
    SyntaxKind::ScssControlFor,
    SyntaxKind::ScssControlWhile,
    SyntaxKind::ScssNestedProperty,
    SyntaxKind::ScssModuleConfig,
    SyntaxKind::SassIndentedBlock,
    SyntaxKind::SassIndentedRule,
    SyntaxKind::ScssAtRootRule,
    SyntaxKind::ScssErrorRule,
    SyntaxKind::ScssWarnRule,
    SyntaxKind::ScssDebugRule,
    SyntaxKind::ScssContentRule,
    SyntaxKind::ScssVariableFlag,
    SyntaxKind::LessStylesheet,
    SyntaxKind::LessVariableDeclaration,
    SyntaxKind::LessVariableReference,
    SyntaxKind::LessMixinDeclaration,
    SyntaxKind::LessMixinCall,
    SyntaxKind::LessMixinGuard,
    SyntaxKind::LessDetachedRulesetNode,
    SyntaxKind::LessExtendRule,
    SyntaxKind::LessNamespaceAccess,
    SyntaxKind::LessPropertyVariable,
    SyntaxKind::ScssMap,
    SyntaxKind::ScssMapEntry,
    SyntaxKind::ScssList,
    SyntaxKind::ScssCondition,
    SyntaxKind::LessCondition,
    SyntaxKind::BogusToken,
    SyntaxKind::BogusTrivia,
    SyntaxKind::BogusRule,
    SyntaxKind::BogusSelector,
    SyntaxKind::BogusSelectorList,
    SyntaxKind::BogusCompoundSelector,
    SyntaxKind::BogusCombinator,
    SyntaxKind::BogusDeclaration,
    SyntaxKind::BogusDeclarationList,
    SyntaxKind::BogusPropertyName,
    SyntaxKind::BogusValue,
    SyntaxKind::BogusValueList,
    SyntaxKind::BogusFunctionCall,
    SyntaxKind::BogusFunctionArguments,
    SyntaxKind::BogusAtRule,
    SyntaxKind::BogusMediaQuery,
    SyntaxKind::BogusSupportsCondition,
    SyntaxKind::BogusContainerCondition,
    SyntaxKind::BogusLayerName,
    SyntaxKind::BogusScopeRange,
    SyntaxKind::BogusKeyframeBlock,
    SyntaxKind::BogusCssModuleBlock,
    SyntaxKind::BogusComposesDeclaration,
    SyntaxKind::BogusComposesTarget,
    SyntaxKind::BogusFromClause,
    SyntaxKind::BogusInterpolation,
    SyntaxKind::BogusScssVariable,
    SyntaxKind::BogusScssMixin,
    SyntaxKind::BogusScssFunction,
    SyntaxKind::BogusScssControl,
    SyntaxKind::BogusSassIndentation,
    SyntaxKind::BogusLessVariable,
    SyntaxKind::BogusLessMixin,
    SyntaxKind::BogusLessGuard,
    SyntaxKind::BogusLessDetachedRuleset,
    SyntaxKind::BogusRecovery,
    SyntaxKind::BogusScssModuleConfig,
    SyntaxKind::BogusAtRulePrelude,
    SyntaxKind::BogusBracketedValue,
    SyntaxKind::BogusSimpleBlock,
    SyntaxKind::BogusScssMap,
    SyntaxKind::BogusScssMapEntry,
    SyntaxKind::BogusScssList,
    SyntaxKind::BogusScssCondition,
    SyntaxKind::BogusLessCondition,
];

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParsedTypedCst {
    root: SyntaxNode,
    source_text: String,
    errors: Vec<ParseError>,
}

impl ParsedTypedCst {
    pub fn from_parse_result(parsed: &ParseResult) -> Self {
        Self {
            root: parsed.syntax(),
            source_text: parsed.source_text().unwrap_or_default(),
            errors: parsed.errors().to_vec(),
        }
    }

    pub fn root(&self) -> SyntaxNode {
        self.root.clone()
    }

    pub fn source_text(&self) -> &str {
        self.source_text.as_str()
    }

    pub fn errors(&self) -> &[ParseError] {
        self.errors.as_slice()
    }

    pub fn stylesheet(&self) -> Option<StylesheetCstNode> {
        self.root
            .descendants()
            .find(|node| node.kind() == SyntaxKind::Stylesheet)
            .and_then(|node| StylesheetCstNode::cast(node.clone()))
    }

    pub fn to_data(&self) -> ParseTreeNodeV0 {
        node_to_data(&self.root, self.source_text(), self.errors(), true)
    }
}

pub trait TypedCstNode: Sized {
    const KIND: SyntaxKind;

    fn cast(syntax: SyntaxNode) -> Option<Self>;
    fn syntax(&self) -> &SyntaxNode;
    fn into_syntax(self) -> SyntaxNode;

    fn kind(&self) -> SyntaxKind {
        self.syntax().kind()
    }

    fn byte_span(&self) -> ParserByteSpanV0 {
        node_byte_span(self.syntax())
    }

    fn children(&self) -> Vec<TypedCstAnyNode> {
        typed_children(self.syntax())
    }

    fn grammar_row(&self) -> Option<NodeGrammarRow> {
        node_grammar_row(self.kind())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TypedCstAnyNode {
    syntax: SyntaxNode,
}

impl TypedCstAnyNode {
    pub fn cast(syntax: SyntaxNode) -> Option<Self> {
        node_grammar_row(syntax.kind()).map(|_| Self { syntax })
    }

    pub fn syntax(&self) -> &SyntaxNode {
        &self.syntax
    }

    pub fn kind(&self) -> SyntaxKind {
        self.syntax.kind()
    }

    pub fn into_syntax(self) -> SyntaxNode {
        self.syntax
    }

    pub fn child(&self, accessor_name: &str) -> Option<TypedCstAnyNode> {
        let row = node_grammar_row(self.kind())?;
        let accessor = row
            .named_child_accessors
            .iter()
            .find(|accessor| accessor.name == accessor_name)?;
        first_child_matching(&self.syntax, accessor.accepted_kinds)
    }

    pub fn token(&self, getter_name: &str) -> Option<SyntaxToken> {
        let row = node_grammar_row(self.kind())?;
        let getter = row
            .named_token_getters
            .iter()
            .find(|getter| getter.name == getter_name)?;
        first_token_matching(&self.syntax, getter.accepted_kinds)
    }

    pub fn children(&self) -> Vec<TypedCstAnyNode> {
        typed_children(&self.syntax)
    }
}

macro_rules! typed_cst_node {
    ($name:ident, $kind:expr) => {
        #[derive(Debug, Clone, PartialEq, Eq)]
        pub struct $name {
            syntax: SyntaxNode,
        }

        impl TypedCstNode for $name {
            const KIND: SyntaxKind = $kind;

            fn cast(syntax: SyntaxNode) -> Option<Self> {
                (syntax.kind() == Self::KIND).then_some(Self { syntax })
            }

            fn syntax(&self) -> &SyntaxNode {
                &self.syntax
            }

            fn into_syntax(self) -> SyntaxNode {
                self.syntax
            }
        }

        impl $name {
            pub fn as_any(&self) -> TypedCstAnyNode {
                TypedCstAnyNode {
                    syntax: self.syntax.clone(),
                }
            }
        }
    };
}

typed_cst_node!(StylesheetCstNode, SyntaxKind::Stylesheet);
typed_cst_node!(RuleCstNode, SyntaxKind::Rule);
typed_cst_node!(DeclarationCstNode, SyntaxKind::Declaration);
typed_cst_node!(ValueCstNode, SyntaxKind::Value);
typed_cst_node!(AtRulePreludeCstNode, SyntaxKind::AtRulePrelude);

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AtRuleCstNode {
    syntax: SyntaxNode,
}

impl TypedCstNode for AtRuleCstNode {
    const KIND: SyntaxKind = SyntaxKind::AtRule;

    fn cast(syntax: SyntaxNode) -> Option<Self> {
        is_at_rule_node_kind(syntax.kind()).then_some(Self { syntax })
    }

    fn syntax(&self) -> &SyntaxNode {
        &self.syntax
    }

    fn into_syntax(self) -> SyntaxNode {
        self.syntax
    }
}

impl AtRuleCstNode {
    pub fn name_token(&self) -> Option<SyntaxToken> {
        first_token_matching(&self.syntax, &[SyntaxKind::AtKeyword])
    }

    pub fn prelude(&self) -> Option<TypedCstAnyNode> {
        first_child_matching(&self.syntax, &[SyntaxKind::AtRulePrelude])
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BogusCstNode {
    syntax: SyntaxNode,
}

impl TypedCstNode for BogusCstNode {
    const KIND: SyntaxKind = SyntaxKind::BogusRecovery;

    fn cast(syntax: SyntaxNode) -> Option<Self> {
        syntax.kind().is_bogus().then_some(Self { syntax })
    }

    fn syntax(&self) -> &SyntaxNode {
        &self.syntax
    }

    fn into_syntax(self) -> SyntaxNode {
        self.syntax
    }
}

impl RuleCstNode {
    pub fn prelude(&self) -> Option<TypedCstAnyNode> {
        self.as_any().child("prelude")
    }

    pub fn block(&self) -> Option<TypedCstAnyNode> {
        self.as_any().child("block")
    }
}

impl DeclarationCstNode {
    pub fn value(&self) -> Option<TypedCstAnyNode> {
        self.as_any().child("value")
    }
}

pub fn parse_style_document_typed_v0(source: &str, dialect: StyleDialect) -> ParseTreeNodeV0 {
    let parsed = omena_parser::parse(source, dialect);
    parse_tree_data(&parsed)
}

pub fn parsed_typed_cst(parsed: &ParseResult) -> ParsedTypedCst {
    ParsedTypedCst::from_parse_result(parsed)
}

pub fn parse_tree_data(parsed: &ParseResult) -> ParseTreeNodeV0 {
    ParsedTypedCst::from_parse_result(parsed).to_data()
}

pub fn structural_manifest_rows() -> Vec<NodeGrammarRow> {
    STRUCTURAL_MANIFEST_KINDS
        .iter()
        .copied()
        .map(structural_node_grammar_row)
        .collect()
}

pub fn node_grammar_row(kind: SyntaxKind) -> Option<NodeGrammarRow> {
    STRUCTURAL_MANIFEST_KINDS
        .contains(&kind)
        .then(|| structural_node_grammar_row(kind))
}

fn structural_node_grammar_row(kind: SyntaxKind) -> NodeGrammarRow {
    debug_assert!(kind.is_node() || kind.is_bogus());
    NodeGrammarRow {
        kind,
        name: syntax_kind_name(kind),
        named_child_accessors: child_accessors_for(kind),
        named_token_getters: token_getters_for(kind),
    }
}

pub fn syntax_kind_name(kind: SyntaxKind) -> &'static str {
    SyntaxKind::ALL
        .iter()
        .position(|candidate| *candidate == kind)
        .and_then(|index| SYNTAX_KIND_NAMES.get(index).copied())
        .unwrap_or("UnknownSyntaxKind")
}

pub fn is_at_rule_node_kind(kind: SyntaxKind) -> bool {
    matches!(
        kind,
        SyntaxKind::AtRule
            | SyntaxKind::MediaRule
            | SyntaxKind::SupportsRule
            | SyntaxKind::ContainerRule
            | SyntaxKind::LayerRule
            | SyntaxKind::ScopeRule
            | SyntaxKind::KeyframesRule
            | SyntaxKind::FontFaceRule
            | SyntaxKind::PageRule
            | SyntaxKind::NamespaceRule
            | SyntaxKind::ImportRule
            | SyntaxKind::CharsetRule
            | SyntaxKind::PropertyRule
            | SyntaxKind::FunctionRule
            | SyntaxKind::StartingStyleRule
            | SyntaxKind::PageMarginRule
            | SyntaxKind::WhenRule
            | SyntaxKind::ElseRule
            | SyntaxKind::IfRule
            | SyntaxKind::CounterStyleRule
            | SyntaxKind::FontPaletteValuesRule
            | SyntaxKind::ColorProfileRule
            | SyntaxKind::PositionTryRule
            | SyntaxKind::FontFeatureValuesRule
            | SyntaxKind::FontFeatureValuesStylisticRule
            | SyntaxKind::FontFeatureValuesStylesetRule
            | SyntaxKind::FontFeatureValuesCharacterVariantRule
            | SyntaxKind::FontFeatureValuesSwashRule
            | SyntaxKind::FontFeatureValuesOrnamentsRule
            | SyntaxKind::FontFeatureValuesAnnotationRule
            | SyntaxKind::FontFeatureValuesHistoricalFormsRule
            | SyntaxKind::ViewTransitionRule
            | SyntaxKind::NestRule
            | SyntaxKind::CustomMediaRule
            | SyntaxKind::ScssUseRule
            | SyntaxKind::ScssForwardRule
            | SyntaxKind::ScssMixinDeclaration
            | SyntaxKind::ScssIncludeRule
            | SyntaxKind::ScssFunctionDeclaration
            | SyntaxKind::ScssReturnRule
            | SyntaxKind::ScssAtRootRule
            | SyntaxKind::ScssErrorRule
            | SyntaxKind::ScssWarnRule
            | SyntaxKind::ScssDebugRule
            | SyntaxKind::ScssContentRule
    )
}

const RULE_PRELUDE_KINDS: &[SyntaxKind] = &[
    SyntaxKind::SelectorList,
    SyntaxKind::Selector,
    SyntaxKind::AtRulePrelude,
    SyntaxKind::MediaQueryList,
    SyntaxKind::SupportsCondition,
    SyntaxKind::ContainerCondition,
    SyntaxKind::LayerName,
    SyntaxKind::ScopeRange,
];

const RULE_BLOCK_KINDS: &[SyntaxKind] = &[
    SyntaxKind::DeclarationList,
    SyntaxKind::RuleList,
    SyntaxKind::KeyframeBlock,
    SyntaxKind::SimpleBlock,
    SyntaxKind::SassIndentedBlock,
];

const DECLARATION_VALUE_KINDS: &[SyntaxKind] = &[
    SyntaxKind::Value,
    SyntaxKind::CustomPropertyValue,
    SyntaxKind::ValueList,
    SyntaxKind::ComponentValueList,
    SyntaxKind::BogusValue,
];

const AT_RULE_NAME_TOKEN_KINDS: &[SyntaxKind] = &[SyntaxKind::AtKeyword];

const RULE_CHILD_ACCESSORS: &[NamedChildAccessor] = &[
    NamedChildAccessor {
        name: "prelude",
        accepted_kinds: RULE_PRELUDE_KINDS,
    },
    NamedChildAccessor {
        name: "block",
        accepted_kinds: RULE_BLOCK_KINDS,
    },
];

const DECLARATION_CHILD_ACCESSORS: &[NamedChildAccessor] = &[NamedChildAccessor {
    name: "value",
    accepted_kinds: DECLARATION_VALUE_KINDS,
}];

const AT_RULE_CHILD_ACCESSORS: &[NamedChildAccessor] = &[NamedChildAccessor {
    name: "prelude",
    accepted_kinds: &[SyntaxKind::AtRulePrelude],
}];

const AT_RULE_TOKEN_GETTERS: &[NamedTokenGetter] = &[NamedTokenGetter {
    name: "nameToken",
    accepted_kinds: AT_RULE_NAME_TOKEN_KINDS,
}];

fn child_accessors_for(kind: SyntaxKind) -> &'static [NamedChildAccessor] {
    match kind {
        SyntaxKind::Rule | SyntaxKind::QualifiedRule => RULE_CHILD_ACCESSORS,
        SyntaxKind::Declaration
        | SyntaxKind::CustomPropertyDeclaration
        | SyntaxKind::CssModuleComposesDeclaration
        | SyntaxKind::ScssVariableDeclaration
        | SyntaxKind::LessVariableDeclaration => DECLARATION_CHILD_ACCESSORS,
        kind if is_at_rule_node_kind(kind) => AT_RULE_CHILD_ACCESSORS,
        _ => &[],
    }
}

fn token_getters_for(kind: SyntaxKind) -> &'static [NamedTokenGetter] {
    if is_at_rule_node_kind(kind) {
        AT_RULE_TOKEN_GETTERS
    } else {
        &[]
    }
}

fn typed_children(node: &SyntaxNode) -> Vec<TypedCstAnyNode> {
    node.children()
        .filter_map(|child| TypedCstAnyNode::cast(child.clone()))
        .collect::<Vec<_>>()
}

fn first_child_matching(node: &SyntaxNode, kinds: &[SyntaxKind]) -> Option<TypedCstAnyNode> {
    node.children()
        .find(|child| kinds.contains(&child.kind()))
        .and_then(|child| TypedCstAnyNode::cast(child.clone()))
}

fn first_token_matching(node: &SyntaxNode, kinds: &[SyntaxKind]) -> Option<SyntaxToken> {
    node.descendants_with_tokens()
        .filter_map(|element| element.into_token())
        .find(|token| kinds.contains(&token.kind()))
        .cloned()
}

fn node_to_data(
    node: &SyntaxNode,
    source_text: &str,
    errors: &[ParseError],
    is_root: bool,
) -> ParseTreeNodeV0 {
    let byte_span = node_byte_span(node);
    let children = node
        .children_with_tokens()
        .map(|element| element_to_data(element, source_text, errors))
        .collect();
    ParseTreeNodeV0 {
        kind: syntax_kind_name(node.kind()).to_string(),
        byte_span,
        range: parser_range_from_byte_range(source_text, byte_span),
        text: None,
        bogus: node.kind().is_bogus().then_some(true),
        error: error_for_range(errors, byte_span, is_root),
        children,
    }
}

fn element_to_data(
    element: SyntaxElementRef<'_>,
    source_text: &str,
    errors: &[ParseError],
) -> ParseTreeNodeV0 {
    if let Some(node) = element.into_node() {
        return node_to_data(node, source_text, errors, false);
    }
    if let Some(token) = element.into_token() {
        return token_to_data(token, source_text, errors);
    }
    ParseTreeNodeV0 {
        kind: "Unknown".to_string(),
        byte_span: ParserByteSpanV0::default(),
        range: ParserRangeV0::default(),
        text: None,
        bogus: Some(true),
        error: Some("unknown syntax element".to_string()),
        children: Vec::new(),
    }
}

fn token_to_data(token: &SyntaxToken, source_text: &str, errors: &[ParseError]) -> ParseTreeNodeV0 {
    let byte_span = token_byte_span(token);
    ParseTreeNodeV0 {
        kind: syntax_kind_name(token.kind()).to_string(),
        byte_span,
        range: parser_range_from_byte_range(source_text, byte_span),
        text: Some(token_text(token, source_text, byte_span)),
        bogus: token.kind().is_bogus().then_some(true),
        error: error_for_range(errors, byte_span, false),
        children: Vec::new(),
    }
}

fn token_text(token: &SyntaxToken, source_text: &str, byte_span: ParserByteSpanV0) -> String {
    if let Some(resolver) = token.resolver() {
        return token.resolve_text(&**resolver).to_string();
    }
    if let Some(static_text) = token.static_text() {
        return static_text.to_string();
    }
    source_text
        .get(byte_span.start..byte_span.end)
        .unwrap_or_default()
        .to_string()
}

fn error_for_range(
    errors: &[ParseError],
    byte_span: ParserByteSpanV0,
    is_root: bool,
) -> Option<String> {
    let exact = errors
        .iter()
        .filter(|error| error_byte_span(error) == byte_span)
        .map(error_message)
        .collect::<Vec<_>>();
    if !exact.is_empty() {
        return Some(exact.join("; "));
    }
    if is_root && !errors.is_empty() {
        return Some(
            errors
                .iter()
                .map(error_message)
                .collect::<Vec<_>>()
                .join("; "),
        );
    }
    None
}

fn error_message(error: &ParseError) -> String {
    format!("{}: {}", parse_error_code_name(error), error.message)
}

fn parse_error_code_name(error: &ParseError) -> &'static str {
    match error.code {
        omena_parser::ParseErrorCode::UnterminatedBlockComment => "UnterminatedBlockComment",
        omena_parser::ParseErrorCode::UnterminatedString => "UnterminatedString",
        omena_parser::ParseErrorCode::UnexpectedCharacter => "UnexpectedCharacter",
        omena_parser::ParseErrorCode::ExpectedSelectorName => "ExpectedSelectorName",
        omena_parser::ParseErrorCode::UnterminatedAttributeSelector => {
            "UnterminatedAttributeSelector"
        }
        omena_parser::ParseErrorCode::ExpectedValue => "ExpectedValue",
    }
}

fn node_byte_span(node: &SyntaxNode) -> ParserByteSpanV0 {
    let range = node.text_range();
    ParserByteSpanV0 {
        start: u32::from(range.start()) as usize,
        end: u32::from(range.end()) as usize,
    }
}

fn token_byte_span(token: &SyntaxToken) -> ParserByteSpanV0 {
    let range = token.text_range();
    ParserByteSpanV0 {
        start: u32::from(range.start()) as usize,
        end: u32::from(range.end()) as usize,
    }
}

fn error_byte_span(error: &ParseError) -> ParserByteSpanV0 {
    let range = error.range;
    ParserByteSpanV0 {
        start: u32::from(range.start()) as usize,
        end: u32::from(range.end()) as usize,
    }
}

fn parser_range_from_byte_range(source_text: &str, byte_span: ParserByteSpanV0) -> ParserRangeV0 {
    ParserRangeV0 {
        start: position_for_offset(source_text, byte_span.start),
        end: position_for_offset(source_text, byte_span.end),
    }
}

fn position_for_offset(source_text: &str, byte_offset: usize) -> ParserPositionV0 {
    let clamped = byte_offset.min(source_text.len());
    let mut line = 0;
    let mut character = 0;
    for (index, character_value) in source_text.char_indices() {
        if index >= clamped {
            break;
        }
        if character_value == '\n' {
            line += 1;
            character = 0;
        } else {
            character += 1;
        }
    }
    ParserPositionV0 { line, character }
}

#[cfg(test)]
fn data_descendant_count(node: &ParseTreeNodeV0) -> usize {
    1 + node
        .children
        .iter()
        .map(data_descendant_count)
        .sum::<usize>()
}

#[cfg(test)]
fn data_contains_bogus(node: &ParseTreeNodeV0) -> bool {
    node.bogus == Some(true) || node.children.iter().any(data_contains_bogus)
}

const SYNTAX_KIND_NAMES: &[&str] = &[
    "Whitespace",
    "LineComment",
    "BlockComment",
    "Ident",
    "Hash",
    "String",
    "BadString",
    "Url",
    "BadUrl",
    "Number",
    "Percentage",
    "Dimension",
    "UnicodeRange",
    "AtKeyword",
    "Delim",
    "Important",
    "Dot",
    "Comma",
    "Colon",
    "Semicolon",
    "LeftBrace",
    "RightBrace",
    "LeftParen",
    "RightParen",
    "LeftBracket",
    "RightBracket",
    "Plus",
    "Minus",
    "Star",
    "Slash",
    "Percent",
    "Equals",
    "Tilde",
    "Pipe",
    "Caret",
    "Dollar",
    "Ampersand",
    "GreaterThan",
    "LessThan",
    "PlusEquals",
    "MinusEquals",
    "StarEquals",
    "SlashEquals",
    "PipeEquals",
    "TildeEquals",
    "CaretEquals",
    "DollarEquals",
    "DoubleColon",
    "DoublePipe",
    "DoubleAmpersand",
    "Arrow",
    "IncludesMatch",
    "DashMatch",
    "PrefixMatch",
    "SuffixMatch",
    "SubstringMatch",
    "ColumnCombinator",
    "NestingSelector",
    "CustomPropertyName",
    "ClassName",
    "IdName",
    "KeywordAnd",
    "KeywordOr",
    "KeywordNot",
    "KeywordOnly",
    "KeywordFrom",
    "KeywordTo",
    "KeywordThrough",
    "KeywordImportant",
    "KeywordGlobal",
    "KeywordLocal",
    "KeywordExport",
    "KeywordImport",
    "KeywordComposes",
    "KeywordAs",
    "KeywordWith",
    "KeywordLayer",
    "KeywordSupports",
    "KeywordContainer",
    "KeywordScope",
    "KeywordMedia",
    "KeywordKeyframes",
    "KeywordCharset",
    "KeywordNamespace",
    "KeywordPage",
    "KeywordFontFace",
    "KeywordProperty",
    "KeywordStartingStyle",
    "KeywordWhen",
    "KeywordElse",
    "KeywordUse",
    "KeywordForward",
    "KeywordMixin",
    "KeywordInclude",
    "KeywordFunction",
    "KeywordReturn",
    "KeywordIf",
    "KeywordEach",
    "KeywordFor",
    "KeywordWhile",
    "KeywordIn",
    "Cdo",
    "Cdc",
    "ScssVariable",
    "ScssInterpolationStart",
    "ScssInterpolationEnd",
    "ScssSilentComment",
    "ScssPlaceholder",
    "ScssModuleNamespace",
    "SassIndentedNewline",
    "SassIndent",
    "SassDedent",
    "SassOptionalSemicolon",
    "LessVariable",
    "LessEscapedString",
    "LessDetachedRuleset",
    "LessMixinGuardWhen",
    "LessExtendKeyword",
    "LessNamespaceSeparator",
    "LessInterpolationStart",
    "LessInterpolationEnd",
    "LessPropertyVariableToken",
    "TemplateInterpolationStart",
    "TemplateInterpolationEnd",
    "TemplatePlaceholder",
    "Stylesheet",
    "Rule",
    "QualifiedRule",
    "Declaration",
    "DeclarationList",
    "RuleList",
    "SelectorList",
    "Selector",
    "ComplexSelector",
    "CompoundSelector",
    "ClassSelector",
    "IdSelector",
    "TypeSelector",
    "UniversalSelector",
    "AttributeSelector",
    "AttributeMatcher",
    "PseudoClassSelector",
    "PseudoElementSelector",
    "PseudoSelectorArgument",
    "NestingSelectorNode",
    "Combinator",
    "SelectorValue",
    "PropertyName",
    "CustomPropertyDeclaration",
    "Value",
    "ValueList",
    "FunctionCall",
    "FunctionArguments",
    "BinaryExpression",
    "UnaryExpression",
    "ParenthesizedExpression",
    "Interpolation",
    "DimensionValue",
    "ColorValue",
    "UrlValue",
    "VarFunction",
    "CalcFunction",
    "AtRule",
    "MediaRule",
    "SupportsRule",
    "ContainerRule",
    "LayerRule",
    "ScopeRule",
    "KeyframesRule",
    "KeyframeBlock",
    "FontFaceRule",
    "PageRule",
    "NamespaceRule",
    "ImportRule",
    "CharsetRule",
    "PropertyRule",
    "StartingStyleRule",
    "MediaQueryList",
    "MediaQuery",
    "MediaFeature",
    "SupportsCondition",
    "ContainerCondition",
    "LayerName",
    "ScopeRange",
    "CssModuleLocalBlock",
    "CssModuleGlobalBlock",
    "CssModuleExportBlock",
    "CssModuleImportBlock",
    "CssModuleComposesDeclaration",
    "CssModuleComposesTarget",
    "CssModuleFromClause",
    "TokenDefinition",
    "TokenReference",
    "Comment",
    "ErrorNode",
    "EnvFunction",
    "AttrFunction",
    "MathFunction",
    "PageMarginRule",
    "WhenRule",
    "ElseRule",
    "CounterStyleRule",
    "FontPaletteValuesRule",
    "ColorProfileRule",
    "PositionTryRule",
    "FontFeatureValuesRule",
    "FontFeatureValuesStylisticRule",
    "FontFeatureValuesStylesetRule",
    "FontFeatureValuesCharacterVariantRule",
    "FontFeatureValuesSwashRule",
    "FontFeatureValuesOrnamentsRule",
    "FontFeatureValuesAnnotationRule",
    "FontFeatureValuesHistoricalFormsRule",
    "ViewTransitionRule",
    "GradientFunction",
    "TransformFunction",
    "FilterFunction",
    "ImageFunction",
    "ShapeFunction",
    "AtRulePrelude",
    "NestRule",
    "CustomMediaRule",
    "IdentifierValue",
    "StringValue",
    "UnicodeRangeValue",
    "NumberValue",
    "PercentageValue",
    "BracketedValue",
    "ImportantAnnotation",
    "ComponentValue",
    "SimpleBlock",
    "ComponentValueList",
    "CommaSeparatedComponentValueList",
    "CustomPropertyValue",
    "AttributeName",
    "AttributeValue",
    "AttributeModifier",
    "NthSelectorArgument",
    "NthSelectorFormula",
    "NthSelectorOfSelectorList",
    "RelativeSelectorList",
    "RelativeSelector",
    "LanguageSelectorArgument",
    "LanguageTag",
    "DirectionalitySelectorArgument",
    "NamespacePrefix",
    "FunctionRule",
    "IfFunction",
    "IfRule",
    "ScssStylesheet",
    "ScssUseRule",
    "ScssForwardRule",
    "ScssMixinDeclaration",
    "ScssIncludeRule",
    "ScssFunctionDeclaration",
    "ScssReturnRule",
    "ScssVariableDeclaration",
    "ScssVariableReference",
    "ScssPlaceholderSelector",
    "ScssExtendRule",
    "ScssControlIf",
    "ScssControlElse",
    "ScssControlEach",
    "ScssControlFor",
    "ScssControlWhile",
    "ScssNestedProperty",
    "ScssModuleConfig",
    "SassIndentedBlock",
    "SassIndentedRule",
    "ScssAtRootRule",
    "ScssErrorRule",
    "ScssWarnRule",
    "ScssDebugRule",
    "ScssContentRule",
    "ScssVariableFlag",
    "LessStylesheet",
    "LessVariableDeclaration",
    "LessVariableReference",
    "LessMixinDeclaration",
    "LessMixinCall",
    "LessMixinGuard",
    "LessDetachedRulesetNode",
    "LessExtendRule",
    "LessNamespaceAccess",
    "LessPropertyVariable",
    "ScssMap",
    "ScssMapEntry",
    "ScssList",
    "ScssCondition",
    "LessCondition",
    "BogusToken",
    "BogusTrivia",
    "BogusRule",
    "BogusSelector",
    "BogusSelectorList",
    "BogusCompoundSelector",
    "BogusCombinator",
    "BogusDeclaration",
    "BogusDeclarationList",
    "BogusPropertyName",
    "BogusValue",
    "BogusValueList",
    "BogusFunctionCall",
    "BogusFunctionArguments",
    "BogusAtRule",
    "BogusMediaQuery",
    "BogusSupportsCondition",
    "BogusContainerCondition",
    "BogusLayerName",
    "BogusScopeRange",
    "BogusKeyframeBlock",
    "BogusCssModuleBlock",
    "BogusComposesDeclaration",
    "BogusComposesTarget",
    "BogusFromClause",
    "BogusInterpolation",
    "BogusScssVariable",
    "BogusScssMixin",
    "BogusScssFunction",
    "BogusScssControl",
    "BogusSassIndentation",
    "BogusLessVariable",
    "BogusLessMixin",
    "BogusLessGuard",
    "BogusLessDetachedRuleset",
    "BogusRecovery",
    "BogusScssModuleConfig",
    "BogusAtRulePrelude",
    "BogusBracketedValue",
    "BogusSimpleBlock",
    "BogusScssMap",
    "BogusScssMapEntry",
    "BogusScssList",
    "BogusScssCondition",
    "BogusLessCondition",
    "Root",
    "Eof",
    "Unknown",
    "Tombstone",
];

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::BTreeSet;

    #[test]
    fn syntax_kind_names_cover_all_current_kinds() {
        assert_eq!(SYNTAX_KIND_NAMES.len(), SyntaxKind::ALL.len());
        for kind in SyntaxKind::ALL {
            assert_ne!(syntax_kind_name(*kind), "UnknownSyntaxKind");
        }
    }

    #[test]
    fn syntax_kind_name_binds_native_if_rule_positionally() {
        assert_eq!(syntax_kind_name(SyntaxKind::IfRule), "IfRule");
        assert_eq!(
            syntax_kind_name(SyntaxKind::ScssStylesheet),
            "ScssStylesheet"
        );
    }

    #[test]
    fn manifest_covers_every_structural_kind() {
        let expected = SyntaxKind::ALL
            .iter()
            .copied()
            .filter(|kind| kind.is_node() || kind.is_bogus())
            .collect::<BTreeSet<_>>();
        let actual = structural_manifest_rows()
            .into_iter()
            .map(|row| row.kind)
            .collect::<BTreeSet<_>>();
        assert_eq!(actual.len(), STRUCTURAL_MANIFEST_KINDS.len());
        assert_eq!(actual, expected);
    }

    #[test]
    fn typed_accessors_borrow_existing_nodes() {
        let parsed = omena_parser::parse(".card { color: red; }", StyleDialect::Css);
        let cst = ParsedTypedCst::from_parse_result(&parsed);
        let stylesheet = cst.stylesheet();
        assert!(stylesheet.is_some(), "stylesheet wrapper should cast");
        let Some(stylesheet) = stylesheet else {
            return;
        };
        let rule = match stylesheet
            .children()
            .into_iter()
            .find(|child| child.kind() == SyntaxKind::Rule)
            .and_then(|child| RuleCstNode::cast(child.into_syntax()))
        {
            Some(rule) => rule,
            None => {
                assert!(
                    stylesheet
                        .children()
                        .iter()
                        .any(|child| child.kind() == SyntaxKind::Rule)
                );
                return;
            }
        };
        let block = rule.block();
        assert!(block.is_some(), "rule block should be exposed");
        let Some(block) = block else {
            return;
        };
        let direct_block = rule
            .syntax()
            .children()
            .find(|child| child.kind() == SyntaxKind::DeclarationList);
        assert!(direct_block.is_some());
        if let Some(direct_block) = direct_block {
            assert_eq!(block.syntax().text_range(), direct_block.text_range());
        }
    }

    #[test]
    fn declaration_value_accessor_exposes_value_child() {
        let parsed = omena_parser::parse(".card { color: red; }", StyleDialect::Css);
        let declaration = parsed
            .syntax()
            .descendants()
            .find(|node| node.kind() == SyntaxKind::Declaration)
            .and_then(|node| DeclarationCstNode::cast(node.clone()));
        assert!(declaration.is_some());
        if let Some(declaration) = declaration {
            let value = declaration.value();
            assert!(value.is_some());
            if let Some(value) = value {
                assert_eq!(value.kind(), SyntaxKind::Value);
            }
        }
    }

    #[test]
    fn parse_tree_data_preserves_complete_tree_and_bogus_channel() {
        let parsed = omena_parser::parse(".card { color: ; }", StyleDialect::Css);
        let data = parse_tree_data(&parsed);
        let syntax = parsed.syntax();
        let expected_count = syntax.descendants_with_tokens().count();
        assert_eq!(data_descendant_count(&data), expected_count);
        assert!(data_contains_bogus(&data));
        assert!(data.error.is_some());

        let serialized = serde_json::to_string(&data);
        assert!(serialized.is_ok());
        if let Ok(serialized) = serialized {
            let value = serde_json::from_str::<serde_json::Value>(&serialized);
            assert!(value.is_ok());
            if let Ok(value) = value {
                let reserialized = serde_json::to_string(&value);
                assert!(reserialized.is_ok());
                if let Ok(reserialized) = reserialized {
                    assert_eq!(serialized, reserialized);
                }
            }
        }
    }

    #[test]
    fn at_rule_name_token_is_available() {
        let parsed =
            omena_parser::parse("@media screen { .card { color: red; } }", StyleDialect::Css);
        let at_rule = parsed
            .syntax()
            .descendants()
            .find_map(|node| AtRuleCstNode::cast(node.clone()));
        assert!(at_rule.is_some());
        if let Some(at_rule) = at_rule {
            let token = at_rule.name_token();
            assert!(token.is_some());
            if let Some(token) = token {
                assert_eq!(token.kind(), SyntaxKind::AtKeyword);
            }
        }
    }
}

//! Green-field parser substrate for omena-css.
//!
//! This crate is intentionally built next to `engine-style-parser`. It owns the
//! future cstree parser track, but it does not replace the current product path
//! until parser parity gates are met.

use cstree::{
    build::GreenNodeBuilder,
    green::GreenNode,
    interning::TokenInterner,
    syntax::SyntaxNode,
    text::{TextRange, TextSize},
};
use omena_interner::NameKind;
pub use omena_syntax::StyleDialect;
use omena_syntax::SyntaxKind;
use std::{collections::BTreeSet, sync::Arc};

#[derive(Debug, Clone)]
pub struct ParseResult {
    green: GreenNode,
    interner: Option<Arc<TokenInterner>>,
    errors: Vec<ParseError>,
    token_count: usize,
    dialect: StyleDialect,
}

impl PartialEq for ParseResult {
    fn eq(&self, other: &Self) -> bool {
        self.green == other.green
            && self.errors == other.errors
            && self.token_count == other.token_count
            && self.dialect == other.dialect
    }
}

impl Eq for ParseResult {}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LexResult {
    tokens: Vec<LexedToken>,
    errors: Vec<ParseError>,
    dialect: StyleDialect,
}

impl LexResult {
    pub fn tokens(&self) -> &[LexedToken] {
        &self.tokens
    }

    pub fn errors(&self) -> &[ParseError] {
        &self.errors
    }

    pub fn dialect(&self) -> StyleDialect {
        self.dialect
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LexedToken {
    pub kind: SyntaxKind,
    pub range: TextRange,
    pub text: String,
}

impl ParseResult {
    pub fn green(&self) -> &GreenNode {
        &self.green
    }

    pub fn syntax(&self) -> SyntaxNode<SyntaxKind> {
        if let Some(interner) = &self.interner {
            return SyntaxNode::new_root_with_resolver(self.green.clone(), Arc::clone(interner))
                .syntax()
                .clone();
        }
        SyntaxNode::new_root(self.green.clone())
    }

    pub fn source_text(&self) -> Option<String> {
        let syntax = self.syntax();
        syntax
            .try_resolved()
            .map(|resolved| resolved.text().to_string())
    }

    pub fn errors(&self) -> &[ParseError] {
        &self.errors
    }

    pub fn token_count(&self) -> usize {
        self.token_count
    }

    pub fn dialect(&self) -> StyleDialect {
        self.dialect
    }

    pub fn cst(&self) -> ParsedCst {
        ParsedCst::new(self.syntax())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParseError {
    pub code: ParseErrorCode,
    pub range: TextRange,
    pub message: &'static str,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ParseErrorCode {
    UnterminatedBlockComment,
    UnterminatedString,
    UnexpectedCharacter,
    ExpectedSelectorName,
    UnterminatedAttributeSelector,
    ExpectedValue,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ParseEntryPoint {
    Stylesheet,
    RuleList,
    Rule,
    DeclarationList,
    Declaration,
    Value,
    ComponentValue,
    ComponentValueList,
    CommaSeparatedComponentValueList,
    SimpleBlock,
}

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
pub struct ParsedStyleFacts {
    pub product: &'static str,
    pub dialect: StyleDialect,
    pub selector_count: usize,
    pub selectors: Vec<ParsedSelectorFact>,
    pub variable_count: usize,
    pub variables: Vec<ParsedVariableFact>,
    pub sass_symbol_count: usize,
    pub sass_symbols: Vec<ParsedSassSymbolFact>,
    pub animation_count: usize,
    pub animations: Vec<ParsedAnimationFact>,
    pub css_module_value_count: usize,
    pub css_module_values: Vec<ParsedCssModuleValueFact>,
    pub css_module_value_import_edge_count: usize,
    pub css_module_value_import_edges: Vec<ParsedCssModuleValueImportEdgeFact>,
    pub css_module_value_definition_edge_count: usize,
    pub css_module_value_definition_edges: Vec<ParsedCssModuleValueDefinitionEdgeFact>,
    pub css_module_composes_count: usize,
    pub css_module_composes: Vec<ParsedCssModuleComposesFact>,
    pub css_module_composes_edge_count: usize,
    pub css_module_composes_edges: Vec<ParsedCssModuleComposesEdgeFact>,
    pub icss_count: usize,
    pub icss: Vec<ParsedIcssFact>,
    pub icss_import_edge_count: usize,
    pub icss_import_edges: Vec<ParsedIcssImportEdgeFact>,
    pub icss_export_edge_count: usize,
    pub icss_export_edges: Vec<ParsedIcssExportEdgeFact>,
    pub at_rule_count: usize,
    pub at_rules: Vec<ParsedAtRuleFact>,
    pub error_count: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParsedSelectorFact {
    pub kind: ParsedSelectorFactKind,
    pub name: String,
    pub range: TextRange,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum ParsedSelectorFactKind {
    Class,
    Id,
    Placeholder,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParsedVariableFact {
    pub kind: ParsedVariableFactKind,
    pub name: String,
    pub range: TextRange,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ParsedVariableFactKind {
    ScssDeclaration,
    ScssReference,
    LessDeclaration,
    LessReference,
    CustomPropertyDeclaration,
    CustomPropertyReference,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParsedSassSymbolFact {
    pub kind: ParsedSassSymbolFactKind,
    pub symbol_kind: &'static str,
    pub name: String,
    pub role: &'static str,
    pub range: TextRange,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum ParsedSassSymbolFactKind {
    VariableDeclaration,
    VariableReference,
    MixinDeclaration,
    MixinInclude,
    FunctionDeclaration,
    FunctionCall,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParsedAnimationFact {
    pub kind: ParsedAnimationFactKind,
    pub name: String,
    pub range: TextRange,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum ParsedAnimationFactKind {
    KeyframesDeclaration,
    AnimationNameReference,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParsedCssModuleValueFact {
    pub kind: ParsedCssModuleValueFactKind,
    pub name: String,
    pub range: TextRange,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum ParsedCssModuleValueFactKind {
    Definition,
    Reference,
    ImportSource,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParsedCssModuleValueImportEdgeFact {
    pub remote_name: String,
    pub local_name: String,
    pub import_source: String,
    pub range: TextRange,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParsedCssModuleValueDefinitionEdgeFact {
    pub definition_name: String,
    pub reference_names: Vec<String>,
    pub range: TextRange,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParsedCssModuleComposesFact {
    pub kind: ParsedCssModuleComposesFactKind,
    pub name: String,
    pub range: TextRange,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum ParsedCssModuleComposesFactKind {
    Target,
    ImportSource,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParsedCssModuleComposesEdgeFact {
    pub kind: ParsedCssModuleComposesEdgeKind,
    pub owner_selector_names: Vec<String>,
    pub target_names: Vec<String>,
    pub import_source: Option<String>,
    pub range: TextRange,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum ParsedCssModuleComposesEdgeKind {
    Local,
    Global,
    External,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParsedIcssFact {
    pub kind: ParsedIcssFactKind,
    pub name: String,
    pub range: TextRange,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum ParsedIcssFactKind {
    ExportName,
    ImportLocalName,
    ImportRemoteName,
    ImportSource,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParsedIcssImportEdgeFact {
    pub local_name: String,
    pub remote_name: String,
    pub import_source: String,
    pub range: TextRange,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParsedIcssExportEdgeFact {
    pub export_name: String,
    pub reference_names: Vec<String>,
    pub range: TextRange,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParsedAtRuleFact {
    pub name: String,
    pub node_kind: Option<SyntaxKind>,
    pub range: TextRange,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParsedCst {
    root: SyntaxNode<SyntaxKind>,
}

impl ParsedCst {
    pub fn new(root: SyntaxNode<SyntaxKind>) -> Self {
        Self { root }
    }

    pub fn root(&self) -> &SyntaxNode<SyntaxKind> {
        &self.root
    }

    pub fn stylesheet(&self) -> Option<StylesheetCstNode> {
        self.first_node(StylesheetCstNode::cast)
    }

    pub fn rules(&self) -> Vec<RuleCstNode> {
        self.nodes(RuleCstNode::cast)
    }

    pub fn selectors(&self) -> Vec<SelectorCstNode> {
        self.nodes(SelectorCstNode::cast)
    }

    pub fn declarations(&self) -> Vec<DeclarationCstNode> {
        self.nodes(DeclarationCstNode::cast)
    }

    pub fn declaration_lists(&self) -> Vec<DeclarationListCstNode> {
        self.nodes(DeclarationListCstNode::cast)
    }

    pub fn values(&self) -> Vec<ValueCstNode> {
        self.nodes(ValueCstNode::cast)
    }

    pub fn component_values(&self) -> Vec<ComponentValueCstNode> {
        self.nodes(ComponentValueCstNode::cast)
    }

    pub fn simple_blocks(&self) -> Vec<SimpleBlockCstNode> {
        self.nodes(SimpleBlockCstNode::cast)
    }

    pub fn component_value_lists(&self) -> Vec<ComponentValueListCstNode> {
        self.nodes(ComponentValueListCstNode::cast)
    }

    pub fn comma_separated_component_value_lists(
        &self,
    ) -> Vec<CommaSeparatedComponentValueListCstNode> {
        self.nodes(CommaSeparatedComponentValueListCstNode::cast)
    }

    pub fn custom_property_values(&self) -> Vec<CustomPropertyValueCstNode> {
        self.nodes(CustomPropertyValueCstNode::cast)
    }

    pub fn at_rules(&self) -> Vec<AtRuleCstNode> {
        self.nodes(AtRuleCstNode::cast)
    }

    pub fn bogus_nodes(&self) -> Vec<BogusCstNode> {
        self.nodes(BogusCstNode::cast)
    }

    pub fn has_bogus_nodes(&self) -> bool {
        self.first_node(BogusCstNode::cast).is_some()
    }

    fn first_node<T>(&self, cast: impl Fn(SyntaxNode<SyntaxKind>) -> Option<T>) -> Option<T> {
        let mut nodes = Vec::new();
        collect_typed_nodes(&self.root, &cast, &mut nodes);
        nodes.into_iter().next()
    }

    fn nodes<T>(&self, cast: impl Fn(SyntaxNode<SyntaxKind>) -> Option<T>) -> Vec<T> {
        let mut nodes = Vec::new();
        collect_typed_nodes(&self.root, &cast, &mut nodes);
        nodes
    }
}

pub trait TypedCstNode: Sized {
    fn cast(syntax: SyntaxNode<SyntaxKind>) -> Option<Self>;
    fn syntax(&self) -> &SyntaxNode<SyntaxKind>;

    fn kind(&self) -> SyntaxKind {
        self.syntax().kind()
    }

    fn text_range(&self) -> TextRange {
        self.syntax().text_range()
    }

    fn into_syntax(self) -> SyntaxNode<SyntaxKind>;
}

macro_rules! typed_cst_node {
    ($name:ident, $kind:expr) => {
        #[derive(Debug, Clone, PartialEq, Eq)]
        pub struct $name {
            syntax: SyntaxNode<SyntaxKind>,
        }

        impl $name {
            pub const KIND: SyntaxKind = $kind;
        }

        impl TypedCstNode for $name {
            fn cast(syntax: SyntaxNode<SyntaxKind>) -> Option<Self> {
                (syntax.kind() == Self::KIND).then_some(Self { syntax })
            }

            fn syntax(&self) -> &SyntaxNode<SyntaxKind> {
                &self.syntax
            }

            fn into_syntax(self) -> SyntaxNode<SyntaxKind> {
                self.syntax
            }
        }
    };
}

typed_cst_node!(StylesheetCstNode, SyntaxKind::Stylesheet);
typed_cst_node!(RuleCstNode, SyntaxKind::Rule);
typed_cst_node!(SelectorCstNode, SyntaxKind::Selector);
typed_cst_node!(DeclarationCstNode, SyntaxKind::Declaration);
typed_cst_node!(DeclarationListCstNode, SyntaxKind::DeclarationList);
typed_cst_node!(ValueCstNode, SyntaxKind::Value);
typed_cst_node!(ComponentValueCstNode, SyntaxKind::ComponentValue);
typed_cst_node!(SimpleBlockCstNode, SyntaxKind::SimpleBlock);
typed_cst_node!(ComponentValueListCstNode, SyntaxKind::ComponentValueList);
typed_cst_node!(
    CommaSeparatedComponentValueListCstNode,
    SyntaxKind::CommaSeparatedComponentValueList
);
typed_cst_node!(CustomPropertyValueCstNode, SyntaxKind::CustomPropertyValue);

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AtRuleCstNode {
    syntax: SyntaxNode<SyntaxKind>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BogusCstNode {
    syntax: SyntaxNode<SyntaxKind>,
}

impl TypedCstNode for AtRuleCstNode {
    fn cast(syntax: SyntaxNode<SyntaxKind>) -> Option<Self> {
        is_at_rule_node_kind(syntax.kind()).then_some(Self { syntax })
    }

    fn syntax(&self) -> &SyntaxNode<SyntaxKind> {
        &self.syntax
    }

    fn into_syntax(self) -> SyntaxNode<SyntaxKind> {
        self.syntax
    }
}

impl TypedCstNode for BogusCstNode {
    fn cast(syntax: SyntaxNode<SyntaxKind>) -> Option<Self> {
        syntax.kind().is_bogus().then_some(Self { syntax })
    }

    fn syntax(&self) -> &SyntaxNode<SyntaxKind> {
        &self.syntax
    }

    fn into_syntax(self) -> SyntaxNode<SyntaxKind> {
        self.syntax
    }
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
            | SyntaxKind::StartingStyleRule
            | SyntaxKind::PageMarginRule
            | SyntaxKind::WhenRule
            | SyntaxKind::ElseRule
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

fn collect_typed_nodes<T>(
    node: &SyntaxNode<SyntaxKind>,
    cast: &impl Fn(SyntaxNode<SyntaxKind>) -> Option<T>,
    nodes: &mut Vec<T>,
) {
    if let Some(typed) = cast(node.clone()) {
        nodes.push(typed);
    }
    for child in node.children() {
        collect_typed_nodes(child, cast, nodes);
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TokenSet {
    kinds: &'static [SyntaxKind],
}

impl TokenSet {
    pub const fn new(kinds: &'static [SyntaxKind]) -> Self {
        Self { kinds }
    }

    pub fn contains(self, kind: SyntaxKind) -> bool {
        self.kinds.contains(&kind)
    }

    pub fn len(self) -> usize {
        self.kinds.len()
    }

    pub fn is_empty(self) -> bool {
        self.kinds.is_empty()
    }
}

pub const RECOVERY_TOP: TokenSet = TokenSet::new(&[
    SyntaxKind::AtKeyword,
    SyntaxKind::Dot,
    SyntaxKind::Hash,
    SyntaxKind::RightBrace,
    SyntaxKind::Semicolon,
]);

pub const RECOVERY_DECLARATION: TokenSet =
    TokenSet::new(&[SyntaxKind::Semicolon, SyntaxKind::RightBrace]);

pub const RECOVERY_SELECTOR: TokenSet = TokenSet::new(&[
    SyntaxKind::Comma,
    SyntaxKind::LeftBrace,
    SyntaxKind::RightBrace,
]);

pub trait DialectExtension {
    fn dialect(&self) -> StyleDialect;

    fn classify_variable_token(&self, text: &str) -> Option<SyntaxKind> {
        match self.dialect() {
            StyleDialect::Css => None,
            StyleDialect::Scss | StyleDialect::Sass if text.starts_with('$') => {
                Some(SyntaxKind::ScssVariable)
            }
            StyleDialect::Less if text.starts_with('@') => Some(SyntaxKind::LessVariable),
            StyleDialect::Scss | StyleDialect::Sass | StyleDialect::Less => None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BuiltinDialectExtension {
    dialect: StyleDialect,
}

impl BuiltinDialectExtension {
    pub const fn new(dialect: StyleDialect) -> Self {
        Self { dialect }
    }
}

impl DialectExtension for BuiltinDialectExtension {
    fn dialect(&self) -> StyleDialect {
        self.dialect
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Token<'text> {
    kind: SyntaxKind,
    text: &'text str,
    range: TextRange,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct AtRuleSpec {
    node_kind: SyntaxKind,
    block_kind: AtRuleBlockKind,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum AtRuleBlockKind {
    GroupRuleList,
    DeclarationList,
    Keyframes,
    Raw,
}

pub fn parse(text: &str, dialect: StyleDialect) -> ParseResult {
    parse_entry_point(text, dialect, ParseEntryPoint::Stylesheet)
}

pub fn parse_entry_point(
    text: &str,
    dialect: StyleDialect,
    entry_point: ParseEntryPoint,
) -> ParseResult {
    let extension = BuiltinDialectExtension::new(dialect);
    parse_entry_point_with_extension(text, &extension, entry_point)
}

pub fn lex(text: &str, dialect: StyleDialect) -> LexResult {
    let extension = BuiltinDialectExtension::new(dialect);
    lex_with_extension(text, &extension)
}

pub fn lex_with_extension(text: &str, extension: &impl DialectExtension) -> LexResult {
    let (tokens, errors) = tokenize(text, extension);
    LexResult {
        tokens: tokens
            .into_iter()
            .map(|token| LexedToken {
                kind: token.kind,
                range: token.range,
                text: public_token_text(token.text),
            })
            .collect(),
        errors,
        dialect: extension.dialect(),
    }
}

pub fn parse_with_extension(text: &str, extension: &impl DialectExtension) -> ParseResult {
    parse_entry_point_with_extension(text, extension, ParseEntryPoint::Stylesheet)
}

pub fn parse_entry_point_with_extension(
    text: &str,
    extension: &impl DialectExtension,
    entry_point: ParseEntryPoint,
) -> ParseResult {
    let (tokens, errors) = tokenize(text, extension);
    let token_count = tokens.len();
    let mut parser = Parser::new(tokens, errors, extension.dialect());
    let (green, interner) = parser.parse_entry_point(entry_point);

    ParseResult {
        green,
        interner,
        errors: parser.into_errors(),
        token_count,
        dialect: extension.dialect(),
    }
}

pub fn collect_style_facts(text: &str, dialect: StyleDialect) -> ParsedStyleFacts {
    let extension = BuiltinDialectExtension::new(dialect);
    collect_style_facts_with_extension(text, &extension)
}

pub fn collect_style_facts_with_extension(
    text: &str,
    extension: &impl DialectExtension,
) -> ParsedStyleFacts {
    let (tokens, lex_errors) = tokenize(text, extension);
    let mut parser = Parser::new(tokens.clone(), lex_errors, extension.dialect());
    let _green = parser.parse();
    let errors = parser.into_errors();
    let selectors = collect_selector_facts_from_tokens(&tokens);
    let variables = collect_variable_facts_from_tokens(&tokens);
    let sass_symbols = collect_sass_symbol_facts_from_tokens(&tokens);
    let animations = collect_animation_facts_from_tokens(&tokens);
    let css_module_values = collect_css_module_value_facts_from_tokens(&tokens);
    let css_module_value_import_edges =
        collect_css_module_value_import_edge_facts_from_tokens(&tokens);
    let css_module_value_definition_edges =
        collect_css_module_value_definition_edge_facts_from_tokens(&tokens);
    let css_module_composes = collect_css_module_composes_facts_from_tokens(&tokens);
    let css_module_composes_edges = collect_css_module_composes_edge_facts_from_tokens(&tokens);
    let icss = collect_icss_facts_from_tokens(&tokens);
    let icss_import_edges = collect_icss_import_edge_facts_from_tokens(&tokens);
    let icss_export_edges = collect_icss_export_edge_facts_from_tokens(&tokens);
    let at_rules = collect_at_rule_facts_from_tokens(&tokens, extension.dialect());

    ParsedStyleFacts {
        product: "omena-parser.style-facts",
        dialect: extension.dialect(),
        selector_count: selectors.len(),
        selectors,
        variable_count: variables.len(),
        variables,
        sass_symbol_count: sass_symbols.len(),
        sass_symbols,
        animation_count: animations.len(),
        animations,
        css_module_value_count: css_module_values.len(),
        css_module_values,
        css_module_value_import_edge_count: css_module_value_import_edges.len(),
        css_module_value_import_edges,
        css_module_value_definition_edge_count: css_module_value_definition_edges.len(),
        css_module_value_definition_edges,
        css_module_composes_count: css_module_composes.len(),
        css_module_composes,
        css_module_composes_edge_count: css_module_composes_edges.len(),
        css_module_composes_edges,
        icss_count: icss.len(),
        icss,
        icss_import_edge_count: icss_import_edges.len(),
        icss_import_edges,
        icss_export_edge_count: icss_export_edges.len(),
        icss_export_edges,
        at_rule_count: at_rules.len(),
        at_rules,
        error_count: errors.len(),
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
            "selectorCstSkeleton",
            "atRuleRegistrySkeleton",
            "prattValueExpressionSkeleton",
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
            "cssModuleComposesStyleFacts",
            "icssStyleFacts",
            "animationNameStyleFacts",
            "animationShorthandStyleFacts",
            "scssStructuredBlockAtRules",
            "scssControlPreludeValidation",
            "scssControlStyleFactExtraction",
            "scssIncludeContentBlockStyleFacts",
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
        ],
        not_ready_surfaces: vec![
            "fullRecursiveDescentGrammar",
            "fullPrattValueParser",
            "differentialCorpus",
            "productCutover",
        ],
    }
}

fn tokenize<'text>(
    text: &'text str,
    extension: &impl DialectExtension,
) -> (Vec<Token<'text>>, Vec<ParseError>) {
    let mut tokenizer = Tokenizer::new(text, extension);
    tokenizer.tokenize();
    (tokenizer.tokens, tokenizer.errors)
}

struct Tokenizer<'text, 'extension, E> {
    text: &'text str,
    extension: &'extension E,
    offset: usize,
    scss_interpolation_depth: usize,
    less_interpolation_depth: usize,
    sass_indent_stack: Vec<usize>,
    tokens: Vec<Token<'text>>,
    errors: Vec<ParseError>,
}

struct Parser<'text> {
    tokens: Vec<Token<'text>>,
    position: usize,
    dialect: StyleDialect,
    builder: GreenNodeBuilder<'static, 'static, SyntaxKind>,
    errors: Vec<ParseError>,
}

impl<'text> Parser<'text> {
    fn new(tokens: Vec<Token<'text>>, errors: Vec<ParseError>, dialect: StyleDialect) -> Self {
        Self {
            tokens,
            position: 0,
            dialect,
            builder: GreenNodeBuilder::new(),
            errors,
        }
    }

    fn parse(&mut self) -> (GreenNode, Option<Arc<TokenInterner>>) {
        self.parse_entry_point(ParseEntryPoint::Stylesheet)
    }

    fn parse_entry_point(
        &mut self,
        entry_point: ParseEntryPoint,
    ) -> (GreenNode, Option<Arc<TokenInterner>>) {
        self.builder.start_node(SyntaxKind::Root);
        match entry_point {
            ParseEntryPoint::Stylesheet => {
                self.builder.start_node(SyntaxKind::Stylesheet);
                self.parse_stylesheet_items();
                self.builder.finish_node();
            }
            ParseEntryPoint::RuleList => {
                self.builder.start_node(SyntaxKind::RuleList);
                self.parse_rule_list_items();
                self.builder.finish_node();
            }
            ParseEntryPoint::Rule => self.parse_rule(),
            ParseEntryPoint::DeclarationList => {
                self.builder.start_node(SyntaxKind::DeclarationList);
                self.parse_declaration_list();
                self.builder.finish_node();
            }
            ParseEntryPoint::Declaration => self.parse_declaration(),
            ParseEntryPoint::Value => {
                self.builder.start_node(SyntaxKind::Value);
                self.parse_value_or_value_list_until(&[]);
                self.builder.finish_node();
            }
            ParseEntryPoint::ComponentValue => self.parse_component_value(&[]),
            ParseEntryPoint::ComponentValueList => self.parse_component_value_list_until(&[]),
            ParseEntryPoint::CommaSeparatedComponentValueList => {
                self.parse_comma_separated_component_value_list_until(&[])
            }
            ParseEntryPoint::SimpleBlock => self.parse_simple_block_entry_point(&[]),
        }
        self.parse_sass_indentation_bogus();
        self.parse_entry_point_trailing_bogus();
        self.builder.finish_node();

        let builder = std::mem::take(&mut self.builder);
        let (green, cache) = builder.finish();
        let interner = cache.and_then(|cache| cache.into_interner()).map(Arc::new);
        (green, interner)
    }

    fn parse_sass_indentation_bogus(&mut self) {
        if self.dialect != StyleDialect::Sass
            || !self
                .errors
                .iter()
                .any(|error| error.message == "inconsistent Sass indentation")
        {
            return;
        }
        self.builder.start_node(SyntaxKind::BogusSassIndentation);
        self.builder.finish_node();
    }

    fn parse_entry_point_trailing_bogus(&mut self) {
        self.eat_trivia();
        if self.at_end() {
            return;
        }
        self.builder.start_node(SyntaxKind::BogusRecovery);
        while !self.at_end() {
            self.token_current();
        }
        self.builder.finish_node();
    }

    fn into_errors(self) -> Vec<ParseError> {
        self.errors
    }

    fn parse_stylesheet_items(&mut self) {
        while !self.at_end() {
            self.eat_trivia();
            if self.at_end() {
                break;
            }
            match self.current_kind() {
                Some(SyntaxKind::AtKeyword) if self.current_is_css_module_value_rule() => {
                    self.parse_css_module_value_rule()
                }
                Some(SyntaxKind::AtKeyword) if self.current_dialect_at_rule_spec().is_some() => {
                    self.parse_dialect_at_rule()
                }
                Some(SyntaxKind::AtKeyword) => self.parse_at_rule(),
                Some(SyntaxKind::ScssVariable)
                    if matches!(self.dialect, StyleDialect::Scss | StyleDialect::Sass) =>
                {
                    self.parse_variable_declaration(SyntaxKind::ScssVariableDeclaration)
                }
                Some(SyntaxKind::LessVariable) if self.dialect == StyleDialect::Less => {
                    self.parse_variable_declaration(SyntaxKind::LessVariableDeclaration)
                }
                Some(SyntaxKind::Cdo | SyntaxKind::Cdc) => self.token_current(),
                Some(SyntaxKind::RightBrace | SyntaxKind::SassDedent) => self.token_current(),
                Some(SyntaxKind::Semicolon | SyntaxKind::SassOptionalSemicolon) => {
                    self.token_current()
                }
                Some(_) => self.parse_rule(),
                None => break,
            }
        }
    }

    fn parse_rule(&mut self) {
        let starts_less_mixin =
            self.dialect == StyleDialect::Less && self.current_starts_less_callable_signature();
        let has_rule_block = self.find_rule_block_open_before_recovery(&[
            SyntaxKind::Semicolon,
            SyntaxKind::SassOptionalSemicolon,
            SyntaxKind::RightBrace,
            SyntaxKind::SassDedent,
        ]);
        let kind = if let Some(kind) = self
            .current_icss_module_rule_kind()
            .filter(|_| has_rule_block)
        {
            kind
        } else if self.current_starts_less_mixin_declaration() {
            SyntaxKind::LessMixinDeclaration
        } else if starts_less_mixin {
            SyntaxKind::BogusLessMixin
        } else if has_rule_block {
            SyntaxKind::Rule
        } else {
            SyntaxKind::BogusRule
        };

        self.builder.start_node(kind);
        if kind == SyntaxKind::CssModuleImportBlock && !self.current_icss_import_has_source() {
            self.error_at_current(ParseErrorCode::ExpectedValue, "expected ICSS import source");
        }
        if kind == SyntaxKind::LessMixinDeclaration {
            self.parse_less_mixin_header();
        } else if kind == SyntaxKind::BogusLessMixin {
            self.parse_until_recovery_with_optional_less_guard(&[
                SyntaxKind::Semicolon,
                SyntaxKind::RightBrace,
                SyntaxKind::SassDedent,
            ]);
            self.error_at_current(
                ParseErrorCode::UnexpectedCharacter,
                "expected Less mixin block",
            );
        } else {
            self.parse_selector_list();
        }
        if self.current_kind() == Some(SyntaxKind::LeftBrace) {
            self.token_current();
            self.builder
                .start_node(if self.previous_left_brace_has_match() {
                    SyntaxKind::DeclarationList
                } else {
                    SyntaxKind::BogusDeclarationList
                });
            self.parse_declaration_list();
            self.builder.finish_node();
            if self.current_kind() == Some(SyntaxKind::RightBrace) {
                self.token_current();
            } else {
                self.missing_token_bogus_trivia(
                    ParseErrorCode::UnexpectedCharacter,
                    "unterminated declaration block",
                );
            }
        } else if self.current_kind() == Some(SyntaxKind::SassIndent) {
            self.builder.start_node(SyntaxKind::SassIndentedBlock);
            self.token_current();
            self.builder.start_node(SyntaxKind::DeclarationList);
            self.parse_declaration_list();
            self.builder.finish_node();
            if self.current_kind() == Some(SyntaxKind::SassDedent) {
                self.token_current();
            } else {
                self.missing_token_bogus_trivia(
                    ParseErrorCode::UnexpectedCharacter,
                    "unterminated Sass indented declaration block",
                );
            }
            self.builder.finish_node();
        } else {
            self.consume_until_recovery(&[
                SyntaxKind::Semicolon,
                SyntaxKind::SassOptionalSemicolon,
                SyntaxKind::RightBrace,
                SyntaxKind::SassDedent,
            ]);
            if self.current_kind().is_some_and(is_statement_end) {
                self.token_current();
            }
        }
        self.builder.finish_node();
    }

    fn current_icss_module_rule_kind(&self) -> Option<SyntaxKind> {
        if self.current_kind() != Some(SyntaxKind::Colon) {
            return None;
        }
        let (name_index, name_kind) = self.non_trivia_token_from(self.position + 1)?;
        if name_kind != SyntaxKind::Ident {
            return None;
        }
        match self.tokens.get(name_index)?.text {
            "export" => Some(SyntaxKind::CssModuleExportBlock),
            "import" => Some(SyntaxKind::CssModuleImportBlock),
            _ => None,
        }
    }

    fn current_icss_import_has_source(&self) -> bool {
        let Some((name_index, SyntaxKind::Ident)) = self.non_trivia_token_from(self.position + 1)
        else {
            return false;
        };
        if self
            .tokens
            .get(name_index)
            .is_none_or(|token| token.text != "import")
        {
            return false;
        }
        let Some((open_index, SyntaxKind::LeftParen)) = self.non_trivia_token_from(name_index + 1)
        else {
            return false;
        };
        let Some((_, source_kind)) = self.non_trivia_token_from(open_index + 1) else {
            return false;
        };
        matches!(
            source_kind,
            SyntaxKind::String | SyntaxKind::Url | SyntaxKind::ScssInterpolationStart
        )
    }

    fn parse_selector_list(&mut self) {
        self.parse_selector_list_until(&[]);
    }

    fn parse_selector_list_until(&mut self, recovery: &[SyntaxKind]) {
        let kind = if self.current_kind() == Some(SyntaxKind::LeftBrace) {
            SyntaxKind::BogusSelectorList
        } else {
            SyntaxKind::SelectorList
        };
        self.builder.start_node(kind);
        while !self.at_end() {
            match self.current_kind() {
                Some(SyntaxKind::Comma) => self.token_current(),
                Some(kind) if is_selector_boundary_until(kind, recovery) => break,
                Some(SyntaxKind::SassIndentedNewline) => self.token_current(),
                Some(_)
                    if recovery.contains(&SyntaxKind::RightParen)
                        && self.current_selector_item_is_bogus(recovery) =>
                {
                    self.parse_bogus_selector_until(recovery)
                }
                Some(_) => self.parse_selector_until(recovery),
                None => break,
            }
        }
        self.builder.finish_node();
    }

    fn parse_strict_selector_list_until(&mut self, recovery: &[SyntaxKind]) {
        self.builder.start_node(
            if self.selector_list_contains_bogus_item_until(recovery)
                && self.current_kind() != Some(SyntaxKind::RightParen)
            {
                SyntaxKind::BogusSelectorList
            } else {
                SyntaxKind::SelectorList
            },
        );
        while !self.at_end() {
            match self.current_kind() {
                Some(SyntaxKind::Comma) => self.token_current(),
                Some(kind) if is_selector_boundary_until(kind, recovery) => break,
                Some(SyntaxKind::SassIndentedNewline) => self.token_current(),
                Some(_)
                    if self.current_selector_item_is_bogus(recovery)
                        && self.current_kind() != Some(SyntaxKind::RightParen) =>
                {
                    self.parse_bogus_selector_until(recovery)
                }
                Some(_) => self.parse_selector_until(recovery),
                None => break,
            }
        }
        self.builder.finish_node();
    }

    fn parse_relative_selector_list_until(&mut self, recovery: &[SyntaxKind]) {
        self.builder.start_node(
            if self.current_selector_item_is_bogus(recovery)
                && self.current_kind() != Some(SyntaxKind::RightParen)
            {
                SyntaxKind::BogusSelectorList
            } else {
                SyntaxKind::RelativeSelectorList
            },
        );
        while !self.at_end() {
            match self.current_kind() {
                Some(SyntaxKind::Comma) => self.token_current(),
                Some(kind) if is_selector_boundary_until(kind, recovery) => break,
                Some(SyntaxKind::SassIndentedNewline) => self.token_current(),
                Some(_)
                    if self.current_selector_item_is_bogus(recovery)
                        && self.current_kind() != Some(SyntaxKind::RightParen) =>
                {
                    self.parse_bogus_selector_until(recovery)
                }
                Some(_) => self.parse_relative_selector_until(recovery),
                None => break,
            }
        }
        self.builder.finish_node();
    }

    fn parse_relative_selector_until(&mut self, recovery: &[SyntaxKind]) {
        self.builder.start_node(SyntaxKind::RelativeSelector);
        self.builder.start_node(SyntaxKind::ComplexSelector);
        self.parse_complex_selector_until(recovery);
        self.builder.finish_node();
        self.builder.finish_node();
    }

    fn parse_bogus_selector_until(&mut self, recovery: &[SyntaxKind]) {
        self.builder.start_node(SyntaxKind::BogusSelector);
        self.error_at_current(
            ParseErrorCode::UnexpectedCharacter,
            "invalid selector in selector list",
        );
        let mut paren_depth = 0usize;
        let mut bracket_depth = 0usize;
        while !self.at_end() {
            let Some(kind) = self.current_kind() else {
                break;
            };
            if paren_depth == 0
                && bracket_depth == 0
                && (kind == SyntaxKind::Comma || is_selector_boundary_until(kind, recovery))
            {
                break;
            }
            match kind {
                SyntaxKind::LeftParen => paren_depth += 1,
                SyntaxKind::RightParen => paren_depth = paren_depth.saturating_sub(1),
                SyntaxKind::LeftBracket => bracket_depth += 1,
                SyntaxKind::RightBracket => bracket_depth = bracket_depth.saturating_sub(1),
                _ => {}
            }
            self.token_current();
        }
        self.builder.finish_node();
    }

    fn parse_selector_until(&mut self, recovery: &[SyntaxKind]) {
        self.builder.start_node(SyntaxKind::Selector);
        self.builder.start_node(SyntaxKind::ComplexSelector);
        self.parse_complex_selector_until(recovery);
        self.builder.finish_node();
        self.builder.finish_node();
    }

    fn parse_complex_selector_until(&mut self, recovery: &[SyntaxKind]) {
        let mut has_component = false;
        while !self.at_end() {
            match self.current_kind() {
                Some(kind) if is_selector_boundary_until(kind, recovery) => break,
                Some(SyntaxKind::Whitespace) => {
                    if has_component
                        && self.next_non_trivia_kind().is_some_and(|kind| {
                            !is_selector_boundary_until(kind, recovery) && !is_combinator(kind)
                        })
                    {
                        self.parse_whitespace_combinator();
                        has_component = false;
                    } else {
                        self.token_current();
                    }
                }
                Some(SyntaxKind::SassIndentedNewline) => self.token_current(),
                Some(kind) if is_combinator(kind) => {
                    self.parse_combinator();
                    has_component = false;
                }
                Some(_) => {
                    self.parse_compound_selector_until(recovery);
                    has_component = true;
                }
                None => break,
            }
        }
    }

    fn parse_compound_selector_until(&mut self, recovery: &[SyntaxKind]) {
        let starts_valid = self.current_kind().is_some_and(|kind| {
            selector_component_can_start(kind)
                || self.current_starts_namespace_qualified_selector(kind)
                || is_interpolation_start(kind)
        });
        self.builder.start_node(if starts_valid {
            SyntaxKind::CompoundSelector
        } else {
            SyntaxKind::BogusCompoundSelector
        });
        let start = self.position;
        while !self.at_end() {
            match self.current_kind() {
                Some(kind)
                    if is_selector_boundary_until(kind, recovery)
                        || kind == SyntaxKind::Whitespace
                        || kind == SyntaxKind::SassIndentedNewline
                        || is_combinator(kind) =>
                {
                    break;
                }
                Some(SyntaxKind::Dot) => self.parse_class_selector(),
                Some(SyntaxKind::Hash) => self.parse_id_selector(),
                Some(kind) if self.current_starts_namespace_qualified_selector(kind) => {
                    self.parse_namespace_qualified_selector()
                }
                Some(SyntaxKind::Ident) => self.parse_type_selector(),
                Some(SyntaxKind::Star) => self.parse_universal_selector(),
                Some(SyntaxKind::Ampersand) => self.parse_nesting_selector(),
                Some(SyntaxKind::ScssPlaceholder) => self.parse_scss_placeholder_selector(),
                Some(kind) if is_interpolation_start(kind) => self.parse_interpolation(
                    kind,
                    &[
                        SyntaxKind::Comma,
                        SyntaxKind::LeftBrace,
                        SyntaxKind::SassIndent,
                        SyntaxKind::RightBrace,
                        SyntaxKind::SassDedent,
                        SyntaxKind::RightParen,
                        SyntaxKind::Semicolon,
                        SyntaxKind::SassOptionalSemicolon,
                    ],
                ),
                Some(SyntaxKind::LeftBracket) => self.parse_attribute_selector(),
                Some(SyntaxKind::Colon) if self.current_starts_less_extend_rule() => {
                    self.parse_less_extend_rule()
                }
                Some(SyntaxKind::Colon) => {
                    self.parse_pseudo_selector(SyntaxKind::PseudoClassSelector)
                }
                Some(SyntaxKind::DoubleColon) => {
                    self.parse_pseudo_selector(SyntaxKind::PseudoElementSelector)
                }
                Some(_) => self.token_current(),
                None => break,
            }
        }
        if self.position == start {
            self.token_current();
        }
        if !starts_valid {
            self.error_at_current(
                ParseErrorCode::UnexpectedCharacter,
                "expected selector component",
            );
        }
        self.builder.finish_node();
    }

    fn parse_class_selector(&mut self) {
        self.builder.start_node(SyntaxKind::ClassSelector);
        self.token_current();
        if matches!(
            self.current_kind(),
            Some(SyntaxKind::Ident | SyntaxKind::CustomPropertyName)
        ) {
            self.token_current();
        } else {
            self.empty_bogus_node(
                SyntaxKind::BogusSelector,
                ParseErrorCode::ExpectedSelectorName,
                "expected class selector name",
            );
        }
        self.builder.finish_node();
    }

    fn parse_id_selector(&mut self) {
        self.builder.start_node(SyntaxKind::IdSelector);
        self.token_current();
        self.builder.finish_node();
    }

    fn parse_type_selector(&mut self) {
        self.builder.start_node(SyntaxKind::TypeSelector);
        self.token_current();
        self.builder.finish_node();
    }

    fn parse_universal_selector(&mut self) {
        self.builder.start_node(SyntaxKind::UniversalSelector);
        self.token_current();
        self.builder.finish_node();
    }

    fn parse_namespace_qualified_selector(&mut self) {
        let selector_kind =
            if self.namespace_qualified_selector_target_kind() == Some(SyntaxKind::Star) {
                SyntaxKind::UniversalSelector
            } else {
                SyntaxKind::TypeSelector
            };
        self.builder.start_node(selector_kind);
        self.builder.start_node(SyntaxKind::NamespacePrefix);
        if self.current_kind() != Some(SyntaxKind::Pipe) {
            self.token_current();
        }
        self.token_current();
        self.builder.finish_node();
        if matches!(
            self.current_kind(),
            Some(SyntaxKind::Ident | SyntaxKind::CustomPropertyName | SyntaxKind::Star)
        ) {
            self.token_current();
        } else {
            self.empty_bogus_node(
                SyntaxKind::BogusSelector,
                ParseErrorCode::ExpectedSelectorName,
                "expected namespace-qualified selector name",
            );
        }
        self.builder.finish_node();
    }

    fn parse_nesting_selector(&mut self) {
        self.builder.start_node(SyntaxKind::NestingSelectorNode);
        self.token_current();
        self.builder.finish_node();
    }

    fn parse_scss_placeholder_selector(&mut self) {
        self.builder.start_node(SyntaxKind::ScssPlaceholderSelector);
        self.token_current();
        self.builder.finish_node();
    }

    fn parse_attribute_selector(&mut self) {
        let kind = if self.find_before_recovery(
            SyntaxKind::RightBracket,
            &[
                SyntaxKind::Comma,
                SyntaxKind::LeftBrace,
                SyntaxKind::RightBrace,
                SyntaxKind::Semicolon,
            ],
        ) {
            SyntaxKind::AttributeSelector
        } else {
            SyntaxKind::BogusSelector
        };
        self.builder.start_node(kind);
        self.token_current();
        let mut saw_matcher = false;
        let mut saw_value = false;
        let mut closed = false;
        while !self.at_end() {
            match self.current_kind() {
                Some(SyntaxKind::RightBracket) => {
                    self.token_current();
                    closed = true;
                    break;
                }
                Some(kind) if is_attribute_matcher(kind) => {
                    self.parse_attribute_matcher();
                    saw_matcher = true;
                }
                Some(kind) if is_selector_boundary(kind) => break,
                Some(kind) if !saw_matcher && attribute_name_token_can_start(kind) => {
                    self.parse_attribute_name()
                }
                Some(kind)
                    if saw_matcher && !saw_value && attribute_value_token_can_start(kind) =>
                {
                    self.parse_attribute_value();
                    saw_value = true;
                }
                Some(SyntaxKind::Ident | SyntaxKind::CustomPropertyName) if saw_value => {
                    self.parse_attribute_modifier()
                }
                Some(_) => self.token_current(),
                None => break,
            }
        }
        if !closed {
            self.error_at_current(
                ParseErrorCode::UnterminatedAttributeSelector,
                "unterminated attribute selector",
            );
        }
        self.builder.finish_node();
    }

    fn parse_attribute_matcher(&mut self) {
        self.builder.start_node(SyntaxKind::AttributeMatcher);
        self.token_current();
        self.builder.finish_node();
    }

    fn parse_attribute_name(&mut self) {
        self.builder.start_node(SyntaxKind::AttributeName);
        while !self.at_end() {
            match self.current_kind() {
                Some(SyntaxKind::RightBracket) => break,
                Some(kind) if is_attribute_matcher(kind) || is_selector_boundary(kind) => break,
                Some(kind) if attribute_name_token_can_continue(kind) => self.token_current(),
                Some(_) => break,
                None => break,
            }
        }
        self.builder.finish_node();
    }

    fn parse_attribute_value(&mut self) {
        self.builder.start_node(SyntaxKind::AttributeValue);
        self.token_current();
        self.builder.finish_node();
    }

    fn parse_attribute_modifier(&mut self) {
        self.builder.start_node(SyntaxKind::AttributeModifier);
        self.token_current();
        self.builder.finish_node();
    }

    fn parse_pseudo_selector(&mut self, kind: SyntaxKind) {
        self.builder.start_node(kind);
        self.token_current();
        let pseudo_name = self.current_text().map(str::to_owned);
        let css_module_scope_kind = if kind == SyntaxKind::PseudoClassSelector {
            self.current_text().and_then(css_module_scope_function_kind)
        } else {
            None
        };
        if self.current_kind() == Some(SyntaxKind::Ident) {
            if let Some(kind) = css_module_scope_kind {
                self.builder.start_node(kind);
            }
            self.token_current();
        } else {
            self.empty_bogus_node(
                SyntaxKind::BogusSelector,
                ParseErrorCode::ExpectedSelectorName,
                "expected pseudo selector name",
            );
        }
        if self.current_kind() == Some(SyntaxKind::LeftParen) {
            self.token_current();
            self.builder.start_node(SyntaxKind::PseudoSelectorArgument);
            if kind == SyntaxKind::PseudoClassSelector
                && pseudo_name
                    .as_deref()
                    .is_some_and(is_selector_list_pseudo_class)
            {
                self.parse_selector_list_until(&[SyntaxKind::RightParen]);
            } else if kind == SyntaxKind::PseudoClassSelector
                && pseudo_name.as_deref() == Some("not")
            {
                self.parse_strict_selector_list_until(&[SyntaxKind::RightParen]);
            } else if kind == SyntaxKind::PseudoClassSelector
                && pseudo_name.as_deref() == Some("has")
            {
                self.parse_relative_selector_list_until(&[SyntaxKind::RightParen]);
            } else if kind == SyntaxKind::PseudoClassSelector
                && pseudo_name.as_deref().is_some_and(is_nth_pseudo_class)
            {
                self.parse_nth_selector_argument();
            } else if kind == SyntaxKind::PseudoClassSelector
                && pseudo_name.as_deref() == Some("lang")
            {
                self.parse_language_selector_argument();
            } else if kind == SyntaxKind::PseudoClassSelector
                && pseudo_name.as_deref() == Some("dir")
            {
                self.parse_directionality_selector_argument();
            } else {
                while !self.at_end() {
                    match self.current_kind() {
                        Some(SyntaxKind::RightParen) => break,
                        Some(kind) if is_selector_boundary(kind) => break,
                        Some(_) => self.token_current(),
                        None => break,
                    }
                }
            }
            self.builder.finish_node();
            if self.current_kind() == Some(SyntaxKind::RightParen) {
                self.token_current();
            }
        }
        if css_module_scope_kind.is_some() {
            self.builder.finish_node();
        }
        self.builder.finish_node();
    }

    fn parse_nth_selector_argument(&mut self) {
        self.builder.start_node(SyntaxKind::NthSelectorArgument);
        self.builder.start_node(SyntaxKind::NthSelectorFormula);
        while !self.at_end() {
            match self.current_kind() {
                Some(SyntaxKind::RightParen) => break,
                Some(kind) if is_selector_boundary(kind) => break,
                Some(SyntaxKind::Ident) if self.current_text() == Some("of") => break,
                Some(_) => self.token_current(),
                None => break,
            }
        }
        self.builder.finish_node();

        if self.current_kind() == Some(SyntaxKind::Ident) && self.current_text() == Some("of") {
            self.builder
                .start_node(SyntaxKind::NthSelectorOfSelectorList);
            self.token_current();
            self.parse_selector_list_until(&[SyntaxKind::RightParen]);
            self.builder.finish_node();
        }

        self.builder.finish_node();
    }

    fn parse_language_selector_argument(&mut self) {
        self.builder
            .start_node(SyntaxKind::LanguageSelectorArgument);
        while !self.at_end() {
            match self.current_kind() {
                Some(SyntaxKind::RightParen) => break,
                Some(SyntaxKind::Comma) => self.token_current(),
                Some(kind) if is_selector_boundary(kind) => break,
                Some(kind) if language_tag_token_can_start(kind) => self.parse_language_tag(),
                Some(_) => self.token_current(),
                None => break,
            }
        }
        self.builder.finish_node();
    }

    fn parse_language_tag(&mut self) {
        self.builder.start_node(SyntaxKind::LanguageTag);
        self.token_current();
        self.builder.finish_node();
    }

    fn parse_directionality_selector_argument(&mut self) {
        self.builder
            .start_node(SyntaxKind::DirectionalitySelectorArgument);
        if self
            .current_kind()
            .is_some_and(language_tag_token_can_start)
        {
            self.token_current();
        }
        while !self.at_end() {
            match self.current_kind() {
                Some(SyntaxKind::RightParen) => break,
                Some(kind) if is_selector_boundary(kind) => break,
                Some(_) => self.token_current(),
                None => break,
            }
        }
        self.builder.finish_node();
    }

    fn parse_less_extend_rule(&mut self) {
        self.builder.start_node(SyntaxKind::LessExtendRule);
        if self.current_kind() == Some(SyntaxKind::Colon) {
            self.token_current();
        }
        if self.current_text() == Some("extend") {
            self.token_current();
        } else {
            self.empty_bogus_node(
                SyntaxKind::BogusSelector,
                ParseErrorCode::ExpectedSelectorName,
                "expected Less extend selector",
            );
        }
        if self.current_kind() == Some(SyntaxKind::LeftParen) {
            self.token_current();
            self.builder.start_node(SyntaxKind::PseudoSelectorArgument);
            while !self.at_end() {
                match self.current_kind() {
                    Some(SyntaxKind::RightParen) => break,
                    Some(kind) if is_selector_boundary(kind) => break,
                    Some(kind) if is_interpolation_start(kind) => self.parse_interpolation(
                        kind,
                        &[
                            SyntaxKind::RightParen,
                            SyntaxKind::Comma,
                            SyntaxKind::LeftBrace,
                            SyntaxKind::SassIndent,
                            SyntaxKind::Semicolon,
                            SyntaxKind::SassOptionalSemicolon,
                        ],
                    ),
                    Some(_) => self.token_current(),
                    None => break,
                }
            }
            self.builder.finish_node();
            if self.current_kind() == Some(SyntaxKind::RightParen) {
                self.token_current();
            }
        }
        self.builder.finish_node();
    }

    fn parse_combinator(&mut self) {
        let has_rhs = self
            .next_non_trivia_kind()
            .is_some_and(|kind| selector_component_can_start(kind) || is_interpolation_start(kind));
        self.builder.start_node(if has_rhs {
            SyntaxKind::Combinator
        } else {
            SyntaxKind::BogusCombinator
        });
        self.token_current();
        if !has_rhs {
            self.error_at_current(
                ParseErrorCode::UnexpectedCharacter,
                "expected selector after combinator",
            );
        }
        self.builder.finish_node();
    }

    fn parse_whitespace_combinator(&mut self) {
        self.builder.start_node(SyntaxKind::Combinator);
        while self.current_kind() == Some(SyntaxKind::Whitespace) {
            self.token_current();
        }
        self.builder.finish_node();
    }

    fn parse_declaration_list(&mut self) {
        while !self.at_end() {
            self.eat_trivia();
            match self.current_kind() {
                Some(SyntaxKind::RightBrace | SyntaxKind::SassDedent) | None => break,
                Some(SyntaxKind::Semicolon | SyntaxKind::SassOptionalSemicolon) => {
                    self.token_current()
                }
                Some(SyntaxKind::AtKeyword) if self.current_is_css_module_value_rule() => {
                    self.parse_css_module_value_rule()
                }
                Some(SyntaxKind::AtKeyword) if self.current_dialect_at_rule_spec().is_some() => {
                    self.parse_dialect_at_rule()
                }
                Some(SyntaxKind::AtKeyword) => self.parse_at_rule(),
                Some(_) if self.current_starts_less_namespace_access() => {
                    self.parse_less_namespace_access()
                }
                Some(_) if self.current_starts_less_mixin_call() => self.parse_less_mixin_call(),
                Some(_) if self.current_starts_scss_nested_property() => {
                    self.parse_scss_nested_property()
                }
                Some(_) if self.current_starts_nested_rule() => self.parse_rule(),
                Some(SyntaxKind::ScssVariable)
                    if matches!(self.dialect, StyleDialect::Scss | StyleDialect::Sass) =>
                {
                    self.parse_variable_declaration(SyntaxKind::ScssVariableDeclaration)
                }
                Some(SyntaxKind::LessVariable) if self.dialect == StyleDialect::Less => {
                    self.parse_variable_declaration(SyntaxKind::LessVariableDeclaration)
                }
                Some(SyntaxKind::LeftBrace) => {
                    self.builder.start_node(SyntaxKind::BogusDeclaration);
                    self.token_current();
                    self.builder.finish_node();
                }
                Some(_) => self.parse_declaration(),
            }
        }
    }

    fn parse_scss_nested_property(&mut self) {
        self.builder.start_node(SyntaxKind::ScssNestedProperty);
        self.builder.start_node(SyntaxKind::PropertyName);
        while !self.at_end() {
            match self.current_kind() {
                Some(SyntaxKind::Colon) => break,
                Some(
                    SyntaxKind::Semicolon
                    | SyntaxKind::SassOptionalSemicolon
                    | SyntaxKind::RightBrace
                    | SyntaxKind::SassDedent,
                ) => break,
                Some(kind) if is_interpolation_start(kind) => self.parse_interpolation(
                    kind,
                    &[
                        SyntaxKind::Colon,
                        SyntaxKind::Semicolon,
                        SyntaxKind::SassOptionalSemicolon,
                        SyntaxKind::RightBrace,
                        SyntaxKind::SassDedent,
                    ],
                ),
                Some(_) => self.token_current(),
                None => break,
            }
        }
        self.builder.finish_node();

        if self.current_kind() == Some(SyntaxKind::Colon) {
            self.token_current();
        }

        let block_recovery = [
            SyntaxKind::LeftBrace,
            SyntaxKind::SassIndent,
            SyntaxKind::Semicolon,
            SyntaxKind::SassOptionalSemicolon,
            SyntaxKind::RightBrace,
            SyntaxKind::SassDedent,
        ];
        if !matches!(
            self.current_kind(),
            Some(
                SyntaxKind::LeftBrace
                    | SyntaxKind::SassIndent
                    | SyntaxKind::Semicolon
                    | SyntaxKind::SassOptionalSemicolon
                    | SyntaxKind::RightBrace
                    | SyntaxKind::SassDedent
            )
        ) {
            self.builder.start_node(SyntaxKind::Value);
            self.parse_value_or_value_list_until(&block_recovery);
            self.builder.finish_node();
        }

        match self.current_kind() {
            Some(SyntaxKind::LeftBrace) => self.parse_declaration_block(),
            Some(SyntaxKind::SassIndent) => self.parse_sass_indented_nested_property_block(),
            Some(_) => self.consume_until_recovery(&[
                SyntaxKind::Semicolon,
                SyntaxKind::SassOptionalSemicolon,
                SyntaxKind::RightBrace,
                SyntaxKind::SassDedent,
            ]),
            None => {}
        }

        if self.current_kind().is_some_and(is_statement_end) {
            self.token_current();
        }
        self.builder.finish_node();
    }

    fn parse_sass_indented_nested_property_block(&mut self) {
        self.builder.start_node(SyntaxKind::SassIndentedBlock);
        if self.current_kind() == Some(SyntaxKind::SassIndent) {
            self.token_current();
        }
        self.builder.start_node(SyntaxKind::DeclarationList);
        self.parse_declaration_list();
        self.builder.finish_node();
        if self.current_kind() == Some(SyntaxKind::SassDedent) {
            self.token_current();
        } else {
            self.error_at_current(
                ParseErrorCode::UnexpectedCharacter,
                "unterminated Sass indented nested property block",
            );
        }
        self.builder.finish_node();
    }

    fn parse_variable_declaration(&mut self, kind: SyntaxKind) {
        let has_colon = self.find_before_recovery(
            SyntaxKind::Colon,
            &[
                SyntaxKind::Semicolon,
                SyntaxKind::SassOptionalSemicolon,
                SyntaxKind::RightBrace,
                SyntaxKind::SassDedent,
            ],
        );
        self.builder
            .start_node(variable_declaration_node_kind(kind, has_colon));
        self.token_current();
        if self.current_kind() == Some(SyntaxKind::Colon) {
            self.token_current();
            self.eat_value_trivia();
            let value_recovery = [
                SyntaxKind::Semicolon,
                SyntaxKind::SassOptionalSemicolon,
                SyntaxKind::RightBrace,
                SyntaxKind::SassDedent,
            ];
            if kind == SyntaxKind::LessVariableDeclaration
                && self.current_kind() == Some(SyntaxKind::LeftBrace)
            {
                self.parse_less_detached_ruleset();
            } else {
                let has_value = self
                    .non_trivia_token_from(self.position)
                    .is_some_and(|(_, kind)| !value_recovery.contains(&kind));
                self.builder.start_node(SyntaxKind::Value);
                if has_value {
                    self.parse_value_or_value_list_until(&value_recovery);
                } else {
                    self.empty_bogus_node(
                        SyntaxKind::BogusValue,
                        ParseErrorCode::ExpectedValue,
                        "expected variable value",
                    );
                }
                self.builder.finish_node();
            }
        } else {
            self.error_at_current(
                ParseErrorCode::UnexpectedCharacter,
                "expected variable declaration colon",
            );
            self.consume_until_recovery(&[
                SyntaxKind::Semicolon,
                SyntaxKind::SassOptionalSemicolon,
                SyntaxKind::RightBrace,
                SyntaxKind::SassDedent,
            ]);
        }
        if self.current_kind().is_some_and(is_statement_end) {
            self.token_current();
        }
        self.builder.finish_node();
    }

    fn parse_less_detached_ruleset(&mut self) {
        let closed = self.current_left_brace_has_match();
        self.builder.start_node(if closed {
            SyntaxKind::LessDetachedRulesetNode
        } else {
            SyntaxKind::BogusLessDetachedRuleset
        });
        if self.current_kind() == Some(SyntaxKind::LeftBrace) {
            self.token_current();
            self.builder.start_node(SyntaxKind::DeclarationList);
            self.parse_declaration_list();
            self.builder.finish_node();
        }
        if self.current_kind() == Some(SyntaxKind::RightBrace) {
            self.token_current();
        } else {
            self.error_at_current(
                ParseErrorCode::UnexpectedCharacter,
                "unterminated Less detached ruleset",
            );
        }
        self.builder.finish_node();
    }

    fn parse_declaration(&mut self) {
        let starts_composes = self.current_text() == Some("composes");
        let starts_custom_property = self.current_kind() == Some(SyntaxKind::CustomPropertyName);
        let has_colon = self.find_before_recovery(
            SyntaxKind::Colon,
            &[
                SyntaxKind::Semicolon,
                SyntaxKind::SassOptionalSemicolon,
                SyntaxKind::RightBrace,
                SyntaxKind::SassDedent,
                SyntaxKind::LeftBrace,
                SyntaxKind::SassIndent,
            ],
        );
        let kind = if starts_composes && has_colon {
            SyntaxKind::CssModuleComposesDeclaration
        } else if starts_composes {
            SyntaxKind::BogusComposesDeclaration
        } else if has_colon {
            SyntaxKind::Declaration
        } else {
            SyntaxKind::BogusDeclaration
        };
        self.builder.start_node(kind);
        if kind == SyntaxKind::CssModuleComposesDeclaration
            && self.current_css_module_scope_context() == Some("global")
        {
            self.error_at_current(
                ParseErrorCode::UnexpectedCharacter,
                "composes is not allowed inside :global scope",
            );
        }
        let property_kind = if matches!(
            self.current_kind(),
            Some(
                SyntaxKind::Colon
                    | SyntaxKind::Semicolon
                    | SyntaxKind::SassOptionalSemicolon
                    | SyntaxKind::LeftBrace
                    | SyntaxKind::SassIndent
                    | SyntaxKind::RightBrace
                    | SyntaxKind::SassDedent
            )
        ) {
            SyntaxKind::BogusPropertyName
        } else {
            SyntaxKind::PropertyName
        };
        self.builder.start_node(property_kind);
        while !self.at_end() {
            match self.current_kind() {
                Some(
                    SyntaxKind::Colon
                    | SyntaxKind::Semicolon
                    | SyntaxKind::SassOptionalSemicolon
                    | SyntaxKind::RightBrace
                    | SyntaxKind::SassDedent,
                ) => break,
                Some(kind) if is_interpolation_start(kind) => self.parse_interpolation(
                    kind,
                    &[
                        SyntaxKind::Colon,
                        SyntaxKind::Semicolon,
                        SyntaxKind::SassOptionalSemicolon,
                        SyntaxKind::RightBrace,
                        SyntaxKind::SassDedent,
                    ],
                ),
                Some(_) => self.token_current(),
                None => break,
            }
        }
        self.builder.finish_node();
        if property_kind == SyntaxKind::BogusPropertyName {
            self.error_at_current(
                ParseErrorCode::UnexpectedCharacter,
                "expected declaration property name",
            );
        }

        if self.current_kind() == Some(SyntaxKind::Colon) {
            self.token_current();
            let value_recovery = [
                SyntaxKind::Semicolon,
                SyntaxKind::SassOptionalSemicolon,
                SyntaxKind::RightBrace,
                SyntaxKind::SassDedent,
            ];
            let has_value = self
                .non_trivia_token_from(self.position)
                .is_some_and(|(_, kind)| !value_recovery.contains(&kind));
            self.builder.start_node(SyntaxKind::Value);
            if kind == SyntaxKind::CssModuleComposesDeclaration {
                self.parse_composes_value_until(&value_recovery);
            } else if starts_custom_property {
                self.builder.start_node(SyntaxKind::CustomPropertyValue);
                self.parse_component_value_list_until(&value_recovery);
                self.builder.finish_node();
            } else if !has_value {
                self.empty_bogus_node(
                    SyntaxKind::BogusValue,
                    ParseErrorCode::ExpectedValue,
                    "expected declaration value",
                );
            } else {
                self.parse_declaration_value_or_value_list_until(&value_recovery);
            }
            self.builder.finish_node();
        } else {
            self.consume_until_recovery(&[
                SyntaxKind::Semicolon,
                SyntaxKind::SassOptionalSemicolon,
                SyntaxKind::RightBrace,
                SyntaxKind::SassDedent,
            ]);
        }

        if self.current_kind().is_some_and(is_statement_end) {
            self.token_current();
        }
        self.builder.finish_node();
    }

    fn parse_composes_value_until(&mut self, recovery: &[SyntaxKind]) {
        let mut saw_target = false;
        if self.current_composes_value_has_multiple_from_clauses(recovery) {
            self.error_at_current(
                ParseErrorCode::UnexpectedCharacter,
                "multiple composes from clauses are not allowed",
            );
        }
        while !self.at_end() {
            self.eat_value_trivia();
            match self.current_kind() {
                Some(kind) if recovery.contains(&kind) => break,
                Some(SyntaxKind::Ident) if self.current_text() == Some("from") => {
                    if !saw_target {
                        self.empty_bogus_node(
                            SyntaxKind::BogusComposesTarget,
                            ParseErrorCode::UnexpectedCharacter,
                            "expected composes target before from clause",
                        );
                        saw_target = true;
                    }
                    self.parse_css_module_from_clause(recovery);
                }
                Some(SyntaxKind::Ident | SyntaxKind::CustomPropertyName) => {
                    self.builder.start_node(SyntaxKind::CssModuleComposesTarget);
                    self.token_current();
                    self.builder.finish_node();
                    saw_target = true;
                }
                Some(kind) if is_interpolation_start(kind) => {
                    self.parse_interpolation(kind, recovery)
                }
                Some(_) => self.token_current(),
                None => break,
            }
        }
        if !saw_target {
            self.empty_bogus_node(
                SyntaxKind::BogusComposesTarget,
                ParseErrorCode::UnexpectedCharacter,
                "expected composes target",
            );
        }
    }

    fn current_composes_value_has_multiple_from_clauses(&self, recovery: &[SyntaxKind]) -> bool {
        let mut index = self.position;
        let mut paren_depth = 0usize;
        let mut bracket_depth = 0usize;
        let mut brace_depth = 0usize;
        let mut from_count = 0usize;
        while let Some(token) = self.tokens.get(index) {
            if paren_depth == 0
                && bracket_depth == 0
                && brace_depth == 0
                && recovery.contains(&token.kind)
            {
                break;
            }
            match token.kind {
                SyntaxKind::LeftParen => paren_depth += 1,
                SyntaxKind::RightParen => paren_depth = paren_depth.saturating_sub(1),
                SyntaxKind::LeftBracket => bracket_depth += 1,
                SyntaxKind::RightBracket => bracket_depth = bracket_depth.saturating_sub(1),
                SyntaxKind::LeftBrace => brace_depth += 1,
                SyntaxKind::RightBrace => brace_depth = brace_depth.saturating_sub(1),
                SyntaxKind::Ident
                    if paren_depth == 0
                        && bracket_depth == 0
                        && brace_depth == 0
                        && token.text == "from" =>
                {
                    from_count += 1;
                    if from_count > 1 {
                        return true;
                    }
                }
                _ => {}
            }
            index += 1;
        }
        false
    }

    fn parse_css_module_from_clause(&mut self, recovery: &[SyntaxKind]) {
        let source = self.non_trivia_token_from(self.position + 1);
        let has_source = source.is_some_and(|(_, kind)| !recovery.contains(&kind));
        let has_valid_source = source.is_some_and(|(index, kind)| {
            self.tokens
                .get(index)
                .is_some_and(|token| is_css_module_from_source_token(kind, token.text))
        });
        self.builder.start_node(if has_valid_source {
            SyntaxKind::CssModuleFromClause
        } else {
            SyntaxKind::BogusFromClause
        });
        self.token_current();
        while !self.at_end() {
            match self.current_kind() {
                Some(kind) if recovery.contains(&kind) => break,
                Some(_) => self.token_current(),
                None => break,
            }
        }
        if !has_source {
            self.error_at_current(
                ParseErrorCode::UnexpectedCharacter,
                "expected CSS Modules from-clause source",
            );
        } else if !has_valid_source {
            self.error_at_current(
                ParseErrorCode::ExpectedValue,
                "invalid CSS Modules from-clause source",
            );
        }
        self.builder.finish_node();
    }

    fn current_css_module_scope_context(&self) -> Option<&'static str> {
        let mut open_blocks = Vec::new();
        for (index, token) in self.tokens.iter().take(self.position).enumerate() {
            match token.kind {
                SyntaxKind::LeftBrace | SyntaxKind::SassIndent => open_blocks.push(index),
                SyntaxKind::RightBrace | SyntaxKind::SassDedent => {
                    open_blocks.pop();
                }
                _ => {}
            }
        }

        if let Some(scope) = open_blocks.iter().copied().find_map(|block_start| {
            let header_start = self.header_start_for_block(block_start);
            css_module_block_scope_marker_in_header(&self.tokens, header_start, block_start)
        }) {
            return Some(scope);
        }

        let block_start = open_blocks.last().copied()?;
        let header_start = self.header_start_for_block(block_start);
        css_module_header_is_global_only(&self.tokens, header_start, block_start)
            .then_some("global")
    }

    fn header_start_for_block(&self, block_start: usize) -> usize {
        let mut index = block_start;
        while index > 0 {
            let previous = index - 1;
            if matches!(
                self.tokens[previous].kind,
                SyntaxKind::LeftBrace
                    | SyntaxKind::RightBrace
                    | SyntaxKind::SassIndent
                    | SyntaxKind::SassDedent
                    | SyntaxKind::Semicolon
                    | SyntaxKind::SassOptionalSemicolon
            ) {
                break;
            }
            index = previous;
        }
        index
    }

    fn parse_dialect_at_rule(&mut self) {
        let Some(spec) = self.current_dialect_at_rule_spec() else {
            self.parse_at_rule();
            return;
        };

        self.builder
            .start_node(self.current_dialect_at_rule_node_kind(spec));
        if self.current_kind() == Some(SyntaxKind::AtKeyword) {
            self.token_current();
        }
        if matches!(
            spec.node_kind,
            SyntaxKind::ScssUseRule | SyntaxKind::ScssForwardRule
        ) {
            self.parse_scss_module_prelude(spec.node_kind);
        }
        if is_scss_control_rule_kind(spec.node_kind)
            && !self.current_scss_control_prelude_is_valid(spec.node_kind)
        {
            self.error_at_current(
                ParseErrorCode::ExpectedValue,
                "invalid SCSS control prelude",
            );
        }
        while !self.at_end() {
            match self.current_kind() {
                Some(kind) if is_statement_end(kind) => {
                    self.token_current();
                    break;
                }
                Some(SyntaxKind::LeftBrace) => {
                    match spec.block_kind {
                        AtRuleBlockKind::GroupRuleList => self.parse_group_at_rule_block(),
                        AtRuleBlockKind::DeclarationList => self.parse_declaration_block(),
                        AtRuleBlockKind::Keyframes => self.parse_keyframes_block(),
                        AtRuleBlockKind::Raw => self.consume_balanced_block(),
                    }
                    break;
                }
                Some(SyntaxKind::SassIndent) => {
                    self.parse_sass_indented_at_rule_block(spec.block_kind);
                    break;
                }
                Some(_) => self.token_current(),
                None => break,
            }
        }
        self.builder.finish_node();
    }

    fn parse_scss_module_prelude(&mut self, node_kind: SyntaxKind) {
        self.validate_scss_module_prelude(node_kind);
        while !self.at_end() {
            match self.current_kind() {
                Some(kind)
                    if is_statement_end(kind)
                        || kind == SyntaxKind::LeftBrace
                        || kind == SyntaxKind::SassIndent =>
                {
                    break;
                }
                Some(SyntaxKind::Ident | SyntaxKind::KeywordWith)
                    if self.current_text() == Some("with")
                        && self
                            .non_trivia_token_from(self.position + 1)
                            .is_some_and(|(_, kind)| kind == SyntaxKind::LeftParen) =>
                {
                    self.parse_scss_module_config()
                }
                Some(kind) if is_interpolation_start(kind) => self.parse_interpolation(
                    kind,
                    &[
                        SyntaxKind::Semicolon,
                        SyntaxKind::SassOptionalSemicolon,
                        SyntaxKind::LeftBrace,
                        SyntaxKind::SassIndent,
                    ],
                ),
                Some(_) => self.token_current(),
                None => break,
            }
        }
    }

    fn validate_scss_module_prelude(&mut self, node_kind: SyntaxKind) {
        let recovery = [
            SyntaxKind::Semicolon,
            SyntaxKind::SassOptionalSemicolon,
            SyntaxKind::LeftBrace,
            SyntaxKind::SassIndent,
        ];
        let Some((source_index, source_kind)) = self.non_trivia_token_from(self.position) else {
            self.error_at_current(ParseErrorCode::ExpectedValue, "expected SCSS module source");
            return;
        };
        if recovery.contains(&source_kind) || !is_scss_module_source_token(source_kind) {
            let range = self
                .tokens
                .get(source_index)
                .map(|token| token.range)
                .unwrap_or_else(|| self.current_range());
            self.errors.push(ParseError {
                code: ParseErrorCode::ExpectedValue,
                range,
                message: "expected SCSS module source",
            });
        }

        let mut index = source_index;
        while let Some(token) = self.tokens.get(index).copied() {
            if recovery.contains(&token.kind) {
                break;
            }
            if token.kind == SyntaxKind::Ident {
                if token.text.eq_ignore_ascii_case("as") {
                    let next_kind = self.non_trivia_token_from(index + 1).map(|(_, kind)| kind);
                    if next_kind.is_none_or(|kind| {
                        recovery.contains(&kind) || !is_scss_module_namespace_token(kind)
                    }) {
                        self.errors.push(ParseError {
                            code: ParseErrorCode::ExpectedValue,
                            range: token.range,
                            message: "expected SCSS module namespace",
                        });
                    }
                } else if token.text.eq_ignore_ascii_case("with") {
                    let next_kind = self.non_trivia_token_from(index + 1).map(|(_, kind)| kind);
                    if next_kind != Some(SyntaxKind::LeftParen) {
                        self.errors.push(ParseError {
                            code: ParseErrorCode::ExpectedValue,
                            range: token.range,
                            message: "expected SCSS module configuration",
                        });
                    }
                } else if matches_ignore_ascii_case(token.text, &["show", "hide"]) {
                    if node_kind != SyntaxKind::ScssForwardRule {
                        self.errors.push(ParseError {
                            code: ParseErrorCode::UnexpectedCharacter,
                            range: token.range,
                            message: "unexpected SCSS module visibility clause",
                        });
                    }
                    let next_kind = self.non_trivia_token_from(index + 1).map(|(_, kind)| kind);
                    if next_kind.is_none_or(|kind| {
                        recovery.contains(&kind) || !is_scss_module_visibility_name_token(kind)
                    }) {
                        self.errors.push(ParseError {
                            code: ParseErrorCode::ExpectedValue,
                            range: token.range,
                            message: "expected SCSS module visibility name",
                        });
                    }
                }
            }
            index += 1;
        }
    }

    fn current_scss_control_prelude_is_valid(&self, node_kind: SyntaxKind) -> bool {
        let recovery = [
            SyntaxKind::LeftBrace,
            SyntaxKind::SassIndent,
            SyntaxKind::Semicolon,
            SyntaxKind::SassOptionalSemicolon,
            SyntaxKind::RightBrace,
            SyntaxKind::SassDedent,
        ];
        match node_kind {
            SyntaxKind::ScssControlIf | SyntaxKind::ScssControlWhile => self
                .non_trivia_token_from(self.position)
                .is_some_and(|(_, kind)| !recovery.contains(&kind)),
            SyntaxKind::ScssControlFor => {
                self.non_trivia_token_from(self.position)
                    .is_some_and(|(_, kind)| kind == SyntaxKind::ScssVariable)
                    && self.find_text_before_recovery("from", &recovery)
                    && (self.find_text_before_recovery("to", &recovery)
                        || self.find_text_before_recovery("through", &recovery))
            }
            SyntaxKind::ScssControlEach => {
                self.non_trivia_token_from(self.position)
                    .is_some_and(|(_, kind)| kind == SyntaxKind::ScssVariable)
                    && self.find_text_before_recovery("in", &recovery)
            }
            SyntaxKind::ScssControlElse => true,
            _ => true,
        }
    }

    fn parse_scss_module_config(&mut self) {
        let has_balanced_config = self.current_scss_module_config_has_balanced_parens();
        self.builder.start_node(if has_balanced_config {
            SyntaxKind::ScssModuleConfig
        } else {
            SyntaxKind::BogusScssModuleConfig
        });
        self.token_current();
        self.eat_trivia();
        if self.current_kind() == Some(SyntaxKind::LeftParen) {
            self.parse_balanced_parenthesized_prelude_until(
                None,
                &[
                    SyntaxKind::LeftBrace,
                    SyntaxKind::SassIndent,
                    SyntaxKind::Semicolon,
                    SyntaxKind::SassOptionalSemicolon,
                ],
            );
        }
        self.builder.finish_node();
    }

    fn parse_css_module_value_rule(&mut self) {
        let has_name = self
            .non_trivia_token_from(self.position + 1)
            .and_then(|(index, kind)| {
                self.tokens
                    .get(index)
                    .map(|token| (kind, token.text != "from"))
            })
            .is_some_and(|(kind, allowed_name)| {
                allowed_name && matches!(kind, SyntaxKind::Ident | SyntaxKind::CustomPropertyName)
            });
        let has_from = self.find_text_before_recovery(
            "from",
            &[
                SyntaxKind::Semicolon,
                SyntaxKind::SassOptionalSemicolon,
                SyntaxKind::LeftBrace,
                SyntaxKind::SassIndent,
            ],
        );
        let has_colon = self.find_before_recovery(
            SyntaxKind::Colon,
            &[
                SyntaxKind::Semicolon,
                SyntaxKind::SassOptionalSemicolon,
                SyntaxKind::LeftBrace,
                SyntaxKind::SassIndent,
            ],
        );
        let kind = if !has_name {
            SyntaxKind::BogusCssModuleBlock
        } else if has_from && !has_colon {
            SyntaxKind::CssModuleImportBlock
        } else {
            SyntaxKind::CssModuleExportBlock
        };

        self.builder.start_node(kind);
        self.token_current();
        if !has_name {
            self.error_at_current(
                ParseErrorCode::UnexpectedCharacter,
                "expected CSS Modules @value name",
            );
        }
        if has_colon {
            self.parse_css_module_value_export();
        } else {
            self.parse_css_module_value_import_or_statement();
        }
        if self.current_kind().is_some_and(is_statement_end) {
            self.token_current();
        }
        self.builder.finish_node();
    }

    fn parse_css_module_value_export(&mut self) {
        self.parse_css_module_token_definitions_until(&[
            SyntaxKind::Colon,
            SyntaxKind::Semicolon,
            SyntaxKind::SassOptionalSemicolon,
        ]);
        if self.current_kind() == Some(SyntaxKind::Colon) {
            self.token_current();
            self.builder.start_node(SyntaxKind::Value);
            self.parse_css_module_token_references_until(&[
                SyntaxKind::Semicolon,
                SyntaxKind::SassOptionalSemicolon,
            ]);
            self.builder.finish_node();
        }
    }

    fn parse_css_module_value_import_or_statement(&mut self) {
        self.parse_css_module_token_definitions_until(&[
            SyntaxKind::Semicolon,
            SyntaxKind::SassOptionalSemicolon,
        ]);
    }

    fn parse_css_module_token_definitions_until(&mut self, recovery: &[SyntaxKind]) {
        while !self.at_end() {
            match self.current_kind() {
                Some(kind) if recovery.contains(&kind) => break,
                Some(SyntaxKind::Ident) if self.current_text() == Some("from") => {
                    self.parse_css_module_from_clause(recovery);
                    break;
                }
                Some(SyntaxKind::Ident | SyntaxKind::CustomPropertyName) => {
                    self.builder.start_node(SyntaxKind::TokenDefinition);
                    self.token_current();
                    self.builder.finish_node();
                }
                Some(_) => self.token_current(),
                None => break,
            }
        }
    }

    fn parse_css_module_token_references_until(&mut self, recovery: &[SyntaxKind]) {
        while !self.at_end() {
            self.eat_value_trivia();
            match self.current_kind() {
                Some(kind) if recovery.contains(&kind) => break,
                Some(SyntaxKind::Ident | SyntaxKind::CustomPropertyName) => {
                    self.builder.start_node(SyntaxKind::TokenReference);
                    self.token_current();
                    self.builder.finish_node();
                }
                Some(kind) if is_interpolation_start(kind) => {
                    self.parse_interpolation(kind, recovery)
                }
                Some(_) => self.token_current(),
                None => break,
            }
        }
    }

    fn parse_less_mixin_header(&mut self) {
        self.builder.start_node(SyntaxKind::SelectorList);
        self.parse_until_recovery_with_optional_less_guard(&[SyntaxKind::LeftBrace]);
        self.builder.finish_node();
    }

    fn parse_less_mixin_call(&mut self) {
        self.builder.start_node(SyntaxKind::LessMixinCall);
        self.parse_until_recovery_with_optional_less_guard(&[
            SyntaxKind::Semicolon,
            SyntaxKind::SassOptionalSemicolon,
            SyntaxKind::RightBrace,
            SyntaxKind::SassDedent,
        ]);
        if self.current_kind().is_some_and(is_statement_end) {
            self.token_current();
        }
        self.builder.finish_node();
    }

    fn parse_less_namespace_access(&mut self) {
        self.builder.start_node(SyntaxKind::LessNamespaceAccess);
        while !self.at_end() {
            match self.current_kind() {
                Some(
                    SyntaxKind::Semicolon
                    | SyntaxKind::SassOptionalSemicolon
                    | SyntaxKind::RightBrace
                    | SyntaxKind::SassDedent
                    | SyntaxKind::LeftBrace
                    | SyntaxKind::SassIndent,
                ) => break,
                Some(_) if self.current_starts_less_mixin_call() => {
                    self.parse_less_mixin_call();
                    break;
                }
                Some(_) => self.token_current(),
                None => break,
            }
        }
        if self.current_kind().is_some_and(is_statement_end) {
            self.token_current();
        }
        self.builder.finish_node();
    }

    fn parse_until_recovery_with_optional_less_guard(&mut self, recovery: &[SyntaxKind]) {
        let mut guard_open = false;
        while !self.at_end() {
            match self.current_kind() {
                Some(kind) if recovery.contains(&kind) => break,
                Some(SyntaxKind::Ident) if self.current_text() == Some("when") && !guard_open => {
                    self.builder.start_node(
                        if self.current_less_guard_has_condition_before(recovery) {
                            SyntaxKind::LessMixinGuard
                        } else {
                            SyntaxKind::BogusLessGuard
                        },
                    );
                    guard_open = true;
                    self.token_current();
                }
                Some(_) => self.token_current(),
                None => break,
            }
        }
        if guard_open {
            self.builder.finish_node();
        }
    }

    fn parse_value_until(&mut self, recovery: &[SyntaxKind]) {
        while !self.at_end() {
            self.eat_value_trivia();
            if matches!(self.current_kind(), Some(kind) if recovery.contains(&kind)) {
                break;
            }
            if self.at_end() {
                break;
            }
            self.parse_value_expression(0, recovery);
        }
    }

    fn parse_value_or_value_list_until(&mut self, recovery: &[SyntaxKind]) {
        if self.current_value_has_top_level_comma_before(recovery) {
            self.parse_value_list_until(recovery);
        } else {
            self.parse_value_until(recovery);
        }
    }

    fn parse_declaration_value_or_value_list_until(&mut self, recovery: &[SyntaxKind]) {
        if self.current_value_has_top_level_comma_before(recovery) {
            self.parse_declaration_value_list_until(recovery);
        } else {
            self.parse_declaration_value_until(recovery);
        }
    }

    fn parse_declaration_value_until(&mut self, recovery: &[SyntaxKind]) {
        let mut saw_value = false;
        while !self.at_end() {
            self.eat_value_trivia();
            if matches!(self.current_kind(), Some(kind) if recovery.contains(&kind)) {
                break;
            }
            if saw_value && self.current_starts_missing_semicolon_declaration(recovery) {
                self.error_at_current(
                    ParseErrorCode::UnexpectedCharacter,
                    "expected semicolon between declarations",
                );
                break;
            }
            if self.at_end() {
                break;
            }
            self.parse_value_expression(0, recovery);
            saw_value = true;
        }
    }

    fn parse_declaration_value_list_until(&mut self, recovery: &[SyntaxKind]) {
        self.builder
            .start_node(if self.current_value_list_is_bogus(recovery) {
                SyntaxKind::BogusValueList
            } else {
                SyntaxKind::ValueList
            });
        let item_recovery = value_list_item_recovery(recovery);
        let mut saw_item = false;
        while !self.at_end() {
            self.eat_value_trivia();
            match self.current_kind() {
                Some(kind) if recovery.contains(&kind) => break,
                Some(SyntaxKind::Comma) => self.token_current(),
                Some(_)
                    if saw_item && self.current_starts_missing_semicolon_declaration(recovery) =>
                {
                    self.error_at_current(
                        ParseErrorCode::UnexpectedCharacter,
                        "expected semicolon between declarations",
                    );
                    break;
                }
                Some(_) => {
                    self.parse_value_expression(0, &item_recovery);
                    saw_item = true;
                }
                None => break,
            }
        }
        self.builder.finish_node();
    }

    fn parse_value_list_until(&mut self, recovery: &[SyntaxKind]) {
        self.builder
            .start_node(if self.current_value_list_is_bogus(recovery) {
                SyntaxKind::BogusValueList
            } else {
                SyntaxKind::ValueList
            });
        let item_recovery = value_list_item_recovery(recovery);
        while !self.at_end() {
            self.eat_value_trivia();
            match self.current_kind() {
                Some(kind) if recovery.contains(&kind) => break,
                Some(SyntaxKind::Comma) => self.token_current(),
                Some(_) => self.parse_value_expression(0, &item_recovery),
                None => break,
            }
        }
        self.builder.finish_node();
    }

    fn parse_component_value(&mut self, recovery: &[SyntaxKind]) {
        self.builder.start_node(SyntaxKind::ComponentValue);
        self.parse_component_value_inner(recovery);
        self.builder.finish_node();
    }

    fn parse_component_value_list_until(&mut self, recovery: &[SyntaxKind]) {
        self.builder.start_node(SyntaxKind::ComponentValueList);
        while !self.at_end() {
            self.eat_value_trivia();
            match self.current_kind() {
                Some(kind) if recovery.contains(&kind) => break,
                Some(_) => self.parse_component_value(recovery),
                None => break,
            }
        }
        self.builder.finish_node();
    }

    fn parse_comma_separated_component_value_list_until(&mut self, recovery: &[SyntaxKind]) {
        self.builder
            .start_node(SyntaxKind::CommaSeparatedComponentValueList);
        let item_recovery = comma_separated_component_value_list_item_recovery(recovery);
        while !self.at_end() {
            self.eat_value_trivia();
            match self.current_kind() {
                Some(kind) if recovery.contains(&kind) => break,
                Some(SyntaxKind::Comma) => self.token_current(),
                Some(_) => self.parse_component_value(&item_recovery),
                None => break,
            }
        }
        self.builder.finish_node();
    }

    fn parse_component_value_inner(&mut self, recovery: &[SyntaxKind]) {
        self.eat_value_trivia();
        match self.current_kind() {
            Some(kind) if recovery.contains(&kind) => {
                self.empty_bogus_node(
                    SyntaxKind::BogusValue,
                    ParseErrorCode::ExpectedValue,
                    "expected component value",
                );
            }
            Some(SyntaxKind::LeftBrace | SyntaxKind::LeftBracket | SyntaxKind::LeftParen) => {
                self.parse_simple_block(recovery)
            }
            Some(SyntaxKind::Ident) if self.next_kind() == Some(SyntaxKind::LeftParen) => {
                self.parse_function_call(recovery)
            }
            Some(kind) if is_component_value_atom_start(kind) => self.parse_value_prefix(recovery),
            Some(_) => self.token_current(),
            None => {
                self.empty_bogus_node(
                    SyntaxKind::BogusValue,
                    ParseErrorCode::ExpectedValue,
                    "expected component value",
                );
            }
        }
    }

    fn parse_simple_block_entry_point(&mut self, recovery: &[SyntaxKind]) {
        self.eat_value_trivia();
        match self.current_kind() {
            Some(SyntaxKind::LeftBrace | SyntaxKind::LeftBracket | SyntaxKind::LeftParen) => {
                self.parse_simple_block(recovery)
            }
            Some(_) | None => {
                self.empty_bogus_node(
                    SyntaxKind::BogusSimpleBlock,
                    ParseErrorCode::ExpectedValue,
                    "expected simple block",
                );
            }
        }
    }

    fn parse_simple_block(&mut self, recovery: &[SyntaxKind]) {
        let Some(open_kind) = self.current_kind() else {
            self.empty_bogus_node(
                SyntaxKind::BogusSimpleBlock,
                ParseErrorCode::ExpectedValue,
                "expected simple block",
            );
            return;
        };
        let Some(close_kind) = matching_simple_block_close(open_kind) else {
            self.empty_bogus_node(
                SyntaxKind::BogusSimpleBlock,
                ParseErrorCode::ExpectedValue,
                "expected simple block",
            );
            return;
        };

        let block_kind = if self.current_simple_block_has_matching_close(recovery) {
            SyntaxKind::SimpleBlock
        } else {
            SyntaxKind::BogusSimpleBlock
        };
        self.builder.start_node(block_kind);
        self.token_current();

        let block_recovery = simple_block_recovery(close_kind, recovery);
        while !self.at_end() {
            self.eat_value_trivia();
            match self.current_kind() {
                Some(kind) if kind == close_kind => break,
                Some(kind) if recovery.contains(&kind) => break,
                Some(_) => self.parse_component_value(&block_recovery),
                None => break,
            }
        }

        if self.current_kind() == Some(close_kind) {
            self.token_current();
        } else {
            self.error_at_current(
                ParseErrorCode::UnexpectedCharacter,
                "unterminated simple block",
            );
        }
        self.builder.finish_node();
    }

    fn parse_value_expression(&mut self, min_binding_power: u8, recovery: &[SyntaxKind]) {
        self.eat_value_trivia();
        let checkpoint = self.builder.checkpoint();
        self.parse_value_prefix(recovery);

        loop {
            self.eat_value_trivia();
            let Some(operator) = self.current_kind() else {
                break;
            };
            if recovery.contains(&operator) {
                break;
            }
            let Some((left_binding_power, right_binding_power)) = infix_binding_power(operator)
            else {
                break;
            };
            if left_binding_power < min_binding_power {
                break;
            }

            self.builder
                .start_node_at(checkpoint, SyntaxKind::BinaryExpression);
            self.token_current();
            self.parse_value_expression(right_binding_power, recovery);
            self.builder.finish_node();
        }
    }

    fn parse_value_prefix(&mut self, recovery: &[SyntaxKind]) {
        match self.current_kind() {
            Some(SyntaxKind::Plus | SyntaxKind::Minus) => {
                self.builder.start_node(SyntaxKind::UnaryExpression);
                self.token_current();
                self.parse_value_expression(5, recovery);
                self.builder.finish_node();
            }
            Some(SyntaxKind::Ident)
                if self
                    .current_text()
                    .is_some_and(|text| text.eq_ignore_ascii_case("url"))
                    && self.next_kind() == Some(SyntaxKind::LeftParen) =>
            {
                self.builder.start_node(SyntaxKind::UrlValue);
                self.parse_function_call(recovery);
                self.builder.finish_node();
            }
            Some(SyntaxKind::Ident) if self.next_kind() == Some(SyntaxKind::LeftParen) => {
                self.parse_function_call(recovery)
            }
            Some(SyntaxKind::Number) => {
                self.builder.start_node(SyntaxKind::NumberValue);
                self.token_current();
                self.builder.finish_node();
            }
            Some(SyntaxKind::Percentage) => {
                self.builder.start_node(SyntaxKind::PercentageValue);
                self.token_current();
                self.builder.finish_node();
            }
            Some(SyntaxKind::Dimension) => {
                self.builder.start_node(SyntaxKind::DimensionValue);
                self.token_current();
                self.builder.finish_node();
            }
            Some(SyntaxKind::Ident | SyntaxKind::CustomPropertyName) => {
                self.builder.start_node(SyntaxKind::IdentifierValue);
                self.token_current();
                self.builder.finish_node();
            }
            Some(SyntaxKind::String | SyntaxKind::LessEscapedString) => {
                self.builder.start_node(SyntaxKind::StringValue);
                self.token_current();
                self.builder.finish_node();
            }
            Some(SyntaxKind::UnicodeRange) => {
                self.builder.start_node(SyntaxKind::UnicodeRangeValue);
                self.token_current();
                self.builder.finish_node();
            }
            Some(SyntaxKind::Hash) => {
                self.builder.start_node(SyntaxKind::ColorValue);
                self.token_current();
                self.builder.finish_node();
            }
            Some(SyntaxKind::Url) => {
                self.builder.start_node(SyntaxKind::UrlValue);
                self.token_current();
                self.builder.finish_node();
            }
            Some(SyntaxKind::BadUrl) => {
                self.builder.start_node(SyntaxKind::BogusValue);
                self.token_current();
                self.builder.finish_node();
            }
            Some(SyntaxKind::BadString) => {
                self.builder.start_node(SyntaxKind::BogusValue);
                self.token_current();
                self.builder.finish_node();
            }
            Some(SyntaxKind::Important) => {
                self.builder.start_node(SyntaxKind::ImportantAnnotation);
                self.token_current();
                self.builder.finish_node();
            }
            Some(SyntaxKind::Delim) if self.current_split_important_annotation() => {
                self.parse_split_important_annotation()
            }
            Some(SyntaxKind::Delim) if self.current_scss_variable_flag_annotation() => {
                self.parse_scss_variable_flag_annotation()
            }
            Some(kind) if is_interpolation_start(kind) => self.parse_interpolation(kind, recovery),
            Some(SyntaxKind::ScssVariable) => {
                self.builder.start_node(SyntaxKind::ScssVariableReference);
                self.token_current();
                self.builder.finish_node();
            }
            Some(SyntaxKind::LessVariable) => {
                self.builder.start_node(SyntaxKind::LessVariableReference);
                self.token_current();
                self.builder.finish_node();
            }
            Some(SyntaxKind::LessPropertyVariableToken) => {
                self.builder.start_node(SyntaxKind::LessPropertyVariable);
                self.token_current();
                self.builder.finish_node();
            }
            Some(SyntaxKind::LeftBrace) => self.parse_simple_block(recovery),
            Some(SyntaxKind::LeftParen) => self.parse_parenthesized_expression(recovery),
            Some(SyntaxKind::LeftBracket) => self.parse_bracketed_value(recovery),
            Some(kind) if recovery.contains(&kind) => {
                self.empty_bogus_node(
                    SyntaxKind::BogusValue,
                    ParseErrorCode::ExpectedValue,
                    "expected value",
                );
            }
            Some(SyntaxKind::Delim) => {
                self.builder.start_node(SyntaxKind::BogusToken);
                self.token_current();
                self.builder.finish_node();
            }
            Some(_) => {
                self.builder.start_node(SyntaxKind::BogusValue);
                self.error_at_current(ParseErrorCode::ExpectedValue, "expected value");
                self.token_current();
                self.builder.finish_node();
            }
            None => {
                self.empty_bogus_node(
                    SyntaxKind::BogusValue,
                    ParseErrorCode::ExpectedValue,
                    "expected value",
                );
            }
        }
    }

    fn parse_split_important_annotation(&mut self) {
        self.builder.start_node(SyntaxKind::ImportantAnnotation);
        self.token_current();
        self.eat_value_trivia();
        if self
            .current_text()
            .is_some_and(|text| text.eq_ignore_ascii_case("important"))
        {
            self.token_current();
        }
        self.builder.finish_node();
    }

    fn parse_scss_variable_flag_annotation(&mut self) {
        self.builder.start_node(SyntaxKind::ScssVariableFlag);
        self.token_current();
        self.eat_value_trivia();
        self.token_current();
        self.builder.finish_node();
    }

    fn eat_value_trivia(&mut self) {
        while matches!(self.current_kind(), Some(kind) if kind.is_trivia()) {
            self.token_current();
        }
    }

    fn parse_function_call(&mut self, recovery: &[SyntaxKind]) {
        let function_name = self.current_text().map(str::to_owned);
        let function_range = self.current_range();
        let argument_count = self.current_function_top_level_argument_count_before(recovery);
        let has_empty_argument_slot =
            self.current_function_has_empty_top_level_argument_slot_before(recovery);
        let argument_head = self.current_function_first_argument_token_before(recovery);
        let specialized_kind = function_name.as_deref().and_then(specialized_function_kind);
        let closed = self.current_function_has_closing_paren_before(recovery);
        let function_kind = if closed {
            SyntaxKind::FunctionCall
        } else {
            SyntaxKind::BogusFunctionCall
        };
        let arguments_kind = if closed {
            SyntaxKind::FunctionArguments
        } else {
            SyntaxKind::BogusFunctionArguments
        };

        self.builder.start_node(function_kind);
        if let Some(kind) = specialized_kind {
            self.builder.start_node(kind);
        }
        self.token_current();
        if self.current_kind() == Some(SyntaxKind::LeftParen) {
            self.token_current();
            self.builder.start_node(arguments_kind);
            let argument_recovery = function_argument_recovery(recovery);
            self.parse_value_or_value_list_until(&argument_recovery);
            self.builder.finish_node();
            if self.current_kind() == Some(SyntaxKind::RightParen) {
                self.token_current();
            } else {
                self.error_at_current(
                    ParseErrorCode::UnexpectedCharacter,
                    "unterminated function call",
                );
            }
        }
        if let Some(function_name) = function_name {
            if let Some(argument_count) = argument_count {
                self.validate_function_argument_count(
                    &function_name,
                    argument_count,
                    function_range,
                );
            }
            if let Some(true) = has_empty_argument_slot {
                self.validate_function_argument_slots(&function_name, function_range);
            }
            self.validate_function_argument_head(&function_name, argument_head, function_range);
        }
        if specialized_kind.is_some() {
            self.builder.finish_node();
        }
        self.builder.finish_node();
    }

    fn current_function_top_level_argument_count_before(
        &self,
        recovery: &[SyntaxKind],
    ) -> Option<usize> {
        if self.next_kind() != Some(SyntaxKind::LeftParen) {
            return None;
        }

        let mut index = self.position + 2;
        let mut depth = 0usize;
        let mut comma_count = 0usize;
        let mut saw_argument = false;
        while let Some(token) = self.tokens.get(index) {
            match token.kind {
                kind if depth == 0 && recovery.contains(&kind) => return None,
                SyntaxKind::RightParen if depth == 0 => {
                    return Some(if saw_argument { comma_count + 1 } else { 0 });
                }
                SyntaxKind::Comma if depth == 0 => {
                    comma_count += 1;
                    saw_argument = false;
                }
                kind if kind.is_trivia() => {}
                SyntaxKind::LeftBrace | SyntaxKind::LeftBracket | SyntaxKind::LeftParen => {
                    depth += 1;
                    saw_argument = true;
                }
                SyntaxKind::RightBrace | SyntaxKind::RightBracket | SyntaxKind::RightParen => {
                    depth = depth.saturating_sub(1);
                    saw_argument = true;
                }
                _ => saw_argument = true,
            }
            index += 1;
        }
        None
    }

    fn current_function_has_empty_top_level_argument_slot_before(
        &self,
        recovery: &[SyntaxKind],
    ) -> Option<bool> {
        if self.next_kind() != Some(SyntaxKind::LeftParen) {
            return None;
        }

        let mut index = self.position + 2;
        let mut depth = 0usize;
        let mut expecting_argument = true;
        let mut saw_argument = false;
        while let Some(token) = self.tokens.get(index) {
            match token.kind {
                kind if depth == 0 && recovery.contains(&kind) => return None,
                SyntaxKind::RightParen if depth == 0 => {
                    return Some(expecting_argument && saw_argument);
                }
                SyntaxKind::Comma if depth == 0 => {
                    if expecting_argument {
                        return Some(true);
                    }
                    expecting_argument = true;
                }
                kind if kind.is_trivia() => {}
                SyntaxKind::LeftBrace | SyntaxKind::LeftBracket | SyntaxKind::LeftParen => {
                    depth += 1;
                    expecting_argument = false;
                    saw_argument = true;
                }
                SyntaxKind::RightBrace | SyntaxKind::RightBracket | SyntaxKind::RightParen => {
                    depth = depth.saturating_sub(1);
                    expecting_argument = false;
                    saw_argument = true;
                }
                _ => {
                    expecting_argument = false;
                    saw_argument = true;
                }
            }
            index += 1;
        }
        None
    }

    fn current_function_first_argument_token_before(
        &self,
        recovery: &[SyntaxKind],
    ) -> Option<Token<'text>> {
        if self.next_kind() != Some(SyntaxKind::LeftParen) {
            return None;
        }

        let mut index = self.position + 2;
        while let Some(token) = self.tokens.get(index).copied() {
            match token.kind {
                kind if recovery.contains(&kind) => return None,
                SyntaxKind::RightParen => return None,
                kind if kind.is_trivia() => {}
                _ => return Some(token),
            }
            index += 1;
        }
        None
    }

    fn validate_function_argument_count(
        &mut self,
        function_name: &str,
        argument_count: usize,
        range: TextRange,
    ) {
        if function_argument_count_is_valid(function_name, argument_count) {
            return;
        }
        self.errors.push(ParseError {
            code: ParseErrorCode::ExpectedValue,
            range,
            message: "invalid function argument count",
        });
    }

    fn validate_function_argument_slots(&mut self, function_name: &str, range: TextRange) {
        if !function_requires_filled_top_level_arguments(function_name) {
            return;
        }
        self.errors.push(ParseError {
            code: ParseErrorCode::ExpectedValue,
            range,
            message: "empty function argument",
        });
    }

    fn validate_function_argument_head(
        &mut self,
        function_name: &str,
        argument_head: Option<Token<'text>>,
        range: TextRange,
    ) {
        let head_kind = argument_head.map(|token| token.kind);
        let valid = if function_name.eq_ignore_ascii_case("var") {
            matches!(head_kind, Some(SyntaxKind::CustomPropertyName))
                || head_kind.is_some_and(is_dynamic_function_argument_head)
        } else if function_name.eq_ignore_ascii_case("env") {
            matches!(
                head_kind,
                Some(SyntaxKind::Ident | SyntaxKind::CustomPropertyName)
            ) || head_kind.is_some_and(is_dynamic_function_argument_head)
        } else if function_name.eq_ignore_ascii_case("attr") {
            matches!(head_kind, Some(SyntaxKind::Ident))
                || head_kind.is_some_and(is_dynamic_function_argument_head)
        } else if function_name.eq_ignore_ascii_case("color-mix") {
            argument_head.is_some_and(|token| token.text.eq_ignore_ascii_case("in"))
                || head_kind.is_some_and(is_dynamic_function_argument_head)
        } else {
            true
        };

        if valid {
            return;
        }
        self.errors.push(ParseError {
            code: ParseErrorCode::ExpectedValue,
            range,
            message: "invalid function argument head",
        });
    }

    fn parse_bracketed_value(&mut self, recovery: &[SyntaxKind]) {
        let closed = self.current_bracketed_value_has_closing_bracket_before(recovery);
        self.builder.start_node(if closed {
            SyntaxKind::BracketedValue
        } else {
            SyntaxKind::BogusBracketedValue
        });
        self.token_current();
        let bracket_recovery = bracketed_value_recovery(recovery);
        self.parse_value_until(&bracket_recovery);
        if self.current_kind() == Some(SyntaxKind::RightBracket) {
            self.token_current();
        } else {
            self.error_at_current(
                ParseErrorCode::UnexpectedCharacter,
                "unterminated bracketed value",
            );
        }
        self.builder.finish_node();
    }

    fn parse_parenthesized_expression(&mut self, recovery: &[SyntaxKind]) {
        self.builder.start_node(SyntaxKind::ParenthesizedExpression);
        self.token_current();
        let paren_recovery = function_argument_recovery(recovery);
        self.parse_value_until(&paren_recovery);
        if self.current_kind() == Some(SyntaxKind::RightParen) {
            self.token_current();
        }
        self.builder.finish_node();
    }

    fn parse_at_rule(&mut self) {
        let spec = self.current_text().and_then(at_rule_spec);
        let at_rule_kind = if spec.is_none() && self.current_text() == Some("@") {
            SyntaxKind::BogusAtRule
        } else {
            SyntaxKind::AtRule
        };
        self.builder.start_node(at_rule_kind);
        if at_rule_kind == SyntaxKind::BogusAtRule {
            self.error_at_current(ParseErrorCode::UnexpectedCharacter, "expected at-rule name");
        }
        if let Some(spec) = spec {
            self.builder.start_node(spec.node_kind);
        }

        if self.current_kind() == Some(SyntaxKind::AtKeyword) {
            self.token_current();
        }
        if let Some(spec) = spec {
            self.parse_at_rule_prelude(spec.node_kind);
        } else {
            self.consume_at_rule_prelude_tokens();
        }

        while !self.at_end() {
            match self.current_kind() {
                Some(kind) if is_statement_end(kind) => {
                    self.token_current();
                    break;
                }
                Some(SyntaxKind::LeftBrace) => {
                    match spec
                        .map(|spec| spec.block_kind)
                        .unwrap_or(AtRuleBlockKind::Raw)
                    {
                        AtRuleBlockKind::GroupRuleList => self.parse_group_at_rule_block(),
                        AtRuleBlockKind::DeclarationList => self.parse_declaration_block(),
                        AtRuleBlockKind::Keyframes => self.parse_keyframes_block(),
                        AtRuleBlockKind::Raw => self.consume_balanced_block(),
                    }
                    break;
                }
                Some(SyntaxKind::SassIndent) => {
                    self.parse_sass_indented_at_rule_block(
                        spec.map(|spec| spec.block_kind)
                            .unwrap_or(AtRuleBlockKind::Raw),
                    );
                    break;
                }
                Some(_) => self.token_current(),
                None => break,
            }
        }

        if spec.is_some() {
            self.builder.finish_node();
        }
        self.builder.finish_node();
    }

    fn parse_at_rule_prelude(&mut self, node_kind: SyntaxKind) {
        match node_kind {
            SyntaxKind::MediaRule => self.parse_media_query_list(),
            SyntaxKind::SupportsRule => self.parse_supports_rule_prelude(),
            SyntaxKind::ContainerRule => self.parse_container_rule_prelude(),
            SyntaxKind::ImportRule => self.parse_import_prelude(),
            SyntaxKind::CharsetRule => self.parse_charset_rule_prelude(),
            SyntaxKind::NamespaceRule => self.parse_namespace_rule_prelude(),
            SyntaxKind::KeyframesRule => self.parse_keyframes_rule_prelude(),
            SyntaxKind::PageRule => self.parse_page_rule_prelude(),
            SyntaxKind::FontFaceRule
            | SyntaxKind::StartingStyleRule
            | SyntaxKind::PageMarginRule
            | SyntaxKind::FontFeatureValuesStylisticRule
            | SyntaxKind::FontFeatureValuesStylesetRule
            | SyntaxKind::FontFeatureValuesCharacterVariantRule
            | SyntaxKind::FontFeatureValuesSwashRule
            | SyntaxKind::FontFeatureValuesOrnamentsRule
            | SyntaxKind::FontFeatureValuesAnnotationRule
            | SyntaxKind::FontFeatureValuesHistoricalFormsRule
            | SyntaxKind::ViewTransitionRule => {
                self.parse_empty_at_rule_prelude("unexpected at-rule prelude")
            }
            SyntaxKind::PropertyRule => self.parse_named_at_rule_prelude(
                at_rule_prelude_head_is_custom_property_name,
                "invalid @property name",
            ),
            SyntaxKind::FontPaletteValuesRule
            | SyntaxKind::ColorProfileRule
            | SyntaxKind::PositionTryRule => self.parse_named_at_rule_prelude(
                at_rule_prelude_head_is_custom_property_name,
                "invalid at-rule custom property name",
            ),
            SyntaxKind::CustomMediaRule => self.parse_custom_media_rule_prelude(),
            SyntaxKind::CounterStyleRule => self.parse_named_at_rule_prelude(
                at_rule_prelude_head_is_custom_ident,
                "invalid @counter-style name",
            ),
            SyntaxKind::FontFeatureValuesRule => self.parse_font_feature_values_prelude(),
            SyntaxKind::LayerRule => self.parse_layer_rule_prelude(),
            SyntaxKind::ScopeRule => self.parse_scope_rule_prelude(),
            _ => self.consume_at_rule_prelude_tokens(),
        }
    }

    fn parse_media_query_list(&mut self) {
        self.builder.start_node(SyntaxKind::MediaQueryList);
        let mut saw_query = false;
        let mut expecting_query = true;
        while !self.at_end() {
            match self.current_kind() {
                Some(kind) if is_at_rule_prelude_boundary(kind) => break,
                Some(SyntaxKind::Comma) => {
                    if expecting_query {
                        self.error_at_current(
                            ParseErrorCode::ExpectedValue,
                            "invalid @media prelude",
                        );
                        self.builder.start_node(SyntaxKind::BogusMediaQuery);
                        self.token_current();
                        self.builder.finish_node();
                    } else {
                        self.token_current();
                        expecting_query = true;
                    }
                }
                Some(_) => {
                    let valid = self.current_media_query_is_valid();
                    if !valid {
                        self.error_at_current(
                            ParseErrorCode::ExpectedValue,
                            "invalid @media prelude",
                        );
                    }
                    self.parse_media_query(valid);
                    saw_query = true;
                    expecting_query = false;
                }
                None => break,
            }
        }
        if !saw_query || expecting_query {
            self.error_at_current(ParseErrorCode::ExpectedValue, "invalid @media prelude");
            self.builder.start_node(SyntaxKind::BogusMediaQuery);
            self.builder.finish_node();
        }
        self.builder.finish_node();
    }

    fn parse_media_query(&mut self, valid: bool) {
        self.builder.start_node(if valid {
            SyntaxKind::MediaQuery
        } else {
            SyntaxKind::BogusMediaQuery
        });
        while !self.at_end() {
            match self.current_kind() {
                Some(kind) if is_at_rule_prelude_boundary(kind) || kind == SyntaxKind::Comma => {
                    break;
                }
                Some(SyntaxKind::LeftParen) => self.parse_balanced_parenthesized_prelude_until(
                    Some(SyntaxKind::MediaFeature),
                    &[
                        SyntaxKind::Comma,
                        SyntaxKind::LeftBrace,
                        SyntaxKind::Semicolon,
                    ],
                ),
                Some(kind) if is_interpolation_start(kind) => self.parse_interpolation(
                    kind,
                    &[
                        SyntaxKind::Comma,
                        SyntaxKind::LeftBrace,
                        SyntaxKind::Semicolon,
                    ],
                ),
                Some(_) => self.token_current(),
                None => break,
            }
        }
        self.builder.finish_node();
    }

    fn current_media_query_is_valid(&self) -> bool {
        let Some((first_index, first_kind)) = self.non_trivia_token_from(self.position) else {
            return false;
        };
        if is_at_rule_prelude_boundary(first_kind) || first_kind == SyntaxKind::Comma {
            return false;
        }
        if !self.current_prelude_parentheses_are_balanced_until(&[
            SyntaxKind::Comma,
            SyntaxKind::LeftBrace,
            SyntaxKind::SassIndent,
            SyntaxKind::Semicolon,
            SyntaxKind::SassOptionalSemicolon,
        ]) {
            return false;
        }
        self.media_query_starts_at(first_index, first_kind)
    }

    fn media_query_starts_at(&self, index: usize, kind: SyntaxKind) -> bool {
        match kind {
            SyntaxKind::Ident | SyntaxKind::LeftParen => true,
            SyntaxKind::KeywordNot | SyntaxKind::KeywordOnly => self
                .non_trivia_token_from(index + 1)
                .is_some_and(|(_, next_kind)| {
                    matches!(next_kind, SyntaxKind::Ident | SyntaxKind::LeftParen)
                        || is_interpolation_start(next_kind)
                }),
            kind if is_interpolation_start(kind) => true,
            _ => false,
        }
    }

    fn parse_charset_rule_prelude(&mut self) {
        if !self.charset_rule_prelude_is_valid() {
            self.error_at_current(ParseErrorCode::ExpectedValue, "invalid @charset prelude");
        }
        self.consume_at_rule_prelude_tokens();
    }

    fn charset_rule_prelude_is_valid(&self) -> bool {
        let Some((source_index, SyntaxKind::String)) = self.non_trivia_token_from(self.position)
        else {
            return false;
        };
        self.non_trivia_token_from(source_index + 1)
            .is_none_or(|(_, kind)| is_at_rule_prelude_boundary(kind))
    }

    fn parse_namespace_rule_prelude(&mut self) {
        if !self.namespace_rule_prelude_is_valid() {
            self.error_at_current(ParseErrorCode::ExpectedValue, "invalid @namespace prelude");
        }
        self.consume_at_rule_prelude_tokens();
    }

    fn parse_custom_media_rule_prelude(&mut self) {
        self.eat_trivia();
        let valid = self.custom_media_rule_prelude_is_valid();
        if !valid {
            self.error_at_current(
                ParseErrorCode::ExpectedValue,
                "invalid @custom-media prelude",
            );
        }
        self.builder.start_node(if valid {
            SyntaxKind::AtRulePrelude
        } else {
            SyntaxKind::BogusAtRulePrelude
        });
        self.consume_at_rule_prelude_tokens_without_wrapping();
        self.builder.finish_node();
    }

    fn custom_media_rule_prelude_is_valid(&self) -> bool {
        let Some((name_index, name_kind)) = self.non_trivia_token_from(self.position) else {
            return false;
        };
        if !self.current_prelude_parentheses_are_balanced_until(&[
            SyntaxKind::Semicolon,
            SyntaxKind::SassOptionalSemicolon,
        ]) {
            return false;
        }
        let tail = if name_kind == SyntaxKind::CustomPropertyName {
            self.non_trivia_token_from(name_index + 1)
        } else if is_interpolation_start(name_kind) {
            self.non_trivia_token_after_interpolation(name_index, name_kind)
        } else {
            return false;
        };
        let Some((tail_index, tail_kind)) = tail else {
            return false;
        };
        if is_at_rule_prelude_boundary(tail_kind) {
            return false;
        }
        self.media_query_starts_at(tail_index, tail_kind)
    }

    fn namespace_rule_prelude_is_valid(&self) -> bool {
        let Some((first_index, first_kind)) = self.non_trivia_token_from(self.position) else {
            return false;
        };

        if self.namespace_source_starts_at(first_index, first_kind) {
            return true;
        }
        if !matches!(
            first_kind,
            SyntaxKind::Ident | SyntaxKind::CustomPropertyName
        ) {
            return false;
        }
        self.non_trivia_token_from(first_index + 1)
            .is_some_and(|(source_index, source_kind)| {
                self.namespace_source_starts_at(source_index, source_kind)
            })
    }

    fn namespace_source_starts_at(&self, index: usize, kind: SyntaxKind) -> bool {
        matches!(kind, SyntaxKind::String | SyntaxKind::Url)
            || is_interpolation_start(kind)
            || self.token_starts_url_function(index, kind)
    }

    fn token_starts_url_function(&self, index: usize, kind: SyntaxKind) -> bool {
        kind == SyntaxKind::Ident
            && self
                .tokens
                .get(index)
                .is_some_and(|token| token.text.eq_ignore_ascii_case("url"))
            && self
                .non_trivia_token_from(index + 1)
                .is_some_and(|(_, next_kind)| next_kind == SyntaxKind::LeftParen)
    }

    fn parse_keyframes_rule_prelude(&mut self) {
        if !self.keyframes_rule_prelude_is_valid() {
            self.error_at_current(ParseErrorCode::ExpectedValue, "invalid @keyframes name");
        }
        self.consume_at_rule_prelude_tokens();
    }

    fn keyframes_rule_prelude_is_valid(&self) -> bool {
        let Some((name_index, name_kind)) = self.non_trivia_token_from(self.position) else {
            return false;
        };
        if is_interpolation_start(name_kind) {
            return true;
        }
        if !matches!(name_kind, SyntaxKind::Ident | SyntaxKind::String) {
            return false;
        }
        self.non_trivia_token_from(name_index + 1)
            .is_none_or(|(_, kind)| is_at_rule_prelude_boundary(kind))
    }

    fn parse_empty_at_rule_prelude(&mut self, message: &'static str) {
        self.eat_trivia();
        if self
            .current_kind()
            .is_some_and(|kind| !is_at_rule_prelude_boundary(kind))
        {
            self.error_at_current(ParseErrorCode::ExpectedValue, message);
            self.consume_at_rule_prelude_tokens();
        }
    }

    fn parse_font_feature_values_prelude(&mut self) {
        if !self.font_feature_values_prelude_is_valid() {
            self.error_at_current(
                ParseErrorCode::ExpectedValue,
                "invalid @font-feature-values family name",
            );
        }
        self.consume_at_rule_prelude_tokens();
    }

    fn font_feature_values_prelude_is_valid(&self) -> bool {
        self.non_trivia_token_from(self.position)
            .is_some_and(|(_, kind)| {
                matches!(kind, SyntaxKind::Ident | SyntaxKind::String)
                    || is_interpolation_start(kind)
            })
    }

    fn parse_layer_rule_prelude(&mut self) {
        self.eat_trivia();
        match self.current_kind() {
            Some(SyntaxKind::LeftBrace | SyntaxKind::SassIndent) => return,
            Some(SyntaxKind::Semicolon | SyntaxKind::SassOptionalSemicolon) | None => {
                self.empty_bogus_node(
                    SyntaxKind::BogusLayerName,
                    ParseErrorCode::ExpectedValue,
                    "invalid @layer prelude",
                );
                return;
            }
            Some(_) => {}
        }

        let valid = self.layer_rule_prelude_is_valid();
        if !valid {
            self.error_at_current(ParseErrorCode::ExpectedValue, "invalid @layer prelude");
        }
        self.builder.start_node(if valid {
            SyntaxKind::LayerName
        } else {
            SyntaxKind::BogusLayerName
        });
        self.consume_at_rule_prelude_tokens_without_wrapping();
        self.builder.finish_node();
    }

    fn layer_rule_prelude_is_valid(&self) -> bool {
        let mut saw_name = false;
        let mut expecting_segment = true;
        let mut index = self.position;

        while let Some(token) = self.tokens.get(index) {
            if token.kind.is_trivia() {
                index += 1;
                continue;
            }
            if is_at_rule_prelude_boundary(token.kind) {
                return saw_name && !expecting_segment;
            }
            if is_interpolation_start(token.kind) {
                return true;
            }
            match token.kind {
                SyntaxKind::Ident if expecting_segment => {
                    saw_name = true;
                    expecting_segment = false;
                }
                SyntaxKind::Comma if saw_name && !expecting_segment => {
                    expecting_segment = true;
                }
                SyntaxKind::Dot if saw_name && !expecting_segment => {
                    expecting_segment = true;
                }
                _ => return false,
            }
            index += 1;
        }

        saw_name && !expecting_segment
    }

    fn parse_container_rule_prelude(&mut self) {
        self.eat_trivia();
        let valid = self.container_rule_prelude_is_valid();
        if !valid {
            self.error_at_current(ParseErrorCode::ExpectedValue, "invalid @container prelude");
        }
        self.builder.start_node(if valid {
            SyntaxKind::ContainerCondition
        } else {
            SyntaxKind::BogusContainerCondition
        });
        self.consume_at_rule_prelude_tokens_without_wrapping();
        self.builder.finish_node();
    }

    fn container_rule_prelude_is_valid(&self) -> bool {
        let Some((first_index, first_kind)) = self.non_trivia_token_from(self.position) else {
            return false;
        };
        if is_at_rule_prelude_boundary(first_kind) {
            return false;
        }
        if !self.current_prelude_parentheses_are_balanced_until(&[
            SyntaxKind::LeftBrace,
            SyntaxKind::SassIndent,
            SyntaxKind::Semicolon,
            SyntaxKind::SassOptionalSemicolon,
        ]) {
            return false;
        }
        if self.container_condition_starts_at(first_index, first_kind) {
            return true;
        }
        if first_kind != SyntaxKind::Ident {
            return false;
        }
        self.non_trivia_token_from(first_index + 1).is_some_and(
            |(condition_index, condition_kind)| {
                self.container_condition_starts_at(condition_index, condition_kind)
            },
        )
    }

    fn container_condition_starts_at(&self, index: usize, kind: SyntaxKind) -> bool {
        if matches!(kind, SyntaxKind::LeftParen | SyntaxKind::KeywordNot)
            || is_interpolation_start(kind)
        {
            return true;
        }
        kind == SyntaxKind::Ident
            && self
                .non_trivia_token_from(index + 1)
                .is_some_and(|(_, next_kind)| next_kind == SyntaxKind::LeftParen)
    }

    fn parse_supports_rule_prelude(&mut self) {
        self.eat_trivia();
        let valid = self.supports_rule_prelude_is_valid();
        if !valid {
            self.error_at_current(ParseErrorCode::ExpectedValue, "invalid @supports prelude");
        }
        self.builder.start_node(if valid {
            SyntaxKind::SupportsCondition
        } else {
            SyntaxKind::BogusSupportsCondition
        });
        self.consume_at_rule_prelude_tokens_without_wrapping();
        self.builder.finish_node();
    }

    fn supports_rule_prelude_is_valid(&self) -> bool {
        let Some((first_index, first_kind)) = self.non_trivia_token_from(self.position) else {
            return false;
        };
        if is_at_rule_prelude_boundary(first_kind) {
            return false;
        }
        if !self.current_prelude_parentheses_are_balanced_until(&[
            SyntaxKind::LeftBrace,
            SyntaxKind::SassIndent,
            SyntaxKind::Semicolon,
            SyntaxKind::SassOptionalSemicolon,
        ]) {
            return false;
        }
        self.supports_condition_starts_at(first_index, first_kind)
    }

    fn supports_condition_starts_at(&self, index: usize, kind: SyntaxKind) -> bool {
        if kind == SyntaxKind::KeywordNot {
            return self
                .non_trivia_token_from(index + 1)
                .is_some_and(|(next_index, next_kind)| {
                    self.supports_condition_starts_at(next_index, next_kind)
                });
        }
        if kind == SyntaxKind::LeftParen || is_interpolation_start(kind) {
            return true;
        }
        kind == SyntaxKind::Ident
            && self
                .non_trivia_token_from(index + 1)
                .is_some_and(|(_, next_kind)| next_kind == SyntaxKind::LeftParen)
    }

    fn parse_scope_rule_prelude(&mut self) {
        self.eat_trivia();
        let valid = self.scope_rule_prelude_is_valid();
        if !valid {
            self.error_at_current(ParseErrorCode::ExpectedValue, "invalid @scope prelude");
        }
        self.builder.start_node(if valid {
            SyntaxKind::ScopeRange
        } else {
            SyntaxKind::BogusScopeRange
        });
        self.consume_at_rule_prelude_tokens_without_wrapping();
        self.builder.finish_node();
    }

    fn scope_rule_prelude_is_valid(&self) -> bool {
        let Some((start_index, start_kind)) = self.non_trivia_token_from(self.position) else {
            return false;
        };
        if is_at_rule_prelude_boundary(start_kind) {
            return false;
        }
        if !self.current_prelude_parentheses_are_balanced_until(&[
            SyntaxKind::LeftBrace,
            SyntaxKind::SassIndent,
            SyntaxKind::Semicolon,
            SyntaxKind::SassOptionalSemicolon,
        ]) {
            return false;
        }
        if is_interpolation_start(start_kind) {
            return true;
        }
        if start_kind != SyntaxKind::LeftParen {
            return false;
        }

        let Some(start_close_index) = self.parenthesized_prelude_close_index(start_index) else {
            return false;
        };
        let Some((after_start_index, after_start_kind)) =
            self.non_trivia_token_from(start_close_index + 1)
        else {
            return true;
        };
        if is_at_rule_prelude_boundary(after_start_kind) {
            return true;
        }
        if after_start_kind != SyntaxKind::Ident
            || !self
                .tokens
                .get(after_start_index)
                .is_some_and(|token| token.text.eq_ignore_ascii_case("to"))
        {
            return false;
        }

        let Some((end_index, end_kind)) = self.non_trivia_token_from(after_start_index + 1) else {
            return false;
        };
        if is_interpolation_start(end_kind) {
            return true;
        }
        if end_kind != SyntaxKind::LeftParen {
            return false;
        }
        let Some(end_close_index) = self.parenthesized_prelude_close_index(end_index) else {
            return false;
        };
        self.non_trivia_token_from(end_close_index + 1)
            .is_none_or(|(_, kind)| is_at_rule_prelude_boundary(kind))
    }

    fn parenthesized_prelude_close_index(&self, open_index: usize) -> Option<usize> {
        let mut depth = 0usize;
        for (index, token) in self.tokens.iter().enumerate().skip(open_index) {
            match token.kind {
                SyntaxKind::LeftParen => depth += 1,
                SyntaxKind::RightParen => {
                    depth = depth.saturating_sub(1);
                    if depth == 0 {
                        return Some(index);
                    }
                }
                kind if depth == 0 && is_at_rule_prelude_boundary(kind) => return None,
                _ => {}
            }
        }
        None
    }

    fn parse_page_rule_prelude(&mut self) {
        self.eat_trivia();
        if self.current_kind().is_none_or(is_at_rule_prelude_boundary) {
            return;
        }
        let valid = self.page_rule_prelude_is_valid();
        if !valid {
            self.error_at_current(ParseErrorCode::ExpectedValue, "invalid @page prelude");
        }
        self.builder.start_node(if valid {
            SyntaxKind::AtRulePrelude
        } else {
            SyntaxKind::BogusAtRulePrelude
        });
        self.consume_at_rule_prelude_tokens_without_wrapping();
        self.builder.finish_node();
    }

    fn page_rule_prelude_is_valid(&self) -> bool {
        let mut expecting_selector = true;
        let mut expecting_pseudo_name = false;
        let mut saw_selector = false;

        for token in self.tokens.iter().skip(self.position) {
            if token.kind.is_trivia() {
                continue;
            }
            if is_at_rule_prelude_boundary(token.kind) {
                return saw_selector && !expecting_selector && !expecting_pseudo_name;
            }
            if is_interpolation_start(token.kind) {
                return true;
            }
            if expecting_pseudo_name {
                if token.kind != SyntaxKind::Ident {
                    return false;
                }
                saw_selector = true;
                expecting_selector = false;
                expecting_pseudo_name = false;
                continue;
            }
            match token.kind {
                SyntaxKind::Ident if expecting_selector => {
                    saw_selector = true;
                    expecting_selector = false;
                }
                SyntaxKind::Colon => {
                    expecting_pseudo_name = true;
                }
                SyntaxKind::Comma if saw_selector && !expecting_selector => {
                    expecting_selector = true;
                }
                _ => return false,
            }
        }

        saw_selector && !expecting_selector && !expecting_pseudo_name
    }

    fn parse_import_prelude(&mut self) {
        self.eat_trivia();
        if self.dialect == StyleDialect::Less && self.current_kind() == Some(SyntaxKind::LeftParen)
        {
            self.builder.start_node(SyntaxKind::AtRulePrelude);
            self.parse_balanced_parenthesized_prelude(None);
            self.builder.finish_node();
            self.eat_trivia();
        }
        if !self.parse_import_source() {
            self.parse_bogus_import_prelude();
            return;
        }
        while !self.at_end() {
            match self.current_kind() {
                Some(kind) if is_at_rule_prelude_boundary(kind) => break,
                Some(kind) if kind.is_trivia() => self.token_current(),
                Some(SyntaxKind::Ident) if self.current_text() == Some("layer") => {
                    self.parse_import_layer_tail_node()
                }
                Some(SyntaxKind::Ident) if self.current_text() == Some("supports") => {
                    self.parse_import_supports_tail_node()
                }
                Some(_) => {
                    self.parse_media_query_list();
                    break;
                }
                None => break,
            }
        }
    }

    fn parse_import_source(&mut self) -> bool {
        match self.current_kind() {
            Some(SyntaxKind::Url) => {
                self.builder.start_node(SyntaxKind::UrlValue);
                self.token_current();
                self.builder.finish_node();
                true
            }
            Some(SyntaxKind::Ident)
                if self
                    .current_text()
                    .is_some_and(|text| text.eq_ignore_ascii_case("url"))
                    && self.next_kind() == Some(SyntaxKind::LeftParen) =>
            {
                self.builder.start_node(SyntaxKind::UrlValue);
                self.parse_function_call(&[SyntaxKind::LeftBrace, SyntaxKind::Semicolon]);
                self.builder.finish_node();
                true
            }
            Some(SyntaxKind::String) => {
                self.token_current();
                true
            }
            Some(kind) if is_interpolation_start(kind) => {
                self.parse_interpolation(kind, &[SyntaxKind::LeftBrace, SyntaxKind::Semicolon]);
                true
            }
            Some(_) | None => false,
        }
    }

    fn parse_bogus_import_prelude(&mut self) {
        self.builder.start_node(SyntaxKind::BogusAtRulePrelude);
        self.error_at_current(ParseErrorCode::ExpectedValue, "invalid @import source");
        self.consume_at_rule_prelude_tokens_without_wrapping();
        self.builder.finish_node();
    }

    fn parse_named_at_rule_prelude(
        &mut self,
        valid_head: fn(SyntaxKind) -> bool,
        message: &'static str,
    ) {
        if self.current_kind().is_none_or(is_at_rule_prelude_boundary) {
            return;
        }
        let valid_name = self
            .non_trivia_token_from(self.position)
            .is_some_and(|(_, kind)| valid_head(kind));
        if !valid_name {
            self.error_at_current(ParseErrorCode::ExpectedValue, message);
        }
        self.consume_at_rule_prelude_tokens();
    }

    fn parse_import_layer_tail_node(&mut self) {
        let valid = self.import_layer_tail_is_valid();
        if !valid {
            self.error_at_current(ParseErrorCode::ExpectedValue, "invalid @import layer tail");
        }
        self.builder.start_node(if valid {
            SyntaxKind::LayerName
        } else {
            SyntaxKind::BogusLayerName
        });
        self.token_current();
        if self.current_kind() == Some(SyntaxKind::LeftParen) {
            self.parse_balanced_parenthesized_prelude(None);
        }
        self.builder.finish_node();
    }

    fn import_layer_tail_is_valid(&self) -> bool {
        let Some((open_index, next_kind)) = self.non_trivia_token_from(self.position + 1) else {
            return true;
        };
        if next_kind != SyntaxKind::LeftParen {
            return true;
        }
        let Some(close_index) = self.parenthesized_prelude_close_index(open_index) else {
            return false;
        };
        self.layer_name_is_valid_between(open_index + 1, close_index)
    }

    fn layer_name_is_valid_between(&self, start: usize, end: usize) -> bool {
        let mut saw_name = false;
        let mut expecting_segment = true;

        for token in self.tokens[start..end]
            .iter()
            .filter(|token| !token.kind.is_trivia())
        {
            if is_interpolation_start(token.kind) {
                return true;
            }
            match token.kind {
                SyntaxKind::Ident if expecting_segment => {
                    saw_name = true;
                    expecting_segment = false;
                }
                SyntaxKind::Dot if saw_name && !expecting_segment => {
                    expecting_segment = true;
                }
                _ => return false,
            }
        }

        saw_name && !expecting_segment
    }

    fn parse_import_supports_tail_node(&mut self) {
        let valid = self.import_supports_tail_is_valid();
        if !valid {
            self.error_at_current(
                ParseErrorCode::ExpectedValue,
                "invalid @import supports tail",
            );
        }
        self.builder.start_node(if valid {
            SyntaxKind::SupportsCondition
        } else {
            SyntaxKind::BogusSupportsCondition
        });
        self.token_current();
        if self.current_kind() == Some(SyntaxKind::LeftParen) {
            self.parse_balanced_parenthesized_prelude(None);
        }
        self.builder.finish_node();
    }

    fn import_supports_tail_is_valid(&self) -> bool {
        let Some((open_index, SyntaxKind::LeftParen)) =
            self.non_trivia_token_from(self.position + 1)
        else {
            return false;
        };
        let Some(close_index) = self.parenthesized_prelude_close_index(open_index) else {
            return false;
        };
        self.non_trivia_token_from(open_index + 1)
            .is_some_and(|(inner_index, inner_kind)| {
                inner_index < close_index && inner_kind != SyntaxKind::RightParen
            })
    }

    fn consume_at_rule_prelude_tokens(&mut self) {
        if self.current_kind().is_none_or(is_at_rule_prelude_boundary) {
            return;
        }
        self.builder
            .start_node(self.current_generic_at_rule_prelude_node_kind());
        self.consume_at_rule_prelude_tokens_without_wrapping();
        self.builder.finish_node();
    }

    fn consume_at_rule_prelude_tokens_without_wrapping(&mut self) {
        while !self.at_end() {
            match self.current_kind() {
                Some(kind) if is_at_rule_prelude_boundary(kind) => break,
                Some(SyntaxKind::LeftParen) => self.parse_balanced_parenthesized_prelude(None),
                Some(kind) if is_interpolation_start(kind) => {
                    self.parse_interpolation(kind, &[SyntaxKind::LeftBrace, SyntaxKind::Semicolon])
                }
                Some(_) => self.token_current(),
                None => break,
            }
        }
    }

    fn parse_balanced_parenthesized_prelude(&mut self, node_kind: Option<SyntaxKind>) {
        self.parse_balanced_parenthesized_prelude_until(
            node_kind,
            &[SyntaxKind::LeftBrace, SyntaxKind::Semicolon],
        );
    }

    fn parse_balanced_parenthesized_prelude_until(
        &mut self,
        node_kind: Option<SyntaxKind>,
        recovery: &[SyntaxKind],
    ) {
        if let Some(kind) = node_kind {
            self.builder.start_node(kind);
        }
        let mut depth = 0usize;
        let mut closed = false;
        while !self.at_end() {
            match self.current_kind() {
                Some(SyntaxKind::LeftParen) => {
                    depth += 1;
                    self.token_current();
                }
                Some(SyntaxKind::RightParen) => {
                    self.token_current();
                    depth = depth.saturating_sub(1);
                    if depth == 0 {
                        closed = true;
                        break;
                    }
                }
                Some(kind) if recovery.contains(&kind) => break,
                Some(kind) if is_interpolation_start(kind) => {
                    self.parse_interpolation(kind, &[SyntaxKind::LeftBrace, SyntaxKind::Semicolon])
                }
                Some(_) => self.token_current(),
                None => break,
            }
        }
        if node_kind.is_some() {
            self.builder.finish_node();
        }
        if !closed {
            self.error_at_current(
                ParseErrorCode::UnexpectedCharacter,
                "unterminated parenthesized prelude",
            );
        }
    }

    fn parse_interpolation(&mut self, start_kind: SyntaxKind, recovery: &[SyntaxKind]) {
        let Some(end_kind) = interpolation_end_kind(start_kind) else {
            self.token_current();
            return;
        };
        let closed = self.find_before_recovery(end_kind, recovery);
        self.builder.start_node(if closed {
            SyntaxKind::Interpolation
        } else {
            SyntaxKind::BogusInterpolation
        });
        if self.current_kind() == Some(start_kind) {
            self.token_current();
        }
        while !self.at_end() {
            match self.current_kind() {
                Some(kind) if kind == end_kind => {
                    self.token_current();
                    break;
                }
                Some(kind) if !closed && recovery.contains(&kind) => break,
                Some(_) => self.token_current(),
                None => break,
            }
        }
        if !closed {
            self.error_at_current(
                ParseErrorCode::UnexpectedCharacter,
                "unterminated interpolation",
            );
        }
        self.builder.finish_node();
    }

    fn parse_group_at_rule_block(&mut self) {
        self.token_current();
        self.builder.start_node(SyntaxKind::RuleList);
        self.parse_rule_list_items();
        self.builder.finish_node();
        if self.current_kind() == Some(SyntaxKind::RightBrace) {
            self.token_current();
        }
    }

    fn parse_rule_list_items(&mut self) {
        while !self.at_end() {
            self.eat_trivia();
            match self.current_kind() {
                Some(SyntaxKind::RightBrace | SyntaxKind::SassDedent) | None => break,
                Some(SyntaxKind::Semicolon | SyntaxKind::SassOptionalSemicolon) => {
                    self.token_current()
                }
                Some(SyntaxKind::AtKeyword) if self.current_is_css_module_value_rule() => {
                    self.parse_css_module_value_rule()
                }
                Some(SyntaxKind::AtKeyword) if self.current_dialect_at_rule_spec().is_some() => {
                    self.parse_dialect_at_rule()
                }
                Some(SyntaxKind::AtKeyword) => self.parse_at_rule(),
                Some(_) => self.parse_rule(),
            }
        }
    }

    fn parse_declaration_block(&mut self) {
        self.token_current();
        self.builder
            .start_node(if self.previous_left_brace_has_match() {
                SyntaxKind::DeclarationList
            } else {
                SyntaxKind::BogusDeclarationList
            });
        self.parse_declaration_list();
        self.builder.finish_node();
        if self.current_kind() == Some(SyntaxKind::RightBrace) {
            self.token_current();
        } else {
            self.missing_token_bogus_trivia(
                ParseErrorCode::UnexpectedCharacter,
                "unterminated declaration block",
            );
        }
    }

    fn parse_sass_indented_at_rule_block(&mut self, block_kind: AtRuleBlockKind) {
        self.builder.start_node(SyntaxKind::SassIndentedBlock);
        if self.current_kind() == Some(SyntaxKind::SassIndent) {
            self.token_current();
        }
        match block_kind {
            AtRuleBlockKind::GroupRuleList => {
                self.builder.start_node(SyntaxKind::RuleList);
                self.parse_rule_list_items();
                self.builder.finish_node();
            }
            AtRuleBlockKind::DeclarationList | AtRuleBlockKind::Keyframes => {
                self.builder.start_node(SyntaxKind::DeclarationList);
                self.parse_declaration_list();
                self.builder.finish_node();
            }
            AtRuleBlockKind::Raw => self.consume_sass_indented_raw_body(),
        }
        if self.current_kind() == Some(SyntaxKind::SassDedent) {
            self.token_current();
        } else {
            self.error_at_current(
                ParseErrorCode::UnexpectedCharacter,
                "unterminated Sass indented at-rule block",
            );
        }
        self.builder.finish_node();
    }

    fn consume_sass_indented_raw_body(&mut self) {
        let mut depth = 0usize;
        while !self.at_end() {
            match self.current_kind() {
                Some(SyntaxKind::SassIndent) => {
                    depth += 1;
                    self.token_current();
                }
                Some(SyntaxKind::SassDedent) if depth == 0 => break,
                Some(SyntaxKind::SassDedent) => {
                    depth = depth.saturating_sub(1);
                    self.token_current();
                }
                Some(_) => self.token_current(),
                None => break,
            }
        }
    }

    fn parse_keyframes_block(&mut self) {
        self.token_current();
        while !self.at_end() {
            self.eat_trivia();
            match self.current_kind() {
                Some(SyntaxKind::RightBrace) | None => break,
                Some(_) => self.parse_keyframe_block(),
            }
        }
        if self.current_kind() == Some(SyntaxKind::RightBrace) {
            self.token_current();
        }
    }

    fn parse_keyframe_block(&mut self) {
        let has_block = self.find_before_recovery(SyntaxKind::LeftBrace, &[SyntaxKind::RightBrace]);
        self.builder.start_node(if has_block {
            SyntaxKind::KeyframeBlock
        } else {
            SyntaxKind::BogusKeyframeBlock
        });
        if has_block && !self.keyframe_selector_list_is_valid() {
            self.error_at_current(ParseErrorCode::ExpectedValue, "invalid keyframe selector");
        }
        while !self.at_end() {
            match self.current_kind() {
                Some(SyntaxKind::LeftBrace) => {
                    self.parse_declaration_block();
                    break;
                }
                Some(SyntaxKind::RightBrace) | None => break,
                Some(_) => self.token_current(),
            }
        }
        if !has_block {
            self.error_at_current(
                ParseErrorCode::UnexpectedCharacter,
                "expected keyframe declaration block",
            );
        }
        self.builder.finish_node();
    }

    fn keyframe_selector_list_is_valid(&self) -> bool {
        let mut index = self.position;
        let mut saw_selector = false;
        let mut expect_selector = true;
        loop {
            let Some((token_index, kind)) = self.non_trivia_token_from(index) else {
                return false;
            };
            if kind == SyntaxKind::LeftBrace {
                return saw_selector && !expect_selector;
            }
            if expect_selector {
                if is_interpolation_start(kind) {
                    return true;
                }
                if !keyframe_selector_token_is_valid(self.tokens[token_index]) {
                    return false;
                }
                saw_selector = true;
                expect_selector = false;
                index = token_index + 1;
                continue;
            }
            if kind != SyntaxKind::Comma {
                return false;
            }
            expect_selector = true;
            index = token_index + 1;
        }
    }

    fn consume_balanced_block(&mut self) {
        let mut depth = 0usize;
        while !self.at_end() {
            match self.current_kind() {
                Some(SyntaxKind::LeftBrace) => {
                    depth += 1;
                    self.token_current();
                }
                Some(SyntaxKind::RightBrace) => {
                    self.token_current();
                    depth = depth.saturating_sub(1);
                    if depth == 0 {
                        break;
                    }
                }
                Some(_) => self.token_current(),
                None => break,
            }
        }
    }

    fn eat_trivia(&mut self) {
        while matches!(self.current_kind(), Some(kind) if kind.is_trivia()) {
            self.token_current();
        }
    }

    fn consume_until_recovery(&mut self, recovery: &[SyntaxKind]) {
        let should_wrap = self
            .current_kind()
            .is_some_and(|kind| !recovery.contains(&kind));
        if should_wrap {
            self.builder.start_node(SyntaxKind::BogusRecovery);
        }
        while !self.at_end() {
            match self.current_kind() {
                Some(kind) if recovery.contains(&kind) => break,
                Some(_) => self.token_current(),
                None => break,
            }
        }
        if should_wrap {
            self.builder.finish_node();
        }
    }

    fn find_before_recovery(&self, target: SyntaxKind, recovery: &[SyntaxKind]) -> bool {
        let mut index = self.position;
        while let Some(token) = self.tokens.get(index) {
            if token.kind == target {
                return true;
            }
            if recovery.contains(&token.kind) {
                return false;
            }
            index += 1;
        }
        false
    }

    fn find_rule_block_open_before_recovery(&self, recovery: &[SyntaxKind]) -> bool {
        let mut index = self.position;
        while let Some(token) = self.tokens.get(index) {
            if token.kind == SyntaxKind::LeftBrace
                || (self.dialect == StyleDialect::Sass && token.kind == SyntaxKind::SassIndent)
            {
                return true;
            }
            if recovery.contains(&token.kind) {
                return false;
            }
            index += 1;
        }
        false
    }

    fn find_text_before_recovery(&self, target: &str, recovery: &[SyntaxKind]) -> bool {
        let mut index = self.position;
        while let Some(token) = self.tokens.get(index) {
            if token.text == target {
                return true;
            }
            if recovery.contains(&token.kind) {
                return false;
            }
            index += 1;
        }
        false
    }

    fn current_function_has_closing_paren_before(&self, recovery: &[SyntaxKind]) -> bool {
        let Some(open_index) = self.position.checked_add(1) else {
            return false;
        };
        if self
            .tokens
            .get(open_index)
            .is_none_or(|token| token.kind != SyntaxKind::LeftParen)
        {
            return false;
        }

        let mut depth = 0usize;
        for token in self.tokens.iter().skip(open_index) {
            match token.kind {
                SyntaxKind::LeftParen => depth += 1,
                SyntaxKind::RightParen => {
                    depth = depth.saturating_sub(1);
                    if depth == 0 {
                        return true;
                    }
                }
                kind if depth == 1 && recovery.contains(&kind) => return false,
                _ => {}
            }
        }
        false
    }

    fn current_split_important_annotation(&self) -> bool {
        self.current_text() == Some("!")
            && self
                .non_trivia_token_from(self.position + 1)
                .is_some_and(|(index, kind)| {
                    matches!(kind, SyntaxKind::Ident | SyntaxKind::KeywordImportant)
                        && self
                            .tokens
                            .get(index)
                            .is_some_and(|token| token.text.eq_ignore_ascii_case("important"))
                })
    }

    fn current_scss_variable_flag_annotation(&self) -> bool {
        matches!(self.dialect, StyleDialect::Scss | StyleDialect::Sass)
            && self.current_text() == Some("!")
            && self
                .non_trivia_token_from(self.position + 1)
                .is_some_and(|(index, kind)| {
                    kind == SyntaxKind::Ident
                        && self.tokens.get(index).is_some_and(|token| {
                            token.text.eq_ignore_ascii_case("default")
                                || token.text.eq_ignore_ascii_case("global")
                        })
                })
    }

    fn current_bracketed_value_has_closing_bracket_before(&self, recovery: &[SyntaxKind]) -> bool {
        let mut depth = 0usize;
        for token in self.tokens.iter().skip(self.position) {
            match token.kind {
                SyntaxKind::LeftBracket => depth += 1,
                SyntaxKind::RightBracket => {
                    depth = depth.saturating_sub(1);
                    if depth == 0 {
                        return true;
                    }
                }
                kind if depth == 1 && recovery.contains(&kind) => return false,
                _ => {}
            }
        }
        false
    }

    fn current_simple_block_has_matching_close(&self, recovery: &[SyntaxKind]) -> bool {
        let Some(open_kind) = self.current_kind() else {
            return false;
        };
        if matching_simple_block_close(open_kind).is_none() {
            return false;
        }

        let mut expected_closes = Vec::new();
        for token in self.tokens.iter().skip(self.position) {
            if let Some(close_kind) = matching_simple_block_close(token.kind) {
                expected_closes.push(close_kind);
                continue;
            }

            if expected_closes.last().copied() == Some(token.kind) {
                expected_closes.pop();
                if expected_closes.is_empty() {
                    return true;
                }
                continue;
            }

            if expected_closes.len() == 1 && recovery.contains(&token.kind) {
                return false;
            }
        }
        false
    }

    fn current_dialect_at_rule_node_kind(&self, spec: AtRuleSpec) -> SyntaxKind {
        if !self.find_rule_block_open_before_recovery(&[
            SyntaxKind::Semicolon,
            SyntaxKind::SassOptionalSemicolon,
            SyntaxKind::RightBrace,
            SyntaxKind::SassDedent,
        ]) {
            return match spec.node_kind {
                SyntaxKind::ScssMixinDeclaration => SyntaxKind::BogusScssMixin,
                SyntaxKind::ScssFunctionDeclaration => SyntaxKind::BogusScssFunction,
                SyntaxKind::ScssControlIf
                | SyntaxKind::ScssControlElse
                | SyntaxKind::ScssControlEach
                | SyntaxKind::ScssControlFor
                | SyntaxKind::ScssControlWhile => SyntaxKind::BogusScssControl,
                _ => spec.node_kind,
            };
        }
        spec.node_kind
    }

    fn current_less_guard_has_condition_before(&self, recovery: &[SyntaxKind]) -> bool {
        let mut index = self.position + 1;
        while let Some(token) = self.tokens.get(index) {
            if recovery.contains(&token.kind) {
                return false;
            }
            if token.kind == SyntaxKind::LeftParen {
                return true;
            }
            index += 1;
        }
        false
    }

    fn current_scss_module_config_has_balanced_parens(&self) -> bool {
        let Some((_, SyntaxKind::LeftParen)) = self.non_trivia_token_from(self.position + 1) else {
            return false;
        };
        self.current_prelude_parentheses_are_balanced_until(&[
            SyntaxKind::Semicolon,
            SyntaxKind::SassOptionalSemicolon,
            SyntaxKind::LeftBrace,
            SyntaxKind::SassIndent,
        ])
    }

    fn current_value_has_top_level_comma_before(&self, recovery: &[SyntaxKind]) -> bool {
        let mut paren_depth = 0usize;
        let mut bracket_depth = 0usize;
        for token in self.tokens.iter().skip(self.position) {
            match token.kind {
                kind if paren_depth == 0 && bracket_depth == 0 && recovery.contains(&kind) => {
                    return false;
                }
                SyntaxKind::LeftParen => paren_depth += 1,
                SyntaxKind::RightParen => paren_depth = paren_depth.saturating_sub(1),
                SyntaxKind::LeftBracket => bracket_depth += 1,
                SyntaxKind::RightBracket => bracket_depth = bracket_depth.saturating_sub(1),
                SyntaxKind::Comma if paren_depth == 0 && bracket_depth == 0 => return true,
                _ => {}
            }
        }
        false
    }

    fn current_value_list_is_bogus(&self, recovery: &[SyntaxKind]) -> bool {
        let mut paren_depth = 0usize;
        let mut bracket_depth = 0usize;
        let mut expecting_item = true;
        for token in self.tokens.iter().skip(self.position) {
            if token.kind.is_trivia() {
                continue;
            }
            match token.kind {
                kind if paren_depth == 0 && bracket_depth == 0 && recovery.contains(&kind) => {
                    return expecting_item;
                }
                SyntaxKind::LeftParen => {
                    paren_depth += 1;
                    expecting_item = false;
                }
                SyntaxKind::RightParen => {
                    paren_depth = paren_depth.saturating_sub(1);
                    expecting_item = false;
                }
                SyntaxKind::LeftBracket => {
                    bracket_depth += 1;
                    expecting_item = false;
                }
                SyntaxKind::RightBracket => {
                    bracket_depth = bracket_depth.saturating_sub(1);
                    expecting_item = false;
                }
                SyntaxKind::Comma if paren_depth == 0 && bracket_depth == 0 => {
                    if expecting_item {
                        return true;
                    }
                    expecting_item = true;
                }
                _ => expecting_item = false,
            }
        }
        expecting_item
    }

    fn current_starts_missing_semicolon_declaration(&self, recovery: &[SyntaxKind]) -> bool {
        match self.current_kind() {
            Some(SyntaxKind::Ident | SyntaxKind::CustomPropertyName) => {}
            _ => return false,
        }

        let mut index = self.position + 1;
        while let Some(token) = self.tokens.get(index) {
            if token.kind.is_trivia() {
                index += 1;
                continue;
            }
            if recovery.contains(&token.kind) {
                return false;
            }
            return token.kind == SyntaxKind::Colon;
        }
        false
    }

    fn current_selector_item_is_bogus(&self, recovery: &[SyntaxKind]) -> bool {
        self.selector_item_is_bogus_from(self.position, recovery)
    }

    fn selector_item_is_bogus_from(&self, start: usize, recovery: &[SyntaxKind]) -> bool {
        let mut paren_depth = 0usize;
        let mut bracket_depth = 0usize;
        let mut saw_selector_token = false;

        for token in self.tokens.iter().skip(start) {
            if token.kind.is_trivia() {
                continue;
            }
            if paren_depth == 0
                && bracket_depth == 0
                && (token.kind == SyntaxKind::Comma
                    || is_selector_boundary_until(token.kind, recovery))
            {
                break;
            }

            match token.kind {
                SyntaxKind::LeftParen => paren_depth += 1,
                SyntaxKind::RightParen => paren_depth = paren_depth.saturating_sub(1),
                SyntaxKind::LeftBracket => bracket_depth += 1,
                SyntaxKind::RightBracket => bracket_depth = bracket_depth.saturating_sub(1),
                _ => {}
            }

            if !selector_item_token_is_recoverable(token.kind) {
                return true;
            }
            saw_selector_token = true;
        }

        !saw_selector_token
    }

    fn selector_list_contains_bogus_item_until(&self, recovery: &[SyntaxKind]) -> bool {
        let mut index = self.position;
        while let Some(token) = self.tokens.get(index) {
            if token.kind.is_trivia() || token.kind == SyntaxKind::Comma {
                index += 1;
                continue;
            }
            if is_selector_boundary_until(token.kind, recovery) {
                return false;
            }
            if self.selector_item_is_bogus_from(index, recovery) {
                return true;
            }

            let mut paren_depth = 0usize;
            let mut bracket_depth = 0usize;
            while let Some(token) = self.tokens.get(index) {
                if paren_depth == 0
                    && bracket_depth == 0
                    && (token.kind == SyntaxKind::Comma
                        || is_selector_boundary_until(token.kind, recovery))
                {
                    break;
                }
                match token.kind {
                    SyntaxKind::LeftParen => paren_depth += 1,
                    SyntaxKind::RightParen => paren_depth = paren_depth.saturating_sub(1),
                    SyntaxKind::LeftBracket => bracket_depth += 1,
                    SyntaxKind::RightBracket => bracket_depth = bracket_depth.saturating_sub(1),
                    _ => {}
                }
                index += 1;
            }
        }
        false
    }

    fn current_generic_at_rule_prelude_node_kind(&self) -> SyntaxKind {
        if self.current_prelude_parentheses_are_balanced_until(&[
            SyntaxKind::LeftBrace,
            SyntaxKind::Semicolon,
        ]) {
            SyntaxKind::AtRulePrelude
        } else {
            SyntaxKind::BogusAtRulePrelude
        }
    }

    fn current_prelude_parentheses_are_balanced_until(&self, recovery: &[SyntaxKind]) -> bool {
        let mut depth = 0usize;
        for token in self.tokens.iter().skip(self.position) {
            match token.kind {
                kind if depth == 0 && recovery.contains(&kind) => return true,
                SyntaxKind::LeftParen => depth += 1,
                SyntaxKind::RightParen => {
                    if depth == 0 {
                        return false;
                    }
                    depth -= 1;
                }
                _ => {}
            }
        }
        depth == 0
    }

    fn previous_left_brace_has_match(&self) -> bool {
        let Some(open_index) = self.position.checked_sub(1) else {
            return false;
        };
        let Some(open) = self.tokens.get(open_index) else {
            return false;
        };
        if open.kind != SyntaxKind::LeftBrace {
            return false;
        }

        let mut depth = 0usize;
        for token in self.tokens.iter().skip(open_index) {
            match token.kind {
                SyntaxKind::LeftBrace => depth += 1,
                SyntaxKind::RightBrace => {
                    depth = depth.saturating_sub(1);
                    if depth == 0 {
                        return true;
                    }
                }
                _ => {}
            }
        }
        false
    }

    fn current_starts_nested_rule(&self) -> bool {
        matches!(
            self.current_kind(),
            Some(
                SyntaxKind::Dot
                    | SyntaxKind::Hash
                    | SyntaxKind::Ampersand
                    | SyntaxKind::Colon
                    | SyntaxKind::DoubleColon
                    | SyntaxKind::LeftBracket
            )
        ) && self.find_rule_block_open_before_recovery(&[
            SyntaxKind::Colon,
            SyntaxKind::Semicolon,
            SyntaxKind::SassOptionalSemicolon,
            SyntaxKind::RightBrace,
            SyntaxKind::SassDedent,
        ])
    }

    fn current_starts_scss_nested_property(&self) -> bool {
        if !matches!(self.dialect, StyleDialect::Scss | StyleDialect::Sass) {
            return false;
        }
        if !matches!(
            self.current_kind(),
            Some(SyntaxKind::Ident | SyntaxKind::CustomPropertyName)
        ) {
            return false;
        }

        let mut saw_colon = false;
        for token in self.tokens.iter().skip(self.position) {
            match token.kind {
                SyntaxKind::Colon => saw_colon = true,
                SyntaxKind::LeftBrace if saw_colon => return true,
                SyntaxKind::SassIndent if saw_colon && self.dialect == StyleDialect::Sass => {
                    return true;
                }
                SyntaxKind::Semicolon
                | SyntaxKind::SassOptionalSemicolon
                | SyntaxKind::RightBrace
                | SyntaxKind::SassDedent => return false,
                _ => {}
            }
        }
        false
    }

    fn current_starts_less_mixin_declaration(&self) -> bool {
        self.dialect == StyleDialect::Less
            && self.current_starts_less_callable_signature()
            && self.find_before_recovery(
                SyntaxKind::LeftBrace,
                &[SyntaxKind::Semicolon, SyntaxKind::RightBrace],
            )
    }

    fn current_starts_less_mixin_call(&self) -> bool {
        self.dialect == StyleDialect::Less
            && self.current_starts_less_callable_signature()
            && !self.find_before_recovery(
                SyntaxKind::LeftBrace,
                &[SyntaxKind::Semicolon, SyntaxKind::RightBrace],
            )
    }

    fn current_starts_less_callable_signature(&self) -> bool {
        match self.current_kind() {
            Some(SyntaxKind::Dot) => {
                let Some((index, SyntaxKind::Ident | SyntaxKind::CustomPropertyName)) =
                    self.non_trivia_token_from(self.position + 1)
                else {
                    return false;
                };
                self.non_trivia_token_from(index + 1)
                    .is_some_and(|(_, kind)| kind == SyntaxKind::LeftParen)
            }
            Some(SyntaxKind::Hash) => self
                .non_trivia_token_from(self.position + 1)
                .is_some_and(|(_, kind)| kind == SyntaxKind::LeftParen),
            _ => false,
        }
    }

    fn current_starts_less_extend_rule(&self) -> bool {
        self.dialect == StyleDialect::Less
            && self.current_kind() == Some(SyntaxKind::Colon)
            && self
                .non_trivia_token_from(self.position + 1)
                .is_some_and(|(index, kind)| {
                    kind == SyntaxKind::Ident
                        && self
                            .tokens
                            .get(index)
                            .is_some_and(|token| token.text == "extend")
                })
    }

    fn current_starts_less_namespace_access(&self) -> bool {
        self.dialect == StyleDialect::Less
            && matches!(
                self.current_kind(),
                Some(SyntaxKind::Dot | SyntaxKind::Hash)
            )
            && self.find_before_recovery(
                SyntaxKind::GreaterThan,
                &[
                    SyntaxKind::Semicolon,
                    SyntaxKind::LeftBrace,
                    SyntaxKind::RightBrace,
                ],
            )
            && self.find_before_recovery(
                SyntaxKind::LeftParen,
                &[
                    SyntaxKind::Semicolon,
                    SyntaxKind::LeftBrace,
                    SyntaxKind::RightBrace,
                ],
            )
    }

    fn current_left_brace_has_match(&self) -> bool {
        let mut depth = 0usize;
        for token in self.tokens.iter().skip(self.position) {
            match token.kind {
                SyntaxKind::LeftBrace => depth += 1,
                SyntaxKind::RightBrace => {
                    depth = depth.saturating_sub(1);
                    if depth == 0 {
                        return true;
                    }
                }
                _ => {}
            }
        }
        false
    }

    fn token_current(&mut self) {
        if let Some(token) = self.tokens.get(self.position).copied() {
            self.builder.token(token.kind, token.text);
            self.position += 1;
        }
    }

    fn empty_bogus_node(&mut self, kind: SyntaxKind, code: ParseErrorCode, message: &'static str) {
        self.builder.start_node(kind);
        self.builder.finish_node();
        self.error_at_current(code, message);
    }

    fn missing_token_bogus_trivia(&mut self, code: ParseErrorCode, message: &'static str) {
        self.builder.start_node(SyntaxKind::BogusTrivia);
        self.builder.finish_node();
        self.error_at_current(code, message);
    }

    fn error_at_current(&mut self, code: ParseErrorCode, message: &'static str) {
        self.errors.push(ParseError {
            code,
            range: self.current_range(),
            message,
        });
    }

    fn current_kind(&self) -> Option<SyntaxKind> {
        self.tokens.get(self.position).map(|token| token.kind)
    }

    fn current_range(&self) -> TextRange {
        if let Some(token) = self.tokens.get(self.position) {
            return token.range;
        }
        let end = self
            .tokens
            .last()
            .map(|token| token.range.end())
            .unwrap_or_else(|| TextSize::from(0));
        TextRange::new(end, end)
    }

    fn current_text(&self) -> Option<&'text str> {
        self.tokens.get(self.position).map(|token| token.text)
    }

    fn current_dialect_at_rule_spec(&self) -> Option<AtRuleSpec> {
        let text = self.current_text()?;
        match self.dialect {
            StyleDialect::Scss | StyleDialect::Sass => scss_at_rule_spec(text),
            StyleDialect::Css | StyleDialect::Less => None,
        }
    }

    fn current_is_css_module_value_rule(&self) -> bool {
        self.current_text() == Some("@value")
    }

    fn next_kind(&self) -> Option<SyntaxKind> {
        self.tokens.get(self.position + 1).map(|token| token.kind)
    }

    fn next_non_trivia_kind(&self) -> Option<SyntaxKind> {
        let mut index = self.position + 1;
        while let Some(token) = self.tokens.get(index) {
            if !token.kind.is_trivia() {
                return Some(token.kind);
            }
            index += 1;
        }
        None
    }

    fn non_trivia_token_from(&self, mut index: usize) -> Option<(usize, SyntaxKind)> {
        while let Some(token) = self.tokens.get(index) {
            if !token.kind.is_trivia() {
                return Some((index, token.kind));
            }
            index += 1;
        }
        None
    }

    fn non_trivia_token_after_interpolation(
        &self,
        mut index: usize,
        start_kind: SyntaxKind,
    ) -> Option<(usize, SyntaxKind)> {
        let end_kind = interpolation_end_kind(start_kind)?;
        index += 1;
        while let Some(token) = self.tokens.get(index) {
            if token.kind == end_kind {
                return self.non_trivia_token_from(index + 1);
            }
            if is_at_rule_prelude_boundary(token.kind) {
                return None;
            }
            index += 1;
        }
        None
    }

    fn current_starts_namespace_qualified_selector(&self, kind: SyntaxKind) -> bool {
        match kind {
            SyntaxKind::Ident | SyntaxKind::Star => {
                self.next_kind() == Some(SyntaxKind::Pipe)
                    && self
                        .tokens
                        .get(self.position + 2)
                        .is_some_and(|token| namespace_selector_target_can_start(token.kind))
            }
            SyntaxKind::Pipe => self
                .tokens
                .get(self.position + 1)
                .is_some_and(|token| namespace_selector_target_can_start(token.kind)),
            _ => false,
        }
    }

    fn namespace_qualified_selector_target_kind(&self) -> Option<SyntaxKind> {
        let target_index = if self.current_kind() == Some(SyntaxKind::Pipe) {
            self.position + 1
        } else {
            self.position + 2
        };
        self.tokens.get(target_index).map(|token| token.kind)
    }

    fn at_end(&self) -> bool {
        self.position >= self.tokens.len()
    }
}

impl<'text, 'extension, E> Tokenizer<'text, 'extension, E>
where
    E: DialectExtension,
{
    fn new(text: &'text str, extension: &'extension E) -> Self {
        Self {
            text,
            extension,
            offset: 0,
            scss_interpolation_depth: 0,
            less_interpolation_depth: 0,
            sass_indent_stack: vec![0],
            tokens: Vec::new(),
            errors: Vec::new(),
        }
    }

    fn tokenize(&mut self) {
        while let Some(current) = self.current_char() {
            let start = self.offset;
            match current {
                '\u{feff}' if start == 0 => self.bump_current(),
                '\r' | '\n' if self.extension.dialect() == StyleDialect::Sass => {
                    self.consume_sass_indented_newline(start)
                }
                char if char.is_whitespace() => {
                    self.consume_while(SyntaxKind::Whitespace, |c| c.is_whitespace())
                }
                '/' if self.starts_with("/*") => self.consume_block_comment(),
                '/' if self.starts_with("//") && self.extension.dialect() != StyleDialect::Css => {
                    self.consume_line_comment()
                }
                '#' if self.starts_with("#{") && self.supports_scss_interpolation() => {
                    self.consume_scss_interpolation_start(start)
                }
                '@' if self.starts_with("@{") && self.supports_less_interpolation() => {
                    self.consume_less_interpolation_start(start)
                }
                '!' if self.starts_with_ascii_keyword("!important") => {
                    self.consume_static(SyntaxKind::Important, start, "!important".len())
                }
                '<' if self.starts_with("<!--") => {
                    self.consume_static(SyntaxKind::Cdo, start, "<!--".len())
                }
                '-' if self.starts_with("-->") => {
                    self.consume_static(SyntaxKind::Cdc, start, "-->".len())
                }
                '"' | '\'' => self.consume_string(current),
                'u' | 'U' if self.starts_unicode_range() => self.consume_unicode_range(),
                '0'..='9' => self.consume_number(),
                '$' if matches!(
                    self.extension.dialect(),
                    StyleDialect::Scss | StyleDialect::Sass
                ) =>
                {
                    self.consume_prefixed_name(SyntaxKind::ScssVariable)
                }
                '@' if self.extension.dialect() == StyleDialect::Less => {
                    self.consume_less_at_name()
                }
                '@' => self.consume_at_keyword(),
                '!' => self.consume_static(SyntaxKind::Delim, start, 1),
                '.' if self.current_starts_number() => self.consume_number(),
                '.' => self.consume_static(SyntaxKind::Dot, start, 1),
                ',' => self.consume_static(SyntaxKind::Comma, start, 1),
                ':' if self.starts_with("::") => {
                    self.consume_static(SyntaxKind::DoubleColon, start, 2)
                }
                ':' => self.consume_static(SyntaxKind::Colon, start, 1),
                ';' => self.consume_static(SyntaxKind::Semicolon, start, 1),
                '{' => self.consume_static(SyntaxKind::LeftBrace, start, 1),
                '}' if self.scss_interpolation_depth > 0 => {
                    self.consume_scss_interpolation_end(start)
                }
                '}' if self.less_interpolation_depth > 0 => {
                    self.consume_less_interpolation_end(start)
                }
                '}' => self.consume_static(SyntaxKind::RightBrace, start, 1),
                '(' => self.consume_static(SyntaxKind::LeftParen, start, 1),
                ')' => self.consume_static(SyntaxKind::RightParen, start, 1),
                '[' => self.consume_static(SyntaxKind::LeftBracket, start, 1),
                ']' => self.consume_static(SyntaxKind::RightBracket, start, 1),
                '+' if self.starts_with("+=") => {
                    self.consume_static(SyntaxKind::PlusEquals, start, 2)
                }
                '+' if self.current_starts_number() => self.consume_number(),
                '+' => self.consume_static(SyntaxKind::Plus, start, 1),
                '-' if self.starts_with("-=") => {
                    self.consume_static(SyntaxKind::MinusEquals, start, 2)
                }
                '-' if self.current_starts_number() => self.consume_number(),
                '-' if self.current_starts_ident_sequence() => self.consume_ident_like(),
                '-' => self.consume_static(SyntaxKind::Minus, start, 1),
                '*' if self.starts_with("*=") => {
                    self.consume_static(SyntaxKind::SubstringMatch, start, 2)
                }
                '*' => self.consume_static(SyntaxKind::Star, start, 1),
                '/' if self.starts_with("/=") => {
                    self.consume_static(SyntaxKind::SlashEquals, start, 2)
                }
                '/' => self.consume_static(SyntaxKind::Slash, start, 1),
                '%' if self.starts_scss_placeholder() => {
                    self.consume_prefixed_name(SyntaxKind::ScssPlaceholder)
                }
                '%' => self.consume_static(SyntaxKind::Percent, start, 1),
                '=' if self.starts_with("=>") => self.consume_static(SyntaxKind::Arrow, start, 2),
                '=' => self.consume_static(SyntaxKind::Equals, start, 1),
                '~' if self.starts_less_escaped_string() => self.consume_less_escaped_string(start),
                '~' if self.starts_with("~=") => {
                    self.consume_static(SyntaxKind::IncludesMatch, start, 2)
                }
                '~' => self.consume_static(SyntaxKind::Tilde, start, 1),
                '|' if self.starts_with("|=") => {
                    self.consume_static(SyntaxKind::DashMatch, start, 2)
                }
                '|' if self.starts_with("||") => {
                    self.consume_static(SyntaxKind::ColumnCombinator, start, 2)
                }
                '|' => self.consume_static(SyntaxKind::Pipe, start, 1),
                '^' if self.starts_with("^=") => {
                    self.consume_static(SyntaxKind::PrefixMatch, start, 2)
                }
                '^' => self.consume_static(SyntaxKind::Caret, start, 1),
                '$' if self.starts_with("$=") => {
                    self.consume_static(SyntaxKind::SuffixMatch, start, 2)
                }
                '$' if self.starts_less_property_variable() => {
                    self.consume_prefixed_name(SyntaxKind::LessPropertyVariableToken)
                }
                '&' if self.starts_with("&&") => {
                    self.consume_static(SyntaxKind::DoubleAmpersand, start, 2)
                }
                '&' => self.consume_static(SyntaxKind::Ampersand, start, 1),
                '>' => self.consume_static(SyntaxKind::GreaterThan, start, 1),
                '<' => self.consume_static(SyntaxKind::LessThan, start, 1),
                '#' if self.current_hash_starts_name() => self.consume_name_like(SyntaxKind::Hash),
                '#' => self.consume_static(SyntaxKind::Delim, start, 1),
                '\\' if self.current_starts_valid_escape() => {
                    self.consume_name_like(SyntaxKind::Ident)
                }
                char if is_name_start(char) => self.consume_ident_like(),
                char => self.consume_unexpected(char),
            }
        }
        self.consume_pending_sass_dedents();
    }

    fn consume_static(&mut self, kind: SyntaxKind, start: usize, byte_len: usize) {
        self.offset += byte_len;
        self.push(kind, start, self.offset);
    }

    fn consume_while(&mut self, kind: SyntaxKind, predicate: impl Fn(char) -> bool) {
        let start = self.offset;
        while let Some(char) = self.current_char() {
            if !predicate(char) {
                break;
            }
            self.bump_char(char);
        }
        self.push(kind, start, self.offset);
    }

    fn consume_block_comment(&mut self) {
        let start = self.offset;
        self.offset += 2;
        while self.offset < self.text.len() {
            if self.starts_with("*/") {
                self.offset += 2;
                self.push(SyntaxKind::BlockComment, start, self.offset);
                return;
            }
            match self.current_char() {
                Some(char) => self.bump_char(char),
                None => break,
            }
        }
        self.push(SyntaxKind::BlockComment, start, self.offset);
        self.error(
            ParseErrorCode::UnterminatedBlockComment,
            start,
            self.offset,
            "unterminated block comment",
        );
    }

    fn consume_line_comment(&mut self) {
        let start = self.offset;
        while let Some(char) = self.current_char() {
            if char == '\n' {
                break;
            }
            if char == '\r' {
                break;
            }
            self.bump_char(char);
        }
        self.push(SyntaxKind::LineComment, start, self.offset);
    }

    fn consume_sass_indented_newline(&mut self, start: usize) {
        self.consume_line_break();
        let indent = self.consume_sass_line_indent();
        let line_start = self.offset;
        let current_indent = self.sass_indent_stack.last().copied().unwrap_or(0);

        if indent > current_indent {
            self.push(SyntaxKind::SassIndentedNewline, start, line_start);
            self.sass_indent_stack.push(indent);
            self.push(SyntaxKind::SassIndent, line_start, line_start);
            return;
        }

        if self.previous_significant_sass_token_can_end_statement() {
            self.push(SyntaxKind::SassOptionalSemicolon, start, start);
        }
        self.push(SyntaxKind::SassIndentedNewline, start, line_start);

        while self.sass_indent_stack.len() > 1
            && self
                .sass_indent_stack
                .last()
                .is_some_and(|current| indent < *current)
        {
            self.sass_indent_stack.pop();
            self.push(SyntaxKind::SassDedent, line_start, line_start);
        }

        if self
            .sass_indent_stack
            .last()
            .is_some_and(|current| indent != *current)
        {
            self.error(
                ParseErrorCode::UnexpectedCharacter,
                line_start,
                line_start,
                "inconsistent Sass indentation",
            );
        }
    }

    fn consume_line_break(&mut self) {
        if self.starts_with("\r\n") {
            self.offset += "\r\n".len();
            return;
        }
        if let Some(char @ ('\r' | '\n')) = self.current_char() {
            self.bump_char(char);
        }
    }

    fn consume_sass_line_indent(&mut self) -> usize {
        let mut indent = 0usize;
        while let Some(char) = self.current_char() {
            match char {
                ' ' => {
                    indent += 1;
                    self.bump_char(char);
                }
                '\t' => {
                    indent += 4;
                    self.bump_char(char);
                }
                _ => break,
            }
        }
        indent
    }

    fn consume_pending_sass_dedents(&mut self) {
        if self.extension.dialect() != StyleDialect::Sass {
            return;
        }
        while self.sass_indent_stack.len() > 1 {
            self.sass_indent_stack.pop();
            self.push(SyntaxKind::SassDedent, self.offset, self.offset);
        }
    }

    fn previous_significant_sass_token_can_end_statement(&self) -> bool {
        self.tokens
            .iter()
            .rev()
            .find(|token| !token.kind.is_trivia())
            .is_some_and(|token| sass_token_can_end_statement(token.kind))
    }

    fn consume_scss_interpolation_start(&mut self, start: usize) {
        self.offset += "#{".len();
        self.scss_interpolation_depth += 1;
        self.push(SyntaxKind::ScssInterpolationStart, start, self.offset);
    }

    fn consume_scss_interpolation_end(&mut self, start: usize) {
        self.offset += '}'.len_utf8();
        self.scss_interpolation_depth = self.scss_interpolation_depth.saturating_sub(1);
        self.push(SyntaxKind::ScssInterpolationEnd, start, self.offset);
    }

    fn consume_less_interpolation_start(&mut self, start: usize) {
        self.offset += "@{".len();
        self.less_interpolation_depth += 1;
        self.push(SyntaxKind::LessInterpolationStart, start, self.offset);
    }

    fn consume_less_interpolation_end(&mut self, start: usize) {
        self.offset += '}'.len_utf8();
        self.less_interpolation_depth = self.less_interpolation_depth.saturating_sub(1);
        self.push(SyntaxKind::LessInterpolationEnd, start, self.offset);
    }

    fn consume_string(&mut self, quote: char) {
        let start = self.offset;
        self.bump_char(quote);
        while let Some(char) = self.current_char() {
            self.bump_char(char);
            if matches!(char, '\n' | '\r' | '\u{000c}') {
                self.push(SyntaxKind::BadString, start, self.offset);
                self.error(
                    ParseErrorCode::UnterminatedString,
                    start,
                    self.offset,
                    "unterminated string",
                );
                return;
            }
            if char == quote {
                self.push(SyntaxKind::String, start, self.offset);
                return;
            }
            if char == '\\'
                && let Some(escaped) = self.current_char()
            {
                self.bump_char(escaped);
            }
        }
        self.push(SyntaxKind::BadString, start, self.offset);
        self.error(
            ParseErrorCode::UnterminatedString,
            start,
            self.offset,
            "unterminated string",
        );
    }

    fn consume_less_escaped_string(&mut self, start: usize) {
        self.offset += '~'.len_utf8();
        let Some(quote @ ('"' | '\'')) = self.current_char() else {
            self.push(SyntaxKind::Tilde, start, self.offset);
            return;
        };
        self.bump_char(quote);
        while let Some(char) = self.current_char() {
            self.bump_char(char);
            if matches!(char, '\n' | '\r' | '\u{000c}') {
                self.push(SyntaxKind::BadString, start, self.offset);
                self.error(
                    ParseErrorCode::UnterminatedString,
                    start,
                    self.offset,
                    "unterminated Less escaped string",
                );
                return;
            }
            if char == quote {
                self.push(SyntaxKind::LessEscapedString, start, self.offset);
                return;
            }
            if char == '\\'
                && let Some(escaped) = self.current_char()
            {
                self.bump_char(escaped);
            }
        }
        self.push(SyntaxKind::BadString, start, self.offset);
        self.error(
            ParseErrorCode::UnterminatedString,
            start,
            self.offset,
            "unterminated Less escaped string",
        );
    }

    fn consume_number(&mut self) {
        let start = self.offset;
        if matches!(self.current_char(), Some('+' | '-')) {
            self.bump_current();
        }
        self.consume_digits();
        if self.current_char() == Some('.') && self.char_after_current_is_ascii_digit() {
            self.bump_current();
            self.consume_digits();
        }
        if self.current_starts_number_exponent() {
            self.bump_current();
            if matches!(self.current_char(), Some('+' | '-')) {
                self.bump_current();
            }
            self.consume_digits();
        }
        if self.current_char() == Some('%') {
            self.offset += 1;
            self.push(SyntaxKind::Percentage, start, self.offset);
            return;
        }
        if self.current_starts_ident_sequence() {
            self.consume_name_continue_sequence();
            self.push(SyntaxKind::Dimension, start, self.offset);
            return;
        }
        self.push(SyntaxKind::Number, start, self.offset);
    }

    fn consume_unicode_range(&mut self) {
        let start = self.offset;
        self.bump_current();
        self.offset += '+'.len_utf8();
        self.consume_unicode_range_codepoints(true);
        if self.current_char() == Some('-') && self.next_char_is_hex_digit() {
            self.bump_current();
            self.consume_unicode_range_codepoints(false);
        }
        self.push(SyntaxKind::UnicodeRange, start, self.offset);
    }

    fn consume_unicode_range_codepoints(&mut self, allow_question_mark: bool) {
        let mut consumed = 0usize;
        while consumed < 6 {
            match self.current_char() {
                Some(char) if char.is_ascii_hexdigit() => {
                    self.bump_char(char);
                    consumed += 1;
                }
                Some('?') if allow_question_mark => {
                    self.bump_current();
                    consumed += 1;
                }
                _ => break,
            }
        }
    }

    fn consume_digits(&mut self) {
        while matches!(self.current_char(), Some('0'..='9')) {
            self.offset += 1;
        }
    }

    fn consume_prefixed_name(&mut self, preferred_kind: SyntaxKind) {
        let start = self.offset;
        self.bump_current();
        while matches!(self.current_char(), Some(char) if is_name_continue(char)) {
            self.bump_current();
        }
        let text = &self.text[start..self.offset];
        let kind = self
            .extension
            .classify_variable_token(text)
            .unwrap_or(preferred_kind);
        self.push(kind, start, self.offset);
    }

    fn consume_less_at_name(&mut self) {
        let start = self.offset;
        self.bump_current();
        while matches!(self.current_char(), Some(char) if is_name_continue(char)) {
            self.bump_current();
        }
        let text = &self.text[start..self.offset];
        let kind = if is_css_at_rule_name(text) {
            SyntaxKind::AtKeyword
        } else {
            self.extension
                .classify_variable_token(text)
                .unwrap_or(SyntaxKind::LessVariable)
        };
        self.push(kind, start, self.offset);
    }

    fn consume_at_keyword(&mut self) {
        let start = self.offset;
        self.bump_current();
        while matches!(self.current_char(), Some(char) if is_name_continue(char)) {
            self.bump_current();
        }
        self.push(SyntaxKind::AtKeyword, start, self.offset);
    }

    fn consume_name_like(&mut self, kind: SyntaxKind) {
        let start = self.offset;
        self.consume_name_start();
        self.consume_name_continue_sequence();
        self.push(kind, start, self.offset);
    }

    fn consume_ident_like(&mut self) {
        let start = self.offset;
        self.consume_name_continue_sequence();
        let ident = &self.text[start..self.offset];
        if ident.eq_ignore_ascii_case("url")
            && self.current_char() == Some('(')
            && !self.url_starts_with_quoted_argument()
        {
            self.consume_url_token(start);
            return;
        }
        let kind = if is_custom_property_name_text(ident) {
            SyntaxKind::CustomPropertyName
        } else {
            SyntaxKind::Ident
        };
        self.push(kind, start, self.offset);
    }

    fn consume_name_start(&mut self) {
        if self.current_starts_valid_escape() {
            self.consume_name_escape();
        } else {
            self.bump_current();
        }
    }

    fn consume_name_continue_sequence(&mut self) {
        loop {
            if self.current_starts_valid_escape() {
                self.consume_name_escape();
            } else if matches!(self.current_char(), Some(char) if is_name_continue(char)) {
                self.bump_current();
            } else {
                break;
            }
        }
    }

    fn consume_name_escape(&mut self) {
        self.bump_current();
        let mut hex_digits = 0usize;
        while hex_digits < 6
            && matches!(self.current_char(), Some(char) if char.is_ascii_hexdigit())
        {
            self.bump_current();
            hex_digits += 1;
        }
        if hex_digits > 0 {
            if matches!(self.current_char(), Some(char) if char.is_whitespace()) {
                self.bump_current();
            }
        } else if self.current_char().is_some() {
            self.bump_current();
        }
    }

    fn consume_url_token(&mut self, start: usize) {
        self.bump_current();
        while matches!(self.current_char(), Some(char) if char.is_whitespace()) {
            self.bump_current();
        }
        while let Some(char) = self.current_char() {
            match char {
                ')' => {
                    self.bump_current();
                    self.push(SyntaxKind::Url, start, self.offset);
                    return;
                }
                char if char.is_whitespace() => {
                    self.bump_current();
                    while matches!(self.current_char(), Some(char) if char.is_whitespace()) {
                        self.bump_current();
                    }
                    if self.current_char() == Some(')') {
                        self.bump_current();
                        self.push(SyntaxKind::Url, start, self.offset);
                        return;
                    }
                    self.consume_bad_url(start);
                    return;
                }
                '"' | '\'' | '(' => {
                    self.consume_bad_url(start);
                    return;
                }
                '\\' if self.current_starts_valid_escape() => {
                    self.consume_name_escape();
                }
                '\\' => {
                    self.consume_bad_url(start);
                    return;
                }
                char if is_non_printable_code_point(char) => {
                    self.consume_bad_url(start);
                    return;
                }
                _ => self.bump_current(),
            }
        }
        self.push(SyntaxKind::BadUrl, start, self.offset);
        self.error(
            ParseErrorCode::UnexpectedCharacter,
            start,
            self.offset,
            "unterminated url token",
        );
    }

    fn consume_bad_url(&mut self, start: usize) {
        while let Some(char) = self.current_char() {
            if char == ')' {
                self.bump_current();
                break;
            }
            if self.current_starts_valid_escape() {
                self.consume_name_escape();
            } else {
                self.bump_current();
            }
        }
        self.push(SyntaxKind::BadUrl, start, self.offset);
        self.error(
            ParseErrorCode::UnexpectedCharacter,
            start,
            self.offset,
            "bad url token",
        );
    }

    fn url_starts_with_quoted_argument(&self) -> bool {
        let Some(mut rest) = self.text.get(self.offset + '('.len_utf8()..) else {
            return false;
        };
        rest = rest.trim_start_matches(char::is_whitespace);
        matches!(rest.chars().next(), Some('"' | '\''))
    }

    fn starts_less_property_variable(&self) -> bool {
        self.extension.dialect() == StyleDialect::Less
            && self.text[self.offset + '$'.len_utf8()..]
                .chars()
                .next()
                .is_some_and(is_name_start)
    }

    fn starts_scss_placeholder(&self) -> bool {
        matches!(
            self.extension.dialect(),
            StyleDialect::Scss | StyleDialect::Sass
        ) && self.text[self.offset + '%'.len_utf8()..]
            .chars()
            .next()
            .is_some_and(is_name_start)
    }

    fn current_hash_starts_name(&self) -> bool {
        if self.current_char() != Some('#') {
            return false;
        }
        let next_offset = self.offset + '#'.len_utf8();
        self.text[next_offset..]
            .chars()
            .next()
            .is_some_and(is_name_continue)
            || self.escape_starts_at(next_offset)
    }

    fn consume_unexpected(&mut self, char: char) {
        let start = self.offset;
        self.bump_char(char);
        self.push(SyntaxKind::Delim, start, self.offset);
        self.error(
            ParseErrorCode::UnexpectedCharacter,
            start,
            self.offset,
            "unexpected character",
        );
    }

    fn push(&mut self, kind: SyntaxKind, start: usize, end: usize) {
        self.tokens.push(Token {
            kind,
            text: &self.text[start..end],
            range: text_range(start, end),
        });
    }

    fn error(&mut self, code: ParseErrorCode, start: usize, end: usize, message: &'static str) {
        self.errors.push(ParseError {
            code,
            range: text_range(start, end),
            message,
        });
    }

    fn starts_with(&self, pattern: &str) -> bool {
        self.text[self.offset..].starts_with(pattern)
    }

    fn current_starts_valid_escape(&self) -> bool {
        self.escape_starts_at(self.offset)
    }

    fn current_starts_number(&self) -> bool {
        self.starts_number_at(self.offset)
    }

    fn current_starts_number_exponent(&self) -> bool {
        let Some('e' | 'E') = self.current_char() else {
            return false;
        };
        let exponent_offset = self.offset + 'e'.len_utf8();
        self.char_at(exponent_offset)
            .is_some_and(|char| char.is_ascii_digit())
            || (matches!(self.char_at(exponent_offset), Some('+' | '-'))
                && self.char_after_offset_is_ascii_digit(exponent_offset))
    }

    fn starts_number_at(&self, offset: usize) -> bool {
        let Some(first) = self.char_at(offset) else {
            return false;
        };
        let second_offset = offset + first.len_utf8();
        match first {
            '+' | '-' => {
                self.char_at(second_offset)
                    .is_some_and(|char| char.is_ascii_digit())
                    || (self.char_at(second_offset) == Some('.')
                        && self.char_after_offset_is_ascii_digit(second_offset))
            }
            '.' => self.char_after_offset_is_ascii_digit(offset),
            char => char.is_ascii_digit(),
        }
    }

    fn current_starts_ident_sequence(&self) -> bool {
        self.starts_ident_sequence_at(self.offset)
    }

    fn starts_ident_sequence_at(&self, offset: usize) -> bool {
        let Some(first) = self.char_at(offset) else {
            return false;
        };
        let second_offset = offset + first.len_utf8();
        match first {
            '-' => {
                self.char_at(second_offset)
                    .is_some_and(|char| char == '-' || is_name_start(char))
                    || self.escape_starts_at(second_offset)
            }
            '\\' => self.escape_starts_at(offset),
            char => is_name_start(char),
        }
    }

    fn escape_starts_at(&self, offset: usize) -> bool {
        if !self
            .text
            .get(offset..)
            .is_some_and(|remaining| remaining.starts_with('\\'))
        {
            return false;
        }
        self.text[offset + '\\'.len_utf8()..]
            .chars()
            .next()
            .is_some_and(|char| !matches!(char, '\n' | '\r' | '\u{000c}'))
    }

    fn char_at(&self, offset: usize) -> Option<char> {
        self.text.get(offset..)?.chars().next()
    }

    fn char_after_current_is_ascii_digit(&self) -> bool {
        self.char_after_offset_is_ascii_digit(self.offset)
    }

    fn char_after_offset_is_ascii_digit(&self, offset: usize) -> bool {
        let Some(char) = self.char_at(offset) else {
            return false;
        };
        self.char_at(offset + char.len_utf8())
            .is_some_and(|char| char.is_ascii_digit())
    }

    fn starts_with_ascii_keyword(&self, keyword: &str) -> bool {
        let remaining = &self.text[self.offset..];
        let Some(prefix) = remaining.get(..keyword.len()) else {
            return false;
        };
        if !prefix.eq_ignore_ascii_case(keyword) {
            return false;
        }
        remaining[keyword.len()..]
            .chars()
            .next()
            .is_none_or(|char| !is_name_continue(char))
    }

    fn supports_scss_interpolation(&self) -> bool {
        matches!(
            self.extension.dialect(),
            StyleDialect::Scss | StyleDialect::Sass
        )
    }

    fn supports_less_interpolation(&self) -> bool {
        self.extension.dialect() == StyleDialect::Less
    }

    fn starts_less_escaped_string(&self) -> bool {
        self.extension.dialect() == StyleDialect::Less
            && (self.starts_with("~\"") || self.starts_with("~'"))
    }

    fn starts_unicode_range(&self) -> bool {
        let mut chars = self.text[self.offset..].chars();
        matches!(chars.next(), Some('u' | 'U'))
            && chars.next() == Some('+')
            && chars
                .next()
                .is_some_and(|char| char.is_ascii_hexdigit() || char == '?')
    }

    fn current_char(&self) -> Option<char> {
        self.text[self.offset..].chars().next()
    }

    fn next_char_is_hex_digit(&self) -> bool {
        let offset = self.offset + '-'.len_utf8();
        self.text
            .get(offset..)
            .and_then(|tail| tail.chars().next())
            .is_some_and(|char| char.is_ascii_hexdigit())
    }

    fn bump_current(&mut self) {
        if let Some(char) = self.current_char() {
            self.bump_char(char);
        }
    }

    fn bump_char(&mut self, char: char) {
        self.offset += char.len_utf8();
    }
}

fn public_token_text(text: &str) -> String {
    text.chars()
        .map(css_syntax_preprocessed_char)
        .collect::<String>()
}

fn css_syntax_preprocessed_char(char: char) -> char {
    if char == '\0' { '\u{fffd}' } else { char }
}

fn is_name_start(char: char) -> bool {
    let char = css_syntax_preprocessed_char(char);
    char == '_' || char == '-' || char.is_alphabetic() || !char.is_ascii()
}

fn is_name_continue(char: char) -> bool {
    is_name_start(char) || char.is_ascii_digit()
}

fn is_non_printable_code_point(char: char) -> bool {
    let char = css_syntax_preprocessed_char(char);
    matches!(char, '\u{0000}'..='\u{0008}' | '\u{000b}' | '\u{000e}'..='\u{001f}' | '\u{007f}')
}

fn is_custom_property_name_text(text: &str) -> bool {
    let Some(rest) = text.strip_prefix("--") else {
        return false;
    };
    let Some(first) = rest.chars().next() else {
        return false;
    };
    first == '-' || is_name_start(first) || starts_valid_escape_text(rest)
}

fn starts_valid_escape_text(text: &str) -> bool {
    text.starts_with('\\')
        && text['\\'.len_utf8()..]
            .chars()
            .next()
            .is_some_and(|char| !matches!(char, '\n' | '\r' | '\u{000c}'))
}

fn is_css_at_rule_name(text: &str) -> bool {
    matches_ignore_ascii_case(
        text,
        &[
            "@charset",
            "@container",
            "@font-face",
            "@font-feature-values",
            "@font-palette-values",
            "@import",
            "@keyframes",
            "@layer",
            "@media",
            "@namespace",
            "@page",
            "@property",
            "@scope",
            "@starting-style",
            "@supports",
            "@counter-style",
            "@custom-media",
            "@color-profile",
            "@nest",
            "@position-try",
            "@view-transition",
            "@stylistic",
            "@styleset",
            "@character-variant",
            "@swash",
            "@ornaments",
            "@annotation",
            "@historical-forms",
            "@when",
            "@else",
        ],
    )
}

fn is_interpolation_start(kind: SyntaxKind) -> bool {
    matches!(
        kind,
        SyntaxKind::ScssInterpolationStart | SyntaxKind::LessInterpolationStart
    )
}

fn is_component_value_atom_start(kind: SyntaxKind) -> bool {
    matches!(
        kind,
        SyntaxKind::Ident
            | SyntaxKind::CustomPropertyName
            | SyntaxKind::Number
            | SyntaxKind::Percentage
            | SyntaxKind::Dimension
            | SyntaxKind::String
            | SyntaxKind::LessEscapedString
            | SyntaxKind::UnicodeRange
            | SyntaxKind::Hash
            | SyntaxKind::Url
            | SyntaxKind::BadUrl
            | SyntaxKind::BadString
            | SyntaxKind::Important
            | SyntaxKind::ScssVariable
            | SyntaxKind::LessVariable
            | SyntaxKind::LessPropertyVariableToken
            | SyntaxKind::ScssInterpolationStart
            | SyntaxKind::LessInterpolationStart
    )
}

fn interpolation_end_kind(start_kind: SyntaxKind) -> Option<SyntaxKind> {
    match start_kind {
        SyntaxKind::ScssInterpolationStart => Some(SyntaxKind::ScssInterpolationEnd),
        SyntaxKind::LessInterpolationStart => Some(SyntaxKind::LessInterpolationEnd),
        _ => None,
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct SelectorBranch {
    name: String,
    range: TextRange,
    bare_suffix_base: bool,
}

fn collect_selector_facts_from_tokens(tokens: &[Token<'_>]) -> Vec<ParsedSelectorFact> {
    let mut selectors = Vec::new();
    let mut seen = BTreeSet::new();
    collect_selector_facts_in_range(
        tokens,
        0,
        tokens.len(),
        &[],
        None,
        &mut seen,
        &mut selectors,
    );
    selectors
}

fn collect_selector_facts_in_range(
    tokens: &[Token<'_>],
    start: usize,
    end: usize,
    parent_branches: &[SelectorBranch],
    css_module_scope: Option<&'static str>,
    seen: &mut BTreeSet<(ParsedSelectorFactKind, String, u32, u32)>,
    selectors: &mut Vec<ParsedSelectorFact>,
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
                        collect_selector_facts_in_range(
                            tokens,
                            open + 1,
                            close,
                            &[],
                            css_module_scope,
                            seen,
                            selectors,
                        );
                    } else {
                        let branches =
                            resolve_selector_header(tokens, index + 1, open, parent_branches);
                        push_class_selector_facts_from_header(
                            selectors,
                            seen,
                            tokens,
                            index + 1,
                            open,
                        );
                        for branch in &branches {
                            push_selector_fact(
                                selectors,
                                seen,
                                ParsedSelectorFactKind::Class,
                                branch.name.clone(),
                                branch.range,
                            );
                        }
                        collect_selector_facts_in_range(
                            tokens,
                            open + 1,
                            close,
                            &branches,
                            css_module_scope,
                            seen,
                            selectors,
                        );
                    }
                } else if style_wrapper_at_rule(tokens[index].text) {
                    collect_selector_facts_in_range(
                        tokens,
                        open + 1,
                        close,
                        parent_branches,
                        css_module_scope,
                        seen,
                        selectors,
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
            collect_selector_facts_in_range(
                tokens,
                open + 1,
                close,
                &[],
                effective_scope,
                seen,
                selectors,
            );
        } else {
            let branches = resolve_selector_header(tokens, index, open, parent_branches);
            push_class_selector_facts_from_header(selectors, seen, tokens, index, open);
            for branch in &branches {
                push_selector_fact(
                    selectors,
                    seen,
                    ParsedSelectorFactKind::Class,
                    branch.name.clone(),
                    branch.range,
                );
            }
            for id in collect_id_selector_facts_from_header(tokens, index, open)
                .into_iter()
                .chain(collect_local_function_id_selector_facts_from_header(
                    tokens, index, open,
                ))
            {
                push_selector_fact(selectors, seen, ParsedSelectorFactKind::Id, id.0, id.1);
            }
            for placeholder in collect_placeholder_selector_facts_from_header(tokens, index, open) {
                push_selector_fact(
                    selectors,
                    seen,
                    ParsedSelectorFactKind::Placeholder,
                    placeholder.0,
                    placeholder.1,
                );
            }

            collect_selector_facts_in_range(
                tokens,
                open + 1,
                close,
                &branches,
                effective_scope,
                seen,
                selectors,
            );
        }
        index = close + 1;
    }
}

fn push_class_selector_facts_from_header(
    selectors: &mut Vec<ParsedSelectorFact>,
    seen: &mut BTreeSet<(ParsedSelectorFactKind, String, u32, u32)>,
    tokens: &[Token<'_>],
    start: usize,
    end: usize,
) {
    for (name, range) in collect_class_selector_names_from_header(tokens, start, end) {
        push_selector_fact(selectors, seen, ParsedSelectorFactKind::Class, name, range);
    }
}

fn push_selector_fact(
    selectors: &mut Vec<ParsedSelectorFact>,
    seen: &mut BTreeSet<(ParsedSelectorFactKind, String, u32, u32)>,
    kind: ParsedSelectorFactKind,
    name: String,
    range: TextRange,
) {
    if seen.insert((
        kind,
        name.clone(),
        u32::from(range.start()),
        u32::from(range.end()),
    )) {
        selectors.push(ParsedSelectorFact { kind, name, range });
    }
}

fn resolve_selector_header(
    tokens: &[Token<'_>],
    start: usize,
    end: usize,
    parent_branches: &[SelectorBranch],
) -> Vec<SelectorBranch> {
    split_selector_groups(tokens, start, end)
        .into_iter()
        .flat_map(|(group_start, group_end)| {
            resolve_selector_group(tokens, group_start, group_end, parent_branches)
        })
        .collect()
}

fn resolve_selector_group(
    tokens: &[Token<'_>],
    start: usize,
    end: usize,
    parent_branches: &[SelectorBranch],
) -> Vec<SelectorBranch> {
    if let Some(mut local_names) = collect_local_function_selector_names(tokens, start, end) {
        local_names.extend(collect_class_selector_names_from_header(tokens, start, end));
        let bare_suffix_base = parent_branches.is_empty() && local_names.len() == 1;
        return local_names
            .into_iter()
            .map(|(name, range)| SelectorBranch {
                name,
                range,
                bare_suffix_base,
            })
            .collect();
    }

    let (tail_start, tail_end) = selector_group_tail_range(tokens, start, end);
    let tail_start = skip_trivia_tokens(tokens, tail_start, tail_end);

    if let Some((suffix, range)) = ampersand_suffix_selector(tokens, tail_start, tail_end) {
        let bases: Vec<&SelectorBranch> = if parent_branches.is_empty() {
            Vec::new()
        } else {
            parent_branches.iter().collect()
        };
        return bases
            .into_iter()
            .map(|parent| SelectorBranch {
                name: format!("{}{}", parent.name, suffix),
                range,
                bare_suffix_base: parent.bare_suffix_base,
            })
            .collect();
    }

    let class_names = collect_class_selector_names_from_header(tokens, tail_start, tail_end);
    if class_names.is_empty() {
        return Vec::new();
    }

    let bare_suffix_base = parent_branches.is_empty() && class_names.len() == 1;
    class_names
        .into_iter()
        .map(|(name, range)| SelectorBranch {
            name,
            range,
            bare_suffix_base,
        })
        .collect()
}

fn split_selector_groups(tokens: &[Token<'_>], start: usize, end: usize) -> Vec<(usize, usize)> {
    let mut groups = Vec::new();
    let mut group_start = start;
    let mut paren_depth = 0usize;
    let mut bracket_depth = 0usize;
    let mut index = start;
    while index < end {
        match tokens[index].kind {
            SyntaxKind::LeftParen => paren_depth += 1,
            SyntaxKind::RightParen => paren_depth = paren_depth.saturating_sub(1),
            SyntaxKind::LeftBracket => bracket_depth += 1,
            SyntaxKind::RightBracket => bracket_depth = bracket_depth.saturating_sub(1),
            SyntaxKind::Comma if paren_depth == 0 && bracket_depth == 0 => {
                groups.push((group_start, index));
                group_start = index + 1;
            }
            _ => {}
        }
        index += 1;
    }
    groups.push((group_start, end));
    groups
}

fn selector_group_tail_range(tokens: &[Token<'_>], start: usize, end: usize) -> (usize, usize) {
    let mut paren_depth = 0usize;
    let mut bracket_depth = 0usize;
    let mut tail_start = start;
    let mut index = start;
    while index < end {
        match tokens[index].kind {
            SyntaxKind::LeftParen => paren_depth += 1,
            SyntaxKind::RightParen => paren_depth = paren_depth.saturating_sub(1),
            SyntaxKind::LeftBracket => bracket_depth += 1,
            SyntaxKind::RightBracket => bracket_depth = bracket_depth.saturating_sub(1),
            kind if paren_depth == 0 && bracket_depth == 0 && is_selector_combinator_kind(kind) => {
                tail_start = index + 1;
            }
            SyntaxKind::Whitespace if paren_depth == 0 && bracket_depth == 0 => {
                let previous = previous_non_trivia_token(tokens, start, index);
                let next = next_non_trivia_token_until(tokens, index + 1, end);
                if previous.is_some_and(|token| selector_component_can_end(token.kind))
                    && next.is_some_and(|token| selector_component_can_start(token.kind))
                {
                    tail_start = index + 1;
                }
            }
            _ => {}
        }
        index += 1;
    }
    (tail_start, end)
}

fn ampersand_suffix_selector(
    tokens: &[Token<'_>],
    start: usize,
    end: usize,
) -> Option<(String, TextRange)> {
    let ampersand_index = skip_trivia_tokens(tokens, start, end);
    if tokens.get(ampersand_index)?.kind != SyntaxKind::Ampersand {
        return None;
    }
    let suffix = next_non_trivia_token_until(tokens, ampersand_index + 1, end)?;
    if matches!(
        suffix.kind,
        SyntaxKind::Ident | SyntaxKind::CustomPropertyName
    ) && (suffix.text.starts_with("__") || suffix.text.starts_with("--"))
    {
        return Some((suffix.text.to_string(), suffix.range));
    }
    None
}

fn collect_class_selector_names_from_header(
    tokens: &[Token<'_>],
    start: usize,
    end: usize,
) -> Vec<(String, TextRange)> {
    let mut names = Vec::new();
    let mut index = start;
    let mut paren_depth = 0usize;
    let mut bracket_depth = 0usize;
    while index < end {
        match tokens[index].kind {
            SyntaxKind::LeftParen => paren_depth += 1,
            SyntaxKind::RightParen => paren_depth = paren_depth.saturating_sub(1),
            SyntaxKind::LeftBracket => bracket_depth += 1,
            SyntaxKind::RightBracket => bracket_depth = bracket_depth.saturating_sub(1),
            _ => {}
        }
        if paren_depth == 0
            && bracket_depth == 0
            && tokens[index].kind == SyntaxKind::Dot
            && let Some(name) = next_non_trivia_token_until(tokens, index + 1, end)
            && matches!(
                name.kind,
                SyntaxKind::Ident | SyntaxKind::CustomPropertyName
            )
        {
            names.push((name.text.to_string(), name.range));
        }
        index += 1;
    }
    names
}

fn collect_local_function_selector_names(
    tokens: &[Token<'_>],
    start: usize,
    end: usize,
) -> Option<Vec<(String, TextRange)>> {
    let colon_index = skip_trivia_tokens(tokens, start, end);
    if tokens.get(colon_index)?.kind != SyntaxKind::Colon {
        return None;
    }
    let ident = next_non_trivia_token_until(tokens, colon_index + 1, end)?;
    if ident.kind != SyntaxKind::Ident || ident.text != "local" {
        return None;
    }
    let open_index = skip_trivia_tokens(tokens, colon_index + 2, end);
    if tokens.get(open_index)?.kind != SyntaxKind::LeftParen {
        return None;
    }
    Some(collect_class_selector_names_from_header(
        tokens,
        open_index + 1,
        end.saturating_sub(1),
    ))
}

fn collect_local_function_id_selector_facts_from_header(
    tokens: &[Token<'_>],
    start: usize,
    end: usize,
) -> Vec<(String, TextRange)> {
    let mut ids = Vec::new();
    let mut index = start;
    while index < end {
        if tokens[index].kind == SyntaxKind::Colon
            && let Some(scope) = next_non_trivia_token_until(tokens, index + 1, end)
            && scope.kind == SyntaxKind::Ident
            && scope.text == "local"
            && let Some(open) = next_non_trivia_token_after_range(tokens, scope.range, end)
            && open.kind == SyntaxKind::LeftParen
            && let Some(close) = matching_right_paren_from_range(tokens, open.range, end)
        {
            ids.extend(collect_id_selector_facts_from_header(
                tokens,
                token_index_by_range(tokens, open.range).map_or(index + 1, |value| value + 1),
                close,
            ));
            index = close.saturating_add(1);
            continue;
        }
        index += 1;
    }
    ids
}

fn css_module_block_scope_marker_in_header(
    tokens: &[Token<'_>],
    start: usize,
    end: usize,
) -> Option<&'static str> {
    if next_non_trivia_token_until(tokens, start, end)
        .is_some_and(|token| token.kind == SyntaxKind::AtKeyword)
    {
        return None;
    }

    css_module_scope_marker_after_colon(tokens, start, end)
        .filter(|_| !css_module_scope_marker_is_function(tokens, start, end))
}

fn css_module_header_is_global_only(tokens: &[Token<'_>], start: usize, end: usize) -> bool {
    if next_non_trivia_token_until(tokens, start, end)
        .is_some_and(|token| token.kind == SyntaxKind::AtKeyword)
    {
        return false;
    }
    css_module_header_contains_scope(tokens, start, end, "global")
        && collect_class_selector_names_from_header(tokens, start, end).is_empty()
        && collect_local_function_selector_names(tokens, start, end)
            .map(|names| names.is_empty())
            .unwrap_or(true)
}

fn css_module_header_contains_scope(
    tokens: &[Token<'_>],
    start: usize,
    end: usize,
    expected_scope: &str,
) -> bool {
    let mut index = start;
    while index < end {
        if tokens[index].kind == SyntaxKind::Colon
            && let Some(scope) = next_non_trivia_token_until(tokens, index + 1, end)
            && scope.kind == SyntaxKind::Ident
            && scope.text == expected_scope
        {
            return true;
        }
        index += 1;
    }
    false
}

fn css_module_scope_marker_after_colon(
    tokens: &[Token<'_>],
    start: usize,
    end: usize,
) -> Option<&'static str> {
    let colon = skip_trivia_tokens(tokens, start, end);
    if tokens.get(colon)?.kind != SyntaxKind::Colon {
        return None;
    }
    let scope = next_non_trivia_token_until(tokens, colon + 1, end)?;
    if scope.kind != SyntaxKind::Ident {
        return None;
    }
    match scope.text {
        "global" => Some("global"),
        "local" => Some("local"),
        _ => None,
    }
}

fn css_module_scope_marker_is_function(tokens: &[Token<'_>], start: usize, end: usize) -> bool {
    let colon = skip_trivia_tokens(tokens, start, end);
    let mut index = colon + 1;
    let Some(scope) = next_non_trivia_token_until(tokens, index, end) else {
        return false;
    };
    while index < end {
        if tokens[index].range == scope.range {
            break;
        }
        index += 1;
    }
    let Some(next) = next_non_trivia_token_until(tokens, index + 1, end) else {
        return false;
    };
    scope.kind == SyntaxKind::Ident && next.kind == SyntaxKind::LeftParen
}

fn collect_id_selector_facts_from_header(
    tokens: &[Token<'_>],
    start: usize,
    end: usize,
) -> Vec<(String, TextRange)> {
    let mut names = Vec::new();
    let mut index = start;
    let mut paren_depth = 0usize;
    let mut bracket_depth = 0usize;
    while index < end {
        match tokens[index].kind {
            SyntaxKind::LeftParen => paren_depth += 1,
            SyntaxKind::RightParen => paren_depth = paren_depth.saturating_sub(1),
            SyntaxKind::LeftBracket => bracket_depth += 1,
            SyntaxKind::RightBracket => bracket_depth = bracket_depth.saturating_sub(1),
            _ => {}
        }
        let token = tokens[index];
        if paren_depth == 0 && bracket_depth == 0 && token.kind == SyntaxKind::Hash {
            names.push((token.text.trim_start_matches('#').to_string(), token.range));
        }
        index += 1;
    }
    names
}

fn collect_placeholder_selector_facts_from_header(
    tokens: &[Token<'_>],
    start: usize,
    end: usize,
) -> Vec<(String, TextRange)> {
    let mut names = Vec::new();
    let mut index = start;
    let mut paren_depth = 0usize;
    let mut bracket_depth = 0usize;
    while index < end {
        match tokens[index].kind {
            SyntaxKind::LeftParen => paren_depth += 1,
            SyntaxKind::RightParen => paren_depth = paren_depth.saturating_sub(1),
            SyntaxKind::LeftBracket => bracket_depth += 1,
            SyntaxKind::RightBracket => bracket_depth = bracket_depth.saturating_sub(1),
            _ => {}
        }
        let token = tokens[index];
        if paren_depth == 0 && bracket_depth == 0 && token.kind == SyntaxKind::ScssPlaceholder {
            names.push((token.text.trim_start_matches('%').to_string(), token.range));
        }
        index += 1;
    }
    names
}

fn collect_variable_facts_from_tokens(tokens: &[Token<'_>]) -> Vec<ParsedVariableFact> {
    let mut variables = Vec::new();
    for (index, token) in tokens.iter().enumerate() {
        let kind = match token.kind {
            SyntaxKind::ScssVariable => {
                if scss_variable_token_is_declaration(tokens, index) {
                    ParsedVariableFactKind::ScssDeclaration
                } else {
                    ParsedVariableFactKind::ScssReference
                }
            }
            SyntaxKind::LessVariable => {
                if next_non_trivia_token(tokens, index + 1)
                    .is_some_and(|candidate| candidate.kind == SyntaxKind::Colon)
                {
                    ParsedVariableFactKind::LessDeclaration
                } else {
                    ParsedVariableFactKind::LessReference
                }
            }
            SyntaxKind::CustomPropertyName => {
                if previous_non_trivia_token(tokens, 0, index).is_some_and(|candidate| {
                    matches!(candidate.kind, SyntaxKind::Ampersand | SyntaxKind::Dot)
                }) {
                    continue;
                }
                if let Some(at_rule_name) = containing_at_rule_header_name(tokens, index) {
                    if at_rule_name == "@property" {
                        ParsedVariableFactKind::CustomPropertyDeclaration
                    } else {
                        continue;
                    }
                } else if next_non_trivia_token(tokens, index + 1)
                    .is_some_and(|candidate| candidate.kind == SyntaxKind::Colon)
                {
                    ParsedVariableFactKind::CustomPropertyDeclaration
                } else {
                    ParsedVariableFactKind::CustomPropertyReference
                }
            }
            _ => continue,
        };
        variables.push(ParsedVariableFact {
            kind,
            name: token.text.to_string(),
            range: token.range,
        });
    }
    variables
}

fn scss_variable_token_is_declaration(tokens: &[Token<'_>], index: usize) -> bool {
    next_non_trivia_token(tokens, index + 1).is_some_and(|candidate| {
        candidate.kind == SyntaxKind::Colon
            || (matches!(candidate.kind, SyntaxKind::Comma | SyntaxKind::RightParen)
                && containing_at_rule_header_name(tokens, index).is_some_and(|name| {
                    name.eq_ignore_ascii_case("@mixin") || name.eq_ignore_ascii_case("@function")
                }))
    })
}

fn collect_sass_symbol_facts_from_tokens(tokens: &[Token<'_>]) -> Vec<ParsedSassSymbolFact> {
    let declared_functions = collect_sass_callable_declaration_names(tokens, "@function");
    let mut symbols = Vec::new();

    for (index, token) in tokens.iter().enumerate() {
        match token.kind {
            SyntaxKind::ScssVariable => {
                let kind = if scss_variable_token_is_declaration(tokens, index) {
                    ParsedSassSymbolFactKind::VariableDeclaration
                } else {
                    ParsedSassSymbolFactKind::VariableReference
                };
                symbols.push(ParsedSassSymbolFact {
                    kind,
                    symbol_kind: "variable",
                    name: token.text.trim_start_matches('$').to_string(),
                    role: match kind {
                        ParsedSassSymbolFactKind::VariableDeclaration => "declaration",
                        _ => "reference",
                    },
                    range: token.range,
                });
            }
            SyntaxKind::AtKeyword if token.text.eq_ignore_ascii_case("@mixin") => {
                if let Some(name) = sass_callable_name_after_at_rule(tokens, index) {
                    symbols.push(ParsedSassSymbolFact {
                        kind: ParsedSassSymbolFactKind::MixinDeclaration,
                        symbol_kind: "mixin",
                        name: name.text.to_string(),
                        role: "declaration",
                        range: name.range,
                    });
                }
            }
            SyntaxKind::AtKeyword if token.text.eq_ignore_ascii_case("@include") => {
                if let Some(name) = sass_callable_name_after_at_rule(tokens, index) {
                    symbols.push(ParsedSassSymbolFact {
                        kind: ParsedSassSymbolFactKind::MixinInclude,
                        symbol_kind: "mixin",
                        name: name.text.to_string(),
                        role: "include",
                        range: name.range,
                    });
                }
            }
            SyntaxKind::AtKeyword if token.text.eq_ignore_ascii_case("@function") => {
                if let Some(name) = sass_callable_name_after_at_rule(tokens, index) {
                    symbols.push(ParsedSassSymbolFact {
                        kind: ParsedSassSymbolFactKind::FunctionDeclaration,
                        symbol_kind: "function",
                        name: name.text.to_string(),
                        role: "declaration",
                        range: name.range,
                    });
                }
            }
            SyntaxKind::Ident
                if declared_functions.contains(token.text)
                    && next_non_trivia_token(tokens, index + 1)
                        .is_some_and(|candidate| candidate.kind == SyntaxKind::LeftParen)
                    && previous_non_trivia_token(tokens, 0, index).map_or(true, |candidate| {
                        !matches!(candidate.kind, SyntaxKind::AtKeyword | SyntaxKind::Dot)
                    }) =>
            {
                symbols.push(ParsedSassSymbolFact {
                    kind: ParsedSassSymbolFactKind::FunctionCall,
                    symbol_kind: "function",
                    name: token.text.to_string(),
                    role: "call",
                    range: token.range,
                });
            }
            _ => {}
        }
    }

    symbols
}

fn collect_sass_callable_declaration_names(
    tokens: &[Token<'_>],
    at_keyword: &str,
) -> BTreeSet<String> {
    tokens
        .iter()
        .enumerate()
        .filter_map(|(index, token)| {
            (token.kind == SyntaxKind::AtKeyword && token.text.eq_ignore_ascii_case(at_keyword))
                .then(|| sass_callable_name_after_at_rule(tokens, index))
                .flatten()
                .map(|name| name.text.to_string())
        })
        .collect()
}

fn sass_callable_name_after_at_rule<'text>(
    tokens: &[Token<'text>],
    at_rule_index: usize,
) -> Option<Token<'text>> {
    let statement_end = css_module_value_statement_end(tokens, at_rule_index + 1);
    let name_index = next_non_trivia_token_index_until(tokens, at_rule_index + 1, statement_end)?;
    let name = tokens[name_index];
    if name.kind != SyntaxKind::Ident {
        return None;
    }
    if next_non_trivia_token_index_until(tokens, name_index + 1, statement_end)
        .is_some_and(|next| tokens[next].kind == SyntaxKind::Dot)
    {
        return None;
    }
    Some(name)
}

fn collect_css_module_value_facts_from_tokens(
    tokens: &[Token<'_>],
) -> Vec<ParsedCssModuleValueFact> {
    let mut values = Vec::new();
    let mut seen = BTreeSet::new();
    for (index, token) in tokens.iter().enumerate() {
        if token.kind != SyntaxKind::AtKeyword || !token.text.eq_ignore_ascii_case("@value") {
            continue;
        }

        let start = skip_trivia_tokens(tokens, index + 1, tokens.len());
        let end = css_module_value_statement_end(tokens, start);
        let colon_index = top_level_token_kind_index(tokens, start, end, SyntaxKind::Colon);
        let from_index = top_level_token_text_index(tokens, start, end, "from");

        if let Some(from_index) = from_index
            && match colon_index {
                Some(colon_index) => from_index < colon_index,
                None => true,
            }
        {
            collect_css_module_value_import_facts(
                tokens,
                start,
                from_index,
                end,
                &mut values,
                &mut seen,
            );
            continue;
        }

        if let Some(colon_index) = colon_index {
            collect_css_module_value_definition_facts(
                tokens,
                start,
                colon_index,
                &mut values,
                &mut seen,
            );
            collect_css_module_value_reference_facts(
                tokens,
                colon_index + 1,
                end,
                &mut values,
                &mut seen,
            );
        } else {
            collect_css_module_value_definition_facts(tokens, start, end, &mut values, &mut seen);
        }
    }
    values
}

fn css_module_value_statement_end(tokens: &[Token<'_>], start: usize) -> usize {
    let mut index = start;
    let mut paren_depth = 0usize;
    let mut bracket_depth = 0usize;
    while index < tokens.len() {
        match tokens[index].kind {
            SyntaxKind::LeftParen => paren_depth += 1,
            SyntaxKind::RightParen => paren_depth = paren_depth.saturating_sub(1),
            SyntaxKind::LeftBracket => bracket_depth += 1,
            SyntaxKind::RightBracket => bracket_depth = bracket_depth.saturating_sub(1),
            SyntaxKind::Semicolon
            | SyntaxKind::SassOptionalSemicolon
            | SyntaxKind::LeftBrace
            | SyntaxKind::RightBrace
            | SyntaxKind::SassIndent
            | SyntaxKind::SassDedent
                if paren_depth == 0 && bracket_depth == 0 =>
            {
                return index;
            }
            _ => {}
        }
        index += 1;
    }
    index
}

fn collect_css_module_value_import_facts(
    tokens: &[Token<'_>],
    start: usize,
    from_index: usize,
    end: usize,
    values: &mut Vec<ParsedCssModuleValueFact>,
    seen: &mut BTreeSet<(ParsedCssModuleValueFactKind, String, u32, u32)>,
) {
    collect_css_module_value_import_names(tokens, start, from_index, values, seen);
    if let Some(source_index) = next_non_trivia_token_index_until(tokens, from_index + 1, end)
        && matches!(
            tokens[source_index].kind,
            SyntaxKind::String | SyntaxKind::Url
        )
    {
        push_css_module_value_fact(
            values,
            seen,
            ParsedCssModuleValueFactKind::ImportSource,
            css_module_value_source_name(tokens[source_index]),
            tokens[source_index].range,
        );
    }
}

fn collect_css_module_value_import_edge_facts_from_tokens(
    tokens: &[Token<'_>],
) -> Vec<ParsedCssModuleValueImportEdgeFact> {
    let mut edges = Vec::new();
    for (index, token) in tokens.iter().enumerate() {
        if token.kind != SyntaxKind::AtKeyword || !token.text.eq_ignore_ascii_case("@value") {
            continue;
        }

        let start = skip_trivia_tokens(tokens, index + 1, tokens.len());
        let end = css_module_value_statement_end(tokens, start);
        let colon_index = top_level_token_kind_index(tokens, start, end, SyntaxKind::Colon);
        let from_index = top_level_token_text_index(tokens, start, end, "from");
        let Some(from_index) = from_index else {
            continue;
        };
        if colon_index.is_some_and(|colon_index| from_index > colon_index) {
            continue;
        }
        let Some(import_source) = css_module_value_import_edge_source(tokens, from_index + 1, end)
        else {
            continue;
        };

        collect_css_module_value_import_edges(tokens, start, from_index, import_source, &mut edges);
    }
    edges
}

fn collect_css_module_value_definition_edge_facts_from_tokens(
    tokens: &[Token<'_>],
) -> Vec<ParsedCssModuleValueDefinitionEdgeFact> {
    let mut edges = Vec::new();
    for (index, token) in tokens.iter().enumerate() {
        if token.kind != SyntaxKind::AtKeyword || !token.text.eq_ignore_ascii_case("@value") {
            continue;
        }

        let start = skip_trivia_tokens(tokens, index + 1, tokens.len());
        let end = css_module_value_statement_end(tokens, start);
        let colon_index = top_level_token_kind_index(tokens, start, end, SyntaxKind::Colon);
        let from_index = top_level_token_text_index(tokens, start, end, "from");
        let Some(colon_index) = colon_index else {
            continue;
        };
        if from_index.is_some_and(|from_index| from_index < colon_index) {
            continue;
        }

        let definition_names = collect_css_module_value_definition_edge_names(
            tokens,
            start,
            colon_index,
            |tokens, index| css_module_value_name_token_can_define(tokens[index]),
        );
        let reference_names = collect_css_module_value_definition_edge_names(
            tokens,
            colon_index + 1,
            end,
            css_module_value_reference_token_can_be_name,
        );
        if reference_names.is_empty() {
            continue;
        }
        let range_end = end
            .checked_sub(1)
            .and_then(|end| tokens.get(end))
            .map(|token| token.range.end())
            .unwrap_or_else(|| tokens[index].range.end());

        for definition_name in definition_names {
            edges.push(ParsedCssModuleValueDefinitionEdgeFact {
                definition_name,
                reference_names: reference_names.clone(),
                range: TextRange::new(tokens[index].range.start(), range_end),
            });
        }
    }
    edges
}

fn collect_css_module_value_definition_edge_names(
    tokens: &[Token<'_>],
    start: usize,
    end: usize,
    predicate: impl Fn(&[Token<'_>], usize) -> bool,
) -> Vec<String> {
    let mut names = Vec::new();
    let mut index = start;
    while index < end {
        if predicate(tokens, index) && !names.iter().any(|name| name == tokens[index].text) {
            names.push(tokens[index].text.to_string());
        }
        index += 1;
    }
    names
}

fn css_module_value_import_edge_source(
    tokens: &[Token<'_>],
    start: usize,
    end: usize,
) -> Option<String> {
    let source_index = next_non_trivia_token_index_until(tokens, start, end)?;
    let token = tokens[source_index];
    matches!(token.kind, SyntaxKind::String | SyntaxKind::Url)
        .then(|| css_module_value_source_name(token))
}

fn collect_css_module_value_import_edges(
    tokens: &[Token<'_>],
    start: usize,
    end: usize,
    import_source: String,
    edges: &mut Vec<ParsedCssModuleValueImportEdgeFact>,
) {
    let mut index = start;
    while index < end {
        let token = tokens[index];
        if !css_module_value_name_token_can_define(token) {
            index += 1;
            continue;
        }
        if previous_non_trivia_token_index(tokens, index, start)
            .is_some_and(|previous| tokens[previous].text == "as")
        {
            index += 1;
            continue;
        }
        let remote_name = token.text.to_string();
        let mut local_name = remote_name.clone();
        if let Some(as_index) = next_non_trivia_token_index_until(tokens, index + 1, end)
            && tokens[as_index].text == "as"
            && let Some(local_index) = next_non_trivia_token_index_until(tokens, as_index + 1, end)
            && css_module_value_name_token_can_define(tokens[local_index])
        {
            local_name = tokens[local_index].text.to_string();
            index = local_index + 1;
        } else {
            index += 1;
        }
        edges.push(ParsedCssModuleValueImportEdgeFact {
            remote_name,
            local_name,
            import_source: import_source.clone(),
            range: token.range,
        });
    }
}

fn collect_css_module_value_import_names(
    tokens: &[Token<'_>],
    start: usize,
    end: usize,
    values: &mut Vec<ParsedCssModuleValueFact>,
    seen: &mut BTreeSet<(ParsedCssModuleValueFactKind, String, u32, u32)>,
) {
    let mut index = start;
    while index < end {
        let token = tokens[index];
        if css_module_value_name_token_can_define(token) {
            let previous = previous_non_trivia_token_index(tokens, index, start);
            let next = next_non_trivia_token_index_until(tokens, index + 1, end);
            let kind = if previous.is_some_and(|previous| tokens[previous].text == "as") {
                Some(ParsedCssModuleValueFactKind::Definition)
            } else if next.is_some_and(|next| tokens[next].text == "as") {
                Some(ParsedCssModuleValueFactKind::Reference)
            } else {
                Some(ParsedCssModuleValueFactKind::Definition)
            };
            if let Some(kind) = kind {
                push_css_module_value_fact(values, seen, kind, token.text.to_string(), token.range);
            }
        }
        index += 1;
    }
}

fn collect_css_module_value_definition_facts(
    tokens: &[Token<'_>],
    start: usize,
    end: usize,
    values: &mut Vec<ParsedCssModuleValueFact>,
    seen: &mut BTreeSet<(ParsedCssModuleValueFactKind, String, u32, u32)>,
) {
    let mut index = start;
    while index < end {
        let token = tokens[index];
        if css_module_value_name_token_can_define(token) {
            push_css_module_value_fact(
                values,
                seen,
                ParsedCssModuleValueFactKind::Definition,
                token.text.to_string(),
                token.range,
            );
        }
        index += 1;
    }
}

fn collect_css_module_value_reference_facts(
    tokens: &[Token<'_>],
    start: usize,
    end: usize,
    values: &mut Vec<ParsedCssModuleValueFact>,
    seen: &mut BTreeSet<(ParsedCssModuleValueFactKind, String, u32, u32)>,
) {
    let mut index = start;
    let mut paren_depth = 0usize;
    let mut bracket_depth = 0usize;
    while index < end {
        match tokens[index].kind {
            SyntaxKind::LeftParen => paren_depth += 1,
            SyntaxKind::RightParen => paren_depth = paren_depth.saturating_sub(1),
            SyntaxKind::LeftBracket => bracket_depth += 1,
            SyntaxKind::RightBracket => bracket_depth = bracket_depth.saturating_sub(1),
            _ => {}
        }
        if paren_depth == 0
            && bracket_depth == 0
            && css_module_value_reference_token_can_be_name(tokens, index)
        {
            push_css_module_value_fact(
                values,
                seen,
                ParsedCssModuleValueFactKind::Reference,
                tokens[index].text.to_string(),
                tokens[index].range,
            );
        }
        index += 1;
    }
}

fn push_css_module_value_fact(
    values: &mut Vec<ParsedCssModuleValueFact>,
    seen: &mut BTreeSet<(ParsedCssModuleValueFactKind, String, u32, u32)>,
    kind: ParsedCssModuleValueFactKind,
    name: String,
    range: TextRange,
) {
    if seen.insert((
        kind,
        name.clone(),
        u32::from(range.start()),
        u32::from(range.end()),
    )) {
        values.push(ParsedCssModuleValueFact { kind, name, range });
    }
}

fn top_level_token_kind_index(
    tokens: &[Token<'_>],
    start: usize,
    end: usize,
    expected: SyntaxKind,
) -> Option<usize> {
    let mut index = start;
    let mut paren_depth = 0usize;
    let mut bracket_depth = 0usize;
    while index < end {
        match tokens[index].kind {
            SyntaxKind::LeftParen => paren_depth += 1,
            SyntaxKind::RightParen => paren_depth = paren_depth.saturating_sub(1),
            SyntaxKind::LeftBracket => bracket_depth += 1,
            SyntaxKind::RightBracket => bracket_depth = bracket_depth.saturating_sub(1),
            kind if kind == expected && paren_depth == 0 && bracket_depth == 0 => {
                return Some(index);
            }
            _ => {}
        }
        index += 1;
    }
    None
}

fn top_level_token_text_index(
    tokens: &[Token<'_>],
    start: usize,
    end: usize,
    expected: &str,
) -> Option<usize> {
    let mut index = start;
    let mut paren_depth = 0usize;
    let mut bracket_depth = 0usize;
    while index < end {
        match tokens[index].kind {
            SyntaxKind::LeftParen => paren_depth += 1,
            SyntaxKind::RightParen => paren_depth = paren_depth.saturating_sub(1),
            SyntaxKind::LeftBracket => bracket_depth += 1,
            SyntaxKind::RightBracket => bracket_depth = bracket_depth.saturating_sub(1),
            SyntaxKind::Ident
                if paren_depth == 0
                    && bracket_depth == 0
                    && tokens[index].text.eq_ignore_ascii_case(expected) =>
            {
                return Some(index);
            }
            _ => {}
        }
        index += 1;
    }
    None
}

fn previous_non_trivia_token_index(
    tokens: &[Token<'_>],
    mut index: usize,
    start: usize,
) -> Option<usize> {
    while index > start {
        index -= 1;
        if !tokens[index].kind.is_trivia() {
            return Some(index);
        }
    }
    None
}

fn css_module_value_name_token_can_define(token: Token<'_>) -> bool {
    matches!(
        token.kind,
        SyntaxKind::Ident | SyntaxKind::CustomPropertyName
    ) && !matches!(token.text, "as" | "from")
}

fn css_module_value_reference_token_can_be_name(tokens: &[Token<'_>], index: usize) -> bool {
    let token = tokens[index];
    if !matches!(
        token.kind,
        SyntaxKind::Ident | SyntaxKind::CustomPropertyName
    ) {
        return false;
    }
    if let Some(next_index) = next_non_trivia_token_index_until(tokens, index + 1, tokens.len())
        && tokens[next_index].kind == SyntaxKind::LeftParen
    {
        return false;
    }
    !css_module_value_literal_ident_is_not_reference(token.text)
}

fn css_module_value_literal_ident_is_not_reference(name: &str) -> bool {
    matches!(
        name.to_ascii_lowercase().as_str(),
        "initial"
            | "inherit"
            | "unset"
            | "revert"
            | "revert-layer"
            | "none"
            | "auto"
            | "normal"
            | "transparent"
            | "currentcolor"
            | "black"
            | "white"
            | "red"
            | "green"
            | "blue"
            | "yellow"
            | "magenta"
            | "cyan"
            | "solid"
            | "dashed"
            | "block"
            | "inline"
            | "flex"
            | "grid"
    )
}

fn css_module_value_source_name(token: Token<'_>) -> String {
    token
        .text
        .trim_matches(|character| character == '"' || character == '\'')
        .to_string()
}

fn collect_css_module_composes_facts_from_tokens(
    tokens: &[Token<'_>],
) -> Vec<ParsedCssModuleComposesFact> {
    let mut composes = Vec::new();
    let mut seen = BTreeSet::new();
    for (index, token) in tokens.iter().enumerate() {
        if token.kind != SyntaxKind::Ident || !token.text.eq_ignore_ascii_case("composes") {
            continue;
        }
        let Some(colon_index) = next_non_trivia_token_index_until(tokens, index + 1, tokens.len())
        else {
            continue;
        };
        if tokens[colon_index].kind != SyntaxKind::Colon {
            continue;
        }

        let start = colon_index + 1;
        let end = css_module_value_statement_end(tokens, start);
        let from_index = top_level_token_text_index(tokens, start, end, "from");
        let target_end = from_index.unwrap_or(end);
        collect_css_module_composes_targets(tokens, start, target_end, &mut composes, &mut seen);
        if let Some(from_index) = from_index {
            collect_css_module_composes_import_source(
                tokens,
                from_index + 1,
                end,
                &mut composes,
                &mut seen,
            );
        }
    }
    composes
}

fn collect_css_module_composes_edge_facts_from_tokens(
    tokens: &[Token<'_>],
) -> Vec<ParsedCssModuleComposesEdgeFact> {
    let mut edges = Vec::new();
    collect_css_module_composes_edge_facts_in_range(tokens, 0, tokens.len(), &[], None, &mut edges);
    edges
}

fn collect_css_module_composes_edge_facts_in_range(
    tokens: &[Token<'_>],
    start: usize,
    end: usize,
    parent_branches: &[SelectorBranch],
    css_module_scope: Option<&'static str>,
    edges: &mut Vec<ParsedCssModuleComposesEdgeFact>,
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
                        collect_css_module_composes_edge_facts_in_range(
                            tokens,
                            open + 1,
                            close,
                            &[],
                            css_module_scope,
                            edges,
                        );
                    } else {
                        let branches =
                            resolve_selector_header(tokens, index + 1, open, parent_branches);
                        collect_immediate_css_module_composes_edge_facts(
                            tokens,
                            open + 1,
                            close,
                            &branches,
                            edges,
                        );
                        collect_css_module_composes_edge_facts_in_range(
                            tokens,
                            open + 1,
                            close,
                            &branches,
                            css_module_scope,
                            edges,
                        );
                    }
                } else if style_wrapper_at_rule(tokens[index].text) {
                    collect_css_module_composes_edge_facts_in_range(
                        tokens,
                        open + 1,
                        close,
                        parent_branches,
                        css_module_scope,
                        edges,
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
            collect_css_module_composes_edge_facts_in_range(
                tokens,
                open + 1,
                close,
                &[],
                effective_scope,
                edges,
            );
        } else {
            let branches = resolve_selector_header(tokens, index, open, parent_branches);
            collect_immediate_css_module_composes_edge_facts(
                tokens,
                open + 1,
                close,
                &branches,
                edges,
            );
            collect_css_module_composes_edge_facts_in_range(
                tokens,
                open + 1,
                close,
                &branches,
                effective_scope,
                edges,
            );
        }
        index = close + 1;
    }
}

fn collect_immediate_css_module_composes_edge_facts(
    tokens: &[Token<'_>],
    start: usize,
    end: usize,
    owner_branches: &[SelectorBranch],
    edges: &mut Vec<ParsedCssModuleComposesEdgeFact>,
) {
    let owner_selector_names = sorted_selector_branch_names(owner_branches);
    let mut index = start;
    let mut block_depth = 0usize;
    while index < end {
        match tokens[index].kind {
            SyntaxKind::LeftBrace | SyntaxKind::SassIndent => {
                block_depth += 1;
                index += 1;
                continue;
            }
            SyntaxKind::RightBrace | SyntaxKind::SassDedent => {
                block_depth = block_depth.saturating_sub(1);
                index += 1;
                continue;
            }
            _ => {}
        }
        if block_depth > 0
            || tokens[index].kind != SyntaxKind::Ident
            || !tokens[index].text.eq_ignore_ascii_case("composes")
        {
            index += 1;
            continue;
        }
        let Some(colon_index) = next_non_trivia_token_index_until(tokens, index + 1, end) else {
            index += 1;
            continue;
        };
        if tokens[colon_index].kind != SyntaxKind::Colon {
            index += 1;
            continue;
        }

        let value_start = colon_index + 1;
        let value_end = css_module_value_statement_end(tokens, value_start).min(end);
        let from_index = top_level_token_text_index(tokens, value_start, value_end, "from");
        let target_end = from_index.unwrap_or(value_end);
        let target_names =
            collect_css_module_composes_target_names(tokens, value_start, target_end);
        if target_names.is_empty() {
            index = value_end;
            continue;
        }

        let (kind, import_source) = from_index
            .and_then(|from_index| {
                css_module_composes_import_edge_source(tokens, from_index + 1, value_end)
            })
            .map(|source| {
                if source == "global" {
                    (ParsedCssModuleComposesEdgeKind::Global, Some(source))
                } else {
                    (ParsedCssModuleComposesEdgeKind::External, Some(source))
                }
            })
            .unwrap_or((ParsedCssModuleComposesEdgeKind::Local, None));
        let range_end = value_end
            .checked_sub(1)
            .and_then(|end| tokens.get(end))
            .map(|token| token.range.end())
            .unwrap_or_else(|| tokens[index].range.end());

        edges.push(ParsedCssModuleComposesEdgeFact {
            kind,
            owner_selector_names: owner_selector_names.clone(),
            target_names,
            import_source,
            range: TextRange::new(tokens[index].range.start(), range_end),
        });
        index = value_end;
    }
}

fn sorted_selector_branch_names(branches: &[SelectorBranch]) -> Vec<String> {
    branches
        .iter()
        .map(|branch| branch.name.clone())
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect()
}

fn collect_css_module_composes_target_names(
    tokens: &[Token<'_>],
    start: usize,
    end: usize,
) -> Vec<String> {
    let mut names = Vec::new();
    let mut index = start;
    while index < end {
        if matches!(
            tokens[index].kind,
            SyntaxKind::Ident | SyntaxKind::CustomPropertyName
        ) && !tokens[index].text.eq_ignore_ascii_case("from")
            && !names.iter().any(|name| name == tokens[index].text)
        {
            names.push(tokens[index].text.to_string());
        }
        index += 1;
    }
    names
}

fn css_module_composes_import_edge_source(
    tokens: &[Token<'_>],
    start: usize,
    end: usize,
) -> Option<String> {
    let source_index = next_non_trivia_token_index_until(tokens, start, end)?;
    let token = tokens[source_index];
    matches!(
        token.kind,
        SyntaxKind::String | SyntaxKind::Url | SyntaxKind::Ident
    )
    .then(|| css_module_value_source_name(token))
}

fn collect_css_module_composes_targets(
    tokens: &[Token<'_>],
    start: usize,
    end: usize,
    composes: &mut Vec<ParsedCssModuleComposesFact>,
    seen: &mut BTreeSet<(ParsedCssModuleComposesFactKind, String, u32, u32)>,
) {
    let mut index = start;
    while index < end {
        if matches!(
            tokens[index].kind,
            SyntaxKind::Ident | SyntaxKind::CustomPropertyName
        ) && !tokens[index].text.eq_ignore_ascii_case("from")
        {
            push_css_module_composes_fact(
                composes,
                seen,
                ParsedCssModuleComposesFactKind::Target,
                tokens[index].text.to_string(),
                tokens[index].range,
            );
        }
        index += 1;
    }
}

fn collect_css_module_composes_import_source(
    tokens: &[Token<'_>],
    start: usize,
    end: usize,
    composes: &mut Vec<ParsedCssModuleComposesFact>,
    seen: &mut BTreeSet<(ParsedCssModuleComposesFactKind, String, u32, u32)>,
) {
    if let Some(source_index) = next_non_trivia_token_index_until(tokens, start, end) {
        let token = tokens[source_index];
        if matches!(
            token.kind,
            SyntaxKind::String | SyntaxKind::Url | SyntaxKind::Ident
        ) {
            push_css_module_composes_fact(
                composes,
                seen,
                ParsedCssModuleComposesFactKind::ImportSource,
                css_module_value_source_name(token),
                token.range,
            );
        }
    }
}

fn push_css_module_composes_fact(
    composes: &mut Vec<ParsedCssModuleComposesFact>,
    seen: &mut BTreeSet<(ParsedCssModuleComposesFactKind, String, u32, u32)>,
    kind: ParsedCssModuleComposesFactKind,
    name: String,
    range: TextRange,
) {
    if seen.insert((
        kind,
        name.clone(),
        u32::from(range.start()),
        u32::from(range.end()),
    )) {
        composes.push(ParsedCssModuleComposesFact { kind, name, range });
    }
}

fn collect_icss_facts_from_tokens(tokens: &[Token<'_>]) -> Vec<ParsedIcssFact> {
    let mut icss = Vec::new();
    let mut seen = BTreeSet::new();
    for (index, token) in tokens.iter().enumerate() {
        if token.kind != SyntaxKind::Colon {
            continue;
        }
        let Some(name_index) = next_non_trivia_token_index_until(tokens, index + 1, tokens.len())
        else {
            continue;
        };
        let name = tokens[name_index].text;
        if !matches!(tokens[name_index].kind, SyntaxKind::Ident) {
            continue;
        }
        if name.eq_ignore_ascii_case("export") {
            if let Some((open, close)) =
                find_block_after_header(tokens, name_index + 1, tokens.len())
            {
                collect_icss_export_names(tokens, open + 1, close, &mut icss, &mut seen);
            }
            continue;
        }
        if name.eq_ignore_ascii_case("import") {
            collect_icss_import_source(tokens, name_index + 1, &mut icss, &mut seen);
            if let Some((open, close)) =
                find_block_after_header(tokens, name_index + 1, tokens.len())
            {
                collect_icss_import_names(tokens, open + 1, close, &mut icss, &mut seen);
            }
        }
    }
    icss
}

fn collect_icss_import_edge_facts_from_tokens(
    tokens: &[Token<'_>],
) -> Vec<ParsedIcssImportEdgeFact> {
    let mut edges = Vec::new();
    for (index, token) in tokens.iter().enumerate() {
        if token.kind != SyntaxKind::Colon {
            continue;
        }
        let Some(name_index) = next_non_trivia_token_index_until(tokens, index + 1, tokens.len())
        else {
            continue;
        };
        if tokens[name_index].kind != SyntaxKind::Ident
            || !tokens[name_index].text.eq_ignore_ascii_case("import")
        {
            continue;
        }
        let Some(import_source) = icss_import_edge_source(tokens, name_index + 1) else {
            continue;
        };
        if let Some((open, close)) = find_block_after_header(tokens, name_index + 1, tokens.len()) {
            collect_icss_import_edges(tokens, open + 1, close, import_source, &mut edges);
        }
    }
    edges
}

fn collect_icss_export_edge_facts_from_tokens(
    tokens: &[Token<'_>],
) -> Vec<ParsedIcssExportEdgeFact> {
    let mut edges = Vec::new();
    for (index, token) in tokens.iter().enumerate() {
        if token.kind != SyntaxKind::Colon {
            continue;
        }
        let Some(name_index) = next_non_trivia_token_index_until(tokens, index + 1, tokens.len())
        else {
            continue;
        };
        if tokens[name_index].kind != SyntaxKind::Ident
            || !tokens[name_index].text.eq_ignore_ascii_case("export")
        {
            continue;
        }
        if let Some((open, close)) = find_block_after_header(tokens, name_index + 1, tokens.len()) {
            collect_icss_export_edges(tokens, open + 1, close, &mut edges);
        }
    }
    edges
}

fn collect_icss_export_edges(
    tokens: &[Token<'_>],
    start: usize,
    end: usize,
    edges: &mut Vec<ParsedIcssExportEdgeFact>,
) {
    let mut index = start;
    while index < end {
        let token = tokens[index];
        if matches!(
            token.kind,
            SyntaxKind::Ident | SyntaxKind::CustomPropertyName
        ) && let Some(colon_index) = next_non_trivia_token_index_until(tokens, index + 1, end)
            && tokens[colon_index].kind == SyntaxKind::Colon
        {
            let value_end = css_module_value_statement_end(tokens, colon_index + 1).min(end);
            let reference_names = collect_css_module_value_definition_edge_names(
                tokens,
                colon_index + 1,
                value_end,
                css_module_value_reference_token_can_be_name,
            );
            if !reference_names.is_empty() {
                let range_end = value_end
                    .checked_sub(1)
                    .and_then(|end| tokens.get(end))
                    .map(|token| token.range.end())
                    .unwrap_or_else(|| token.range.end());
                edges.push(ParsedIcssExportEdgeFact {
                    export_name: token.text.to_string(),
                    reference_names,
                    range: TextRange::new(token.range.start(), range_end),
                });
            }
            index = value_end;
            continue;
        }
        index += 1;
    }
}

fn icss_import_edge_source(tokens: &[Token<'_>], start: usize) -> Option<String> {
    let open_index = next_non_trivia_token_index_until(tokens, start, tokens.len())?;
    if tokens[open_index].kind != SyntaxKind::LeftParen {
        return None;
    }
    let source_index = next_non_trivia_token_index_until(tokens, open_index + 1, tokens.len())?;
    let token = tokens[source_index];
    matches!(
        token.kind,
        SyntaxKind::String | SyntaxKind::Url | SyntaxKind::Ident
    )
    .then(|| css_module_value_source_name(token))
}

fn collect_icss_import_edges(
    tokens: &[Token<'_>],
    start: usize,
    end: usize,
    import_source: String,
    edges: &mut Vec<ParsedIcssImportEdgeFact>,
) {
    let mut index = start;
    while index < end {
        let token = tokens[index];
        if matches!(
            token.kind,
            SyntaxKind::Ident | SyntaxKind::CustomPropertyName
        ) && let Some(colon_index) = next_non_trivia_token_index_until(tokens, index + 1, end)
            && tokens[colon_index].kind == SyntaxKind::Colon
            && let Some(remote_index) =
                next_non_trivia_token_index_until(tokens, colon_index + 1, end)
            && matches!(
                tokens[remote_index].kind,
                SyntaxKind::Ident | SyntaxKind::CustomPropertyName
            )
        {
            edges.push(ParsedIcssImportEdgeFact {
                local_name: token.text.to_string(),
                remote_name: tokens[remote_index].text.to_string(),
                import_source: import_source.clone(),
                range: token.range,
            });
            index = css_module_value_statement_end(tokens, colon_index + 1);
            continue;
        }
        index += 1;
    }
}

fn collect_icss_export_names(
    tokens: &[Token<'_>],
    start: usize,
    end: usize,
    icss: &mut Vec<ParsedIcssFact>,
    seen: &mut BTreeSet<(ParsedIcssFactKind, String, u32, u32)>,
) {
    let mut index = start;
    while index < end {
        let token = tokens[index];
        if matches!(
            token.kind,
            SyntaxKind::Ident | SyntaxKind::CustomPropertyName
        ) && let Some(colon_index) = next_non_trivia_token_index_until(tokens, index + 1, end)
            && tokens[colon_index].kind == SyntaxKind::Colon
        {
            push_icss_fact(
                icss,
                seen,
                ParsedIcssFactKind::ExportName,
                token.text.to_string(),
                token.range,
            );
            index = css_module_value_statement_end(tokens, colon_index + 1);
            continue;
        }
        index += 1;
    }
}

fn collect_icss_import_source(
    tokens: &[Token<'_>],
    start: usize,
    icss: &mut Vec<ParsedIcssFact>,
    seen: &mut BTreeSet<(ParsedIcssFactKind, String, u32, u32)>,
) {
    let Some(open_index) = next_non_trivia_token_index_until(tokens, start, tokens.len()) else {
        return;
    };
    if tokens[open_index].kind != SyntaxKind::LeftParen {
        return;
    }
    let Some(source_index) =
        next_non_trivia_token_index_until(tokens, open_index + 1, tokens.len())
    else {
        return;
    };
    let token = tokens[source_index];
    if matches!(
        token.kind,
        SyntaxKind::String | SyntaxKind::Url | SyntaxKind::Ident
    ) {
        push_icss_fact(
            icss,
            seen,
            ParsedIcssFactKind::ImportSource,
            css_module_value_source_name(token),
            token.range,
        );
    }
}

fn collect_icss_import_names(
    tokens: &[Token<'_>],
    start: usize,
    end: usize,
    icss: &mut Vec<ParsedIcssFact>,
    seen: &mut BTreeSet<(ParsedIcssFactKind, String, u32, u32)>,
) {
    let mut index = start;
    while index < end {
        let token = tokens[index];
        if matches!(
            token.kind,
            SyntaxKind::Ident | SyntaxKind::CustomPropertyName
        ) && let Some(colon_index) = next_non_trivia_token_index_until(tokens, index + 1, end)
            && tokens[colon_index].kind == SyntaxKind::Colon
        {
            push_icss_fact(
                icss,
                seen,
                ParsedIcssFactKind::ImportLocalName,
                token.text.to_string(),
                token.range,
            );
            if let Some(remote_index) =
                next_non_trivia_token_index_until(tokens, colon_index + 1, end)
                && matches!(
                    tokens[remote_index].kind,
                    SyntaxKind::Ident | SyntaxKind::CustomPropertyName
                )
            {
                push_icss_fact(
                    icss,
                    seen,
                    ParsedIcssFactKind::ImportRemoteName,
                    tokens[remote_index].text.to_string(),
                    tokens[remote_index].range,
                );
            }
            index = css_module_value_statement_end(tokens, colon_index + 1);
            continue;
        }
        index += 1;
    }
}

fn push_icss_fact(
    icss: &mut Vec<ParsedIcssFact>,
    seen: &mut BTreeSet<(ParsedIcssFactKind, String, u32, u32)>,
    kind: ParsedIcssFactKind,
    name: String,
    range: TextRange,
) {
    if seen.insert((
        kind,
        name.clone(),
        u32::from(range.start()),
        u32::from(range.end()),
    )) {
        icss.push(ParsedIcssFact { kind, name, range });
    }
}

fn collect_animation_facts_from_tokens(tokens: &[Token<'_>]) -> Vec<ParsedAnimationFact> {
    let mut animations = Vec::new();
    let mut seen = BTreeSet::new();
    for (index, token) in tokens.iter().enumerate() {
        if token.kind == SyntaxKind::AtKeyword && token.text.eq_ignore_ascii_case("@keyframes") {
            if let Some(name_index) =
                next_non_trivia_token_index_until(tokens, index + 1, tokens.len())
                && let Some(name) = animation_name_from_token(tokens[name_index])
            {
                push_animation_fact(
                    &mut animations,
                    &mut seen,
                    ParsedAnimationFactKind::KeyframesDeclaration,
                    name,
                    tokens[name_index].range,
                );
            }
            continue;
        }

        if token.kind == SyntaxKind::Ident
            && token.text.eq_ignore_ascii_case("animation-name")
            && let Some(colon_index) =
                next_non_trivia_token_index_until(tokens, index + 1, tokens.len())
            && tokens[colon_index].kind == SyntaxKind::Colon
        {
            collect_animation_name_references_until(
                tokens,
                colon_index + 1,
                &mut animations,
                &mut seen,
            );
        }

        if token.kind == SyntaxKind::Ident
            && token.text.eq_ignore_ascii_case("animation")
            && let Some(colon_index) =
                next_non_trivia_token_index_until(tokens, index + 1, tokens.len())
            && tokens[colon_index].kind == SyntaxKind::Colon
        {
            collect_animation_shorthand_references_until(
                tokens,
                colon_index + 1,
                &mut animations,
                &mut seen,
            );
        }
    }
    animations
}

fn collect_animation_name_references_until(
    tokens: &[Token<'_>],
    start: usize,
    animations: &mut Vec<ParsedAnimationFact>,
    seen: &mut BTreeSet<(ParsedAnimationFactKind, String, u32, u32)>,
) {
    let mut index = start;
    let mut paren_depth = 0usize;
    let mut bracket_depth = 0usize;
    while index < tokens.len() {
        match tokens[index].kind {
            SyntaxKind::LeftParen => paren_depth += 1,
            SyntaxKind::RightParen => paren_depth = paren_depth.saturating_sub(1),
            SyntaxKind::LeftBracket => bracket_depth += 1,
            SyntaxKind::RightBracket => bracket_depth = bracket_depth.saturating_sub(1),
            SyntaxKind::Semicolon
            | SyntaxKind::SassOptionalSemicolon
            | SyntaxKind::RightBrace
            | SyntaxKind::SassDedent
                if paren_depth == 0 && bracket_depth == 0 =>
            {
                break;
            }
            _ => {}
        }

        if paren_depth == 0
            && bracket_depth == 0
            && let Some(name) = animation_name_from_token(tokens[index])
        {
            push_animation_fact(
                animations,
                seen,
                ParsedAnimationFactKind::AnimationNameReference,
                name,
                tokens[index].range,
            );
        }
        index += 1;
    }
}

fn collect_animation_shorthand_references_until(
    tokens: &[Token<'_>],
    start: usize,
    animations: &mut Vec<ParsedAnimationFact>,
    seen: &mut BTreeSet<(ParsedAnimationFactKind, String, u32, u32)>,
) {
    let mut index = start;
    let mut paren_depth = 0usize;
    let mut bracket_depth = 0usize;
    while index < tokens.len() {
        match tokens[index].kind {
            SyntaxKind::LeftParen => paren_depth += 1,
            SyntaxKind::RightParen => paren_depth = paren_depth.saturating_sub(1),
            SyntaxKind::LeftBracket => bracket_depth += 1,
            SyntaxKind::RightBracket => bracket_depth = bracket_depth.saturating_sub(1),
            SyntaxKind::Semicolon
            | SyntaxKind::SassOptionalSemicolon
            | SyntaxKind::RightBrace
            | SyntaxKind::SassDedent
                if paren_depth == 0 && bracket_depth == 0 =>
            {
                break;
            }
            _ => {}
        }

        if paren_depth == 0
            && bracket_depth == 0
            && animation_shorthand_token_can_be_name(tokens, index)
            && let Some(name) = animation_name_from_token(tokens[index])
        {
            push_animation_fact(
                animations,
                seen,
                ParsedAnimationFactKind::AnimationNameReference,
                name,
                tokens[index].range,
            );
        }
        index += 1;
    }
}

fn animation_shorthand_token_can_be_name(tokens: &[Token<'_>], index: usize) -> bool {
    let token = tokens[index];
    if token.kind == SyntaxKind::String {
        return true;
    }
    if token.kind != SyntaxKind::Ident {
        return false;
    }
    if let Some(next_index) = next_non_trivia_token_index_until(tokens, index + 1, tokens.len())
        && tokens[next_index].kind == SyntaxKind::LeftParen
    {
        return false;
    }
    !animation_shorthand_ident_is_non_name(token.text)
}

fn animation_shorthand_ident_is_non_name(name: &str) -> bool {
    matches!(
        name.to_ascii_lowercase().as_str(),
        "ease"
            | "ease-in"
            | "ease-out"
            | "ease-in-out"
            | "linear"
            | "step-start"
            | "step-end"
            | "infinite"
            | "normal"
            | "reverse"
            | "alternate"
            | "alternate-reverse"
            | "running"
            | "paused"
            | "forwards"
            | "backwards"
            | "both"
            | "replace"
            | "add"
            | "accumulate"
            | "auto"
    )
}

fn push_animation_fact(
    animations: &mut Vec<ParsedAnimationFact>,
    seen: &mut BTreeSet<(ParsedAnimationFactKind, String, u32, u32)>,
    kind: ParsedAnimationFactKind,
    name: String,
    range: TextRange,
) {
    if seen.insert((
        kind,
        name.clone(),
        u32::from(range.start()),
        u32::from(range.end()),
    )) {
        animations.push(ParsedAnimationFact { kind, name, range });
    }
}

fn animation_name_from_token(token: Token<'_>) -> Option<String> {
    if !matches!(token.kind, SyntaxKind::Ident | SyntaxKind::String) {
        return None;
    }
    let name = token
        .text
        .trim_matches(|character| character == '"' || character == '\'')
        .to_string();
    if name.is_empty() || animation_name_is_reserved(&name) {
        return None;
    }
    Some(name)
}

fn animation_name_is_reserved(name: &str) -> bool {
    matches!(
        name.to_ascii_lowercase().as_str(),
        "none" | "initial" | "inherit" | "unset" | "revert" | "revert-layer"
    )
}

fn containing_at_rule_header_name<'text>(
    tokens: &'text [Token<'text>],
    index: usize,
) -> Option<&'text str> {
    let mut current = index;
    while current > 0 {
        current -= 1;
        let token = tokens.get(current)?;
        if token.kind.is_trivia() {
            continue;
        }
        if matches!(
            token.kind,
            SyntaxKind::Semicolon
                | SyntaxKind::SassOptionalSemicolon
                | SyntaxKind::LeftBrace
                | SyntaxKind::RightBrace
                | SyntaxKind::SassIndent
                | SyntaxKind::SassDedent
        ) {
            return None;
        }
        if token.kind == SyntaxKind::AtKeyword {
            return Some(token.text);
        }
    }
    None
}

fn skip_trivia_tokens(tokens: &[Token<'_>], mut index: usize, end: usize) -> usize {
    while index < end && tokens[index].kind.is_trivia() {
        index += 1;
    }
    index
}

fn skip_statement(tokens: &[Token<'_>], mut index: usize, end: usize) -> usize {
    while index < end {
        match tokens[index].kind {
            SyntaxKind::Semicolon | SyntaxKind::SassOptionalSemicolon => return index + 1,
            SyntaxKind::RightBrace | SyntaxKind::SassDedent => return index,
            _ => index += 1,
        }
    }
    index
}

fn find_block_after_header(
    tokens: &[Token<'_>],
    start: usize,
    end: usize,
) -> Option<(usize, usize)> {
    let mut index = start;
    while index < end {
        match tokens[index].kind {
            SyntaxKind::Semicolon
            | SyntaxKind::SassOptionalSemicolon
            | SyntaxKind::RightBrace
            | SyntaxKind::SassDedent => return None,
            SyntaxKind::LeftBrace => {
                let close = matching_right_brace(tokens, index, end)?;
                return Some((index, close));
            }
            SyntaxKind::SassIndent => {
                let close = matching_sass_dedent(tokens, index, end)?;
                return Some((index, close));
            }
            _ => index += 1,
        }
    }
    None
}

fn matching_right_brace(tokens: &[Token<'_>], open: usize, end: usize) -> Option<usize> {
    let mut depth = 0usize;
    let mut index = open;
    while index < end {
        match tokens[index].kind {
            SyntaxKind::LeftBrace => depth += 1,
            SyntaxKind::RightBrace => {
                depth = depth.saturating_sub(1);
                if depth == 0 {
                    return Some(index);
                }
            }
            _ => {}
        }
        index += 1;
    }
    None
}

fn matching_sass_dedent(tokens: &[Token<'_>], open: usize, end: usize) -> Option<usize> {
    let mut depth = 0usize;
    let mut index = open;
    while index < end {
        match tokens[index].kind {
            SyntaxKind::SassIndent => depth += 1,
            SyntaxKind::SassDedent => {
                depth = depth.saturating_sub(1);
                if depth == 0 {
                    return Some(index);
                }
            }
            _ => {}
        }
        index += 1;
    }
    None
}

fn style_wrapper_at_rule(name: &str) -> bool {
    matches_ignore_ascii_case(
        name,
        &[
            "@media",
            "@supports",
            "@when",
            "@else",
            "@layer",
            "@scope",
            "@container",
            "@starting-style",
            "@if",
            "@else",
            "@for",
            "@each",
            "@while",
            "@at-root",
            "@include",
        ],
    )
}

fn is_selector_combinator_kind(kind: SyntaxKind) -> bool {
    matches!(
        kind,
        SyntaxKind::GreaterThan
            | SyntaxKind::Plus
            | SyntaxKind::Tilde
            | SyntaxKind::ColumnCombinator
            | SyntaxKind::DoublePipe
    )
}

fn selector_component_can_start(kind: SyntaxKind) -> bool {
    matches!(
        kind,
        SyntaxKind::Dot
            | SyntaxKind::Hash
            | SyntaxKind::Ident
            | SyntaxKind::Star
            | SyntaxKind::Ampersand
            | SyntaxKind::ScssPlaceholder
            | SyntaxKind::LeftBracket
            | SyntaxKind::Colon
            | SyntaxKind::DoubleColon
    )
}

fn namespace_selector_target_can_start(kind: SyntaxKind) -> bool {
    matches!(
        kind,
        SyntaxKind::Ident | SyntaxKind::CustomPropertyName | SyntaxKind::Star
    )
}

fn keyframe_selector_token_is_valid(token: Token<'_>) -> bool {
    token.kind == SyntaxKind::Percentage
        || (token.kind == SyntaxKind::Ident
            && (token.text.eq_ignore_ascii_case("from") || token.text.eq_ignore_ascii_case("to")))
}

fn selector_component_can_end(kind: SyntaxKind) -> bool {
    matches!(
        kind,
        SyntaxKind::Ident
            | SyntaxKind::CustomPropertyName
            | SyntaxKind::Hash
            | SyntaxKind::RightBracket
            | SyntaxKind::RightParen
            | SyntaxKind::Star
    )
}

fn collect_at_rule_facts_from_tokens(
    tokens: &[Token<'_>],
    dialect: StyleDialect,
) -> Vec<ParsedAtRuleFact> {
    tokens
        .iter()
        .filter(|token| token.kind == SyntaxKind::AtKeyword)
        .map(|token| {
            let css_spec = at_rule_spec(token.text);
            let node_kind = css_spec
                .or_else(|| match dialect {
                    StyleDialect::Scss | StyleDialect::Sass => scss_at_rule_spec(token.text),
                    StyleDialect::Css | StyleDialect::Less => None,
                })
                .map(|spec| spec.node_kind);
            let name = if css_spec.is_some() {
                token.text.to_ascii_lowercase()
            } else {
                token.text.to_string()
            };
            ParsedAtRuleFact {
                name,
                node_kind,
                range: token.range,
            }
        })
        .collect()
}

fn next_non_trivia_token<'text>(
    tokens: &'text [Token<'text>],
    mut index: usize,
) -> Option<Token<'text>> {
    while let Some(token) = tokens.get(index).copied() {
        if !token.kind.is_trivia() {
            return Some(token);
        }
        index += 1;
    }
    None
}

fn next_non_trivia_token_until<'text>(
    tokens: &'text [Token<'text>],
    mut index: usize,
    end: usize,
) -> Option<Token<'text>> {
    while index < end {
        let token = tokens.get(index).copied()?;
        if !token.kind.is_trivia() {
            return Some(token);
        }
        index += 1;
    }
    None
}

fn next_non_trivia_token_index_until(
    tokens: &[Token<'_>],
    mut index: usize,
    end: usize,
) -> Option<usize> {
    while index < end {
        let token = tokens.get(index)?;
        if !token.kind.is_trivia() {
            return Some(index);
        }
        index += 1;
    }
    None
}

fn next_non_trivia_token_after_range<'text>(
    tokens: &'text [Token<'text>],
    range: TextRange,
    end: usize,
) -> Option<Token<'text>> {
    let index = token_index_by_range(tokens, range)?;
    next_non_trivia_token_until(tokens, index + 1, end)
}

fn token_index_by_range(tokens: &[Token<'_>], range: TextRange) -> Option<usize> {
    tokens.iter().position(|token| token.range == range)
}

fn matching_right_paren_from_range(
    tokens: &[Token<'_>],
    open_range: TextRange,
    end: usize,
) -> Option<usize> {
    let mut depth = 0usize;
    let mut index = token_index_by_range(tokens, open_range)?;
    while index < end {
        match tokens[index].kind {
            SyntaxKind::LeftParen => depth += 1,
            SyntaxKind::RightParen => {
                depth = depth.saturating_sub(1);
                if depth == 0 {
                    return Some(index);
                }
            }
            _ => {}
        }
        index += 1;
    }
    None
}

fn previous_non_trivia_token<'text>(
    tokens: &'text [Token<'text>],
    start: usize,
    index: usize,
) -> Option<Token<'text>> {
    let mut current = index;
    while current > start {
        current -= 1;
        let token = tokens.get(current).copied()?;
        if !token.kind.is_trivia() {
            return Some(token);
        }
    }
    None
}

fn at_rule_spec(text: &str) -> Option<AtRuleSpec> {
    let lowered = text.to_ascii_lowercase();
    let (node_kind, block_kind) = match lowered.as_str() {
        "@media" => (SyntaxKind::MediaRule, AtRuleBlockKind::GroupRuleList),
        "@supports" => (SyntaxKind::SupportsRule, AtRuleBlockKind::GroupRuleList),
        "@when" => (SyntaxKind::WhenRule, AtRuleBlockKind::GroupRuleList),
        "@else" => (SyntaxKind::ElseRule, AtRuleBlockKind::GroupRuleList),
        "@container" => (SyntaxKind::ContainerRule, AtRuleBlockKind::GroupRuleList),
        "@layer" => (SyntaxKind::LayerRule, AtRuleBlockKind::GroupRuleList),
        "@scope" => (SyntaxKind::ScopeRule, AtRuleBlockKind::GroupRuleList),
        "@starting-style" => (
            SyntaxKind::StartingStyleRule,
            AtRuleBlockKind::GroupRuleList,
        ),
        "@nest" => (SyntaxKind::NestRule, AtRuleBlockKind::DeclarationList),
        "@keyframes" => (SyntaxKind::KeyframesRule, AtRuleBlockKind::Keyframes),
        "@font-face" => (SyntaxKind::FontFaceRule, AtRuleBlockKind::DeclarationList),
        "@page" => (SyntaxKind::PageRule, AtRuleBlockKind::DeclarationList),
        "@property" => (SyntaxKind::PropertyRule, AtRuleBlockKind::DeclarationList),
        "@counter-style" => (
            SyntaxKind::CounterStyleRule,
            AtRuleBlockKind::DeclarationList,
        ),
        "@font-palette-values" => (
            SyntaxKind::FontPaletteValuesRule,
            AtRuleBlockKind::DeclarationList,
        ),
        "@color-profile" => (
            SyntaxKind::ColorProfileRule,
            AtRuleBlockKind::DeclarationList,
        ),
        "@position-try" => (
            SyntaxKind::PositionTryRule,
            AtRuleBlockKind::DeclarationList,
        ),
        "@font-feature-values" => (
            SyntaxKind::FontFeatureValuesRule,
            AtRuleBlockKind::GroupRuleList,
        ),
        "@stylistic" => (
            SyntaxKind::FontFeatureValuesStylisticRule,
            AtRuleBlockKind::DeclarationList,
        ),
        "@styleset" => (
            SyntaxKind::FontFeatureValuesStylesetRule,
            AtRuleBlockKind::DeclarationList,
        ),
        "@character-variant" => (
            SyntaxKind::FontFeatureValuesCharacterVariantRule,
            AtRuleBlockKind::DeclarationList,
        ),
        "@swash" => (
            SyntaxKind::FontFeatureValuesSwashRule,
            AtRuleBlockKind::DeclarationList,
        ),
        "@ornaments" => (
            SyntaxKind::FontFeatureValuesOrnamentsRule,
            AtRuleBlockKind::DeclarationList,
        ),
        "@annotation" => (
            SyntaxKind::FontFeatureValuesAnnotationRule,
            AtRuleBlockKind::DeclarationList,
        ),
        "@historical-forms" => (
            SyntaxKind::FontFeatureValuesHistoricalFormsRule,
            AtRuleBlockKind::DeclarationList,
        ),
        "@view-transition" => (
            SyntaxKind::ViewTransitionRule,
            AtRuleBlockKind::DeclarationList,
        ),
        "@charset" => (SyntaxKind::CharsetRule, AtRuleBlockKind::Raw),
        "@import" => (SyntaxKind::ImportRule, AtRuleBlockKind::Raw),
        "@namespace" => (SyntaxKind::NamespaceRule, AtRuleBlockKind::Raw),
        "@custom-media" => (SyntaxKind::CustomMediaRule, AtRuleBlockKind::Raw),
        text if is_page_margin_at_rule(text) => {
            (SyntaxKind::PageMarginRule, AtRuleBlockKind::DeclarationList)
        }
        _ => return None,
    };
    Some(AtRuleSpec {
        node_kind,
        block_kind,
    })
}

fn is_page_margin_at_rule(text: &str) -> bool {
    matches!(
        text,
        "@top-left-corner"
            | "@top-left"
            | "@top-center"
            | "@top-right"
            | "@top-right-corner"
            | "@bottom-left-corner"
            | "@bottom-left"
            | "@bottom-center"
            | "@bottom-right"
            | "@bottom-right-corner"
            | "@left-top"
            | "@left-middle"
            | "@left-bottom"
            | "@right-top"
            | "@right-middle"
            | "@right-bottom"
    )
}

fn scss_at_rule_spec(text: &str) -> Option<AtRuleSpec> {
    let (node_kind, block_kind) = match text {
        "@use" => (SyntaxKind::ScssUseRule, AtRuleBlockKind::Raw),
        "@forward" => (SyntaxKind::ScssForwardRule, AtRuleBlockKind::Raw),
        "@mixin" => (
            SyntaxKind::ScssMixinDeclaration,
            AtRuleBlockKind::DeclarationList,
        ),
        "@include" => (
            SyntaxKind::ScssIncludeRule,
            AtRuleBlockKind::DeclarationList,
        ),
        "@function" => (
            SyntaxKind::ScssFunctionDeclaration,
            AtRuleBlockKind::DeclarationList,
        ),
        "@return" => (SyntaxKind::ScssReturnRule, AtRuleBlockKind::Raw),
        "@extend" => (SyntaxKind::ScssExtendRule, AtRuleBlockKind::Raw),
        "@if" => (SyntaxKind::ScssControlIf, AtRuleBlockKind::DeclarationList),
        "@else" => (
            SyntaxKind::ScssControlElse,
            AtRuleBlockKind::DeclarationList,
        ),
        "@each" => (
            SyntaxKind::ScssControlEach,
            AtRuleBlockKind::DeclarationList,
        ),
        "@for" => (SyntaxKind::ScssControlFor, AtRuleBlockKind::DeclarationList),
        "@while" => (
            SyntaxKind::ScssControlWhile,
            AtRuleBlockKind::DeclarationList,
        ),
        "@at-root" => (SyntaxKind::ScssAtRootRule, AtRuleBlockKind::DeclarationList),
        "@error" => (SyntaxKind::ScssErrorRule, AtRuleBlockKind::Raw),
        "@warn" => (SyntaxKind::ScssWarnRule, AtRuleBlockKind::Raw),
        "@debug" => (SyntaxKind::ScssDebugRule, AtRuleBlockKind::Raw),
        "@content" => (SyntaxKind::ScssContentRule, AtRuleBlockKind::Raw),
        _ => return None,
    };
    Some(AtRuleSpec {
        node_kind,
        block_kind,
    })
}

fn is_selector_boundary(kind: SyntaxKind) -> bool {
    matches!(
        kind,
        SyntaxKind::Comma
            | SyntaxKind::LeftBrace
            | SyntaxKind::SassIndent
            | SyntaxKind::RightBrace
            | SyntaxKind::SassDedent
            | SyntaxKind::Semicolon
            | SyntaxKind::SassOptionalSemicolon
    )
}

fn is_selector_boundary_until(kind: SyntaxKind, recovery: &[SyntaxKind]) -> bool {
    is_selector_boundary(kind) || recovery.contains(&kind)
}

fn is_selector_list_pseudo_class(text: &str) -> bool {
    matches!(text, "is" | "where" | "local" | "global")
}

fn is_nth_pseudo_class(text: &str) -> bool {
    matches!(
        text,
        "nth-child" | "nth-last-child" | "nth-of-type" | "nth-last-of-type"
    )
}

fn language_tag_token_can_start(kind: SyntaxKind) -> bool {
    matches!(kind, SyntaxKind::Ident | SyntaxKind::String)
}

fn selector_item_token_is_recoverable(kind: SyntaxKind) -> bool {
    matches!(
        kind,
        SyntaxKind::Whitespace
            | SyntaxKind::SassIndentedNewline
            | SyntaxKind::Dot
            | SyntaxKind::Comma
            | SyntaxKind::Hash
            | SyntaxKind::Ident
            | SyntaxKind::CustomPropertyName
            | SyntaxKind::String
            | SyntaxKind::Number
            | SyntaxKind::Percentage
            | SyntaxKind::Dimension
            | SyntaxKind::Star
            | SyntaxKind::Ampersand
            | SyntaxKind::ScssPlaceholder
            | SyntaxKind::LeftBracket
            | SyntaxKind::RightBracket
            | SyntaxKind::Colon
            | SyntaxKind::DoubleColon
            | SyntaxKind::LeftParen
            | SyntaxKind::RightParen
            | SyntaxKind::Equals
            | SyntaxKind::IncludesMatch
            | SyntaxKind::DashMatch
            | SyntaxKind::PrefixMatch
            | SyntaxKind::SuffixMatch
            | SyntaxKind::SubstringMatch
            | SyntaxKind::Pipe
            | SyntaxKind::ColumnCombinator
            | SyntaxKind::GreaterThan
            | SyntaxKind::Plus
            | SyntaxKind::Minus
            | SyntaxKind::Tilde
            | SyntaxKind::KeywordAnd
            | SyntaxKind::KeywordOr
            | SyntaxKind::KeywordNot
    )
}

fn is_at_rule_prelude_boundary(kind: SyntaxKind) -> bool {
    matches!(
        kind,
        SyntaxKind::LeftBrace
            | SyntaxKind::SassIndent
            | SyntaxKind::Semicolon
            | SyntaxKind::SassOptionalSemicolon
    )
}

fn is_statement_end(kind: SyntaxKind) -> bool {
    matches!(
        kind,
        SyntaxKind::Semicolon | SyntaxKind::SassOptionalSemicolon
    )
}

fn sass_token_can_end_statement(kind: SyntaxKind) -> bool {
    !matches!(
        kind,
        SyntaxKind::Whitespace
            | SyntaxKind::LineComment
            | SyntaxKind::BlockComment
            | SyntaxKind::SassIndentedNewline
            | SyntaxKind::SassIndent
            | SyntaxKind::SassDedent
            | SyntaxKind::SassOptionalSemicolon
            | SyntaxKind::Comma
            | SyntaxKind::Colon
            | SyntaxKind::DoubleColon
            | SyntaxKind::LeftBrace
            | SyntaxKind::LeftParen
            | SyntaxKind::LeftBracket
            | SyntaxKind::Plus
            | SyntaxKind::Minus
            | SyntaxKind::Star
            | SyntaxKind::Slash
            | SyntaxKind::GreaterThan
            | SyntaxKind::LessThan
            | SyntaxKind::Equals
            | SyntaxKind::Arrow
            | SyntaxKind::Pipe
            | SyntaxKind::Tilde
            | SyntaxKind::Caret
            | SyntaxKind::Ampersand
            | SyntaxKind::DoubleAmpersand
            | SyntaxKind::ColumnCombinator
            | SyntaxKind::IncludesMatch
            | SyntaxKind::DashMatch
            | SyntaxKind::PrefixMatch
            | SyntaxKind::SuffixMatch
            | SyntaxKind::SubstringMatch
            | SyntaxKind::PlusEquals
            | SyntaxKind::MinusEquals
            | SyntaxKind::SlashEquals
    )
}

fn function_argument_recovery(recovery: &[SyntaxKind]) -> Vec<SyntaxKind> {
    let mut kinds = vec![SyntaxKind::RightParen];
    for kind in recovery {
        if !kinds.contains(kind) {
            kinds.push(*kind);
        }
    }
    kinds
}

fn bracketed_value_recovery(recovery: &[SyntaxKind]) -> Vec<SyntaxKind> {
    let mut kinds = vec![SyntaxKind::RightBracket];
    for kind in recovery {
        if !kinds.contains(kind) {
            kinds.push(*kind);
        }
    }
    kinds
}

fn simple_block_recovery(close_kind: SyntaxKind, recovery: &[SyntaxKind]) -> Vec<SyntaxKind> {
    let mut kinds = vec![close_kind];
    for kind in recovery {
        if !kinds.contains(kind) {
            kinds.push(*kind);
        }
    }
    kinds
}

fn matching_simple_block_close(open_kind: SyntaxKind) -> Option<SyntaxKind> {
    match open_kind {
        SyntaxKind::LeftBrace => Some(SyntaxKind::RightBrace),
        SyntaxKind::LeftBracket => Some(SyntaxKind::RightBracket),
        SyntaxKind::LeftParen => Some(SyntaxKind::RightParen),
        _ => None,
    }
}

fn value_list_item_recovery(recovery: &[SyntaxKind]) -> Vec<SyntaxKind> {
    let mut kinds = vec![SyntaxKind::Comma];
    for kind in recovery {
        if !kinds.contains(kind) {
            kinds.push(*kind);
        }
    }
    kinds
}

fn comma_separated_component_value_list_item_recovery(recovery: &[SyntaxKind]) -> Vec<SyntaxKind> {
    let mut kinds = vec![SyntaxKind::Comma];
    for kind in recovery {
        if !kinds.contains(kind) {
            kinds.push(*kind);
        }
    }
    kinds
}

fn variable_declaration_node_kind(kind: SyntaxKind, has_colon: bool) -> SyntaxKind {
    if has_colon {
        return kind;
    }
    match kind {
        SyntaxKind::ScssVariableDeclaration => SyntaxKind::BogusScssVariable,
        SyntaxKind::LessVariableDeclaration => SyntaxKind::BogusLessVariable,
        _ => kind,
    }
}

fn is_attribute_matcher(kind: SyntaxKind) -> bool {
    matches!(
        kind,
        SyntaxKind::Equals
            | SyntaxKind::IncludesMatch
            | SyntaxKind::DashMatch
            | SyntaxKind::PrefixMatch
            | SyntaxKind::SuffixMatch
            | SyntaxKind::SubstringMatch
    )
}

fn attribute_name_token_can_start(kind: SyntaxKind) -> bool {
    matches!(
        kind,
        SyntaxKind::Ident | SyntaxKind::CustomPropertyName | SyntaxKind::Star
    )
}

fn attribute_name_token_can_continue(kind: SyntaxKind) -> bool {
    matches!(
        kind,
        SyntaxKind::Ident
            | SyntaxKind::CustomPropertyName
            | SyntaxKind::Star
            | SyntaxKind::Pipe
            | SyntaxKind::ColumnCombinator
    )
}

fn attribute_value_token_can_start(kind: SyntaxKind) -> bool {
    matches!(
        kind,
        SyntaxKind::Ident
            | SyntaxKind::CustomPropertyName
            | SyntaxKind::String
            | SyntaxKind::Hash
            | SyntaxKind::Number
            | SyntaxKind::Dimension
    )
}

fn is_combinator(kind: SyntaxKind) -> bool {
    matches!(
        kind,
        SyntaxKind::GreaterThan
            | SyntaxKind::Plus
            | SyntaxKind::Tilde
            | SyntaxKind::ColumnCombinator
    )
}

fn infix_binding_power(kind: SyntaxKind) -> Option<(u8, u8)> {
    match kind {
        SyntaxKind::Plus | SyntaxKind::Minus => Some((1, 2)),
        SyntaxKind::Star | SyntaxKind::Slash | SyntaxKind::Percent => Some((3, 4)),
        _ => None,
    }
}

fn specialized_function_kind(text: &str) -> Option<SyntaxKind> {
    if text.eq_ignore_ascii_case("var") {
        return Some(SyntaxKind::VarFunction);
    }
    if text.eq_ignore_ascii_case("calc") {
        return Some(SyntaxKind::CalcFunction);
    }
    if text.eq_ignore_ascii_case("env") {
        return Some(SyntaxKind::EnvFunction);
    }
    if text.eq_ignore_ascii_case("attr") {
        return Some(SyntaxKind::AttrFunction);
    }
    if matches_ignore_ascii_case(
        text,
        &[
            "min", "max", "clamp", "round", "mod", "rem", "sin", "cos", "tan", "asin", "acos",
            "atan", "atan2", "pow", "sqrt", "hypot", "log", "exp", "abs", "sign",
        ],
    ) {
        return Some(SyntaxKind::MathFunction);
    }
    if matches_ignore_ascii_case(
        text,
        &[
            "rgb",
            "rgba",
            "hsl",
            "hsla",
            "hwb",
            "lab",
            "lch",
            "oklab",
            "oklch",
            "color",
            "color-mix",
            "device-cmyk",
            "light-dark",
            "contrast-color",
        ],
    ) {
        return Some(SyntaxKind::ColorValue);
    }
    if matches_ignore_ascii_case(
        text,
        &[
            "linear-gradient",
            "radial-gradient",
            "conic-gradient",
            "repeating-linear-gradient",
            "repeating-radial-gradient",
            "repeating-conic-gradient",
        ],
    ) {
        return Some(SyntaxKind::GradientFunction);
    }
    if matches_ignore_ascii_case(
        text,
        &[
            "matrix",
            "matrix3d",
            "translate",
            "translate3d",
            "translateX",
            "translateY",
            "translateZ",
            "scale",
            "scale3d",
            "scaleX",
            "scaleY",
            "scaleZ",
            "rotate",
            "rotate3d",
            "rotateX",
            "rotateY",
            "rotateZ",
            "skew",
            "skewX",
            "skewY",
            "perspective",
        ],
    ) {
        return Some(SyntaxKind::TransformFunction);
    }
    if matches_ignore_ascii_case(
        text,
        &[
            "blur",
            "brightness",
            "contrast",
            "drop-shadow",
            "grayscale",
            "hue-rotate",
            "invert",
            "opacity",
            "saturate",
            "sepia",
        ],
    ) {
        return Some(SyntaxKind::FilterFunction);
    }
    if matches_ignore_ascii_case(
        text,
        &["image", "image-set", "cross-fade", "element", "paint"],
    ) {
        return Some(SyntaxKind::ImageFunction);
    }
    if matches_ignore_ascii_case(
        text,
        &[
            "path", "shape", "ray", "inset", "circle", "ellipse", "polygon",
        ],
    ) {
        return Some(SyntaxKind::ShapeFunction);
    }
    None
}

fn function_argument_count_is_valid(function_name: &str, argument_count: usize) -> bool {
    if function_name.eq_ignore_ascii_case("calc") {
        return argument_count == 1;
    }
    if matches_ignore_ascii_case(function_name, &["min", "max", "hypot"]) {
        return argument_count >= 1;
    }
    if function_name.eq_ignore_ascii_case("clamp") {
        return argument_count == 3;
    }
    if function_name.eq_ignore_ascii_case("round") {
        return (2..=3).contains(&argument_count);
    }
    if function_name.eq_ignore_ascii_case("log") {
        return (1..=2).contains(&argument_count);
    }
    if matches_ignore_ascii_case(function_name, &["mod", "rem", "pow", "atan2"]) {
        return argument_count == 2;
    }
    if matches_ignore_ascii_case(
        function_name,
        &[
            "sin", "cos", "tan", "asin", "acos", "atan", "sqrt", "exp", "abs", "sign",
        ],
    ) {
        return argument_count == 1;
    }
    if function_name.eq_ignore_ascii_case("color-mix") {
        return argument_count == 3;
    }
    if function_name.eq_ignore_ascii_case("light-dark") {
        return argument_count == 2;
    }
    if function_name.eq_ignore_ascii_case("contrast-color") {
        return argument_count == 1;
    }
    true
}

fn function_requires_filled_top_level_arguments(function_name: &str) -> bool {
    function_name.eq_ignore_ascii_case("calc")
        || matches_ignore_ascii_case(
            function_name,
            &[
                "min", "max", "clamp", "round", "mod", "rem", "sin", "cos", "tan", "asin", "acos",
                "atan", "atan2", "pow", "sqrt", "hypot", "log", "exp", "abs", "sign",
            ],
        )
        || matches_ignore_ascii_case(
            function_name,
            &["color-mix", "light-dark", "contrast-color"],
        )
}

fn at_rule_prelude_head_is_custom_property_name(kind: SyntaxKind) -> bool {
    kind == SyntaxKind::CustomPropertyName || is_interpolation_start(kind)
}

fn at_rule_prelude_head_is_custom_ident(kind: SyntaxKind) -> bool {
    kind == SyntaxKind::Ident || is_interpolation_start(kind)
}

fn is_dynamic_function_argument_head(kind: SyntaxKind) -> bool {
    matches!(
        kind,
        SyntaxKind::ScssVariable
            | SyntaxKind::LessVariable
            | SyntaxKind::ScssInterpolationStart
            | SyntaxKind::LessInterpolationStart
    )
}

fn is_scss_module_source_token(kind: SyntaxKind) -> bool {
    matches!(
        kind,
        SyntaxKind::String | SyntaxKind::Url | SyntaxKind::ScssInterpolationStart
    )
}

fn is_scss_module_namespace_token(kind: SyntaxKind) -> bool {
    matches!(
        kind,
        SyntaxKind::Ident | SyntaxKind::Star | SyntaxKind::ScssInterpolationStart
    )
}

fn is_scss_module_visibility_name_token(kind: SyntaxKind) -> bool {
    matches!(
        kind,
        SyntaxKind::Ident
            | SyntaxKind::ScssVariable
            | SyntaxKind::ScssPlaceholder
            | SyntaxKind::ScssInterpolationStart
    )
}

fn is_css_module_from_source_token(kind: SyntaxKind, text: &str) -> bool {
    matches!(
        kind,
        SyntaxKind::String
            | SyntaxKind::Url
            | SyntaxKind::ScssInterpolationStart
            | SyntaxKind::LessInterpolationStart
    ) || (kind == SyntaxKind::Ident && text == "global")
}

fn is_scss_control_rule_kind(kind: SyntaxKind) -> bool {
    matches!(
        kind,
        SyntaxKind::ScssControlIf
            | SyntaxKind::ScssControlElse
            | SyntaxKind::ScssControlEach
            | SyntaxKind::ScssControlFor
            | SyntaxKind::ScssControlWhile
    )
}

fn matches_ignore_ascii_case(value: &str, candidates: &[&str]) -> bool {
    candidates
        .iter()
        .any(|candidate| value.eq_ignore_ascii_case(candidate))
}

fn css_module_scope_function_kind(text: &str) -> Option<SyntaxKind> {
    match text {
        "local" => Some(SyntaxKind::CssModuleLocalBlock),
        "global" => Some(SyntaxKind::CssModuleGlobalBlock),
        _ => None,
    }
}

fn text_range(start: usize, end: usize) -> TextRange {
    TextRange::new(TextSize::from(start as u32), TextSize::from(end as u32))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn builds_cst_root_for_plain_css() {
        let result = parse(".button { color: red; }", StyleDialect::Css);

        assert_eq!(result.syntax().kind(), SyntaxKind::Root);
        assert_eq!(result.dialect(), StyleDialect::Css);
        assert!(
            result.errors().is_empty(),
            "unexpected parse errors: {:?}",
            result.errors()
        );
        assert!(result.token_count() > 0);

        let kinds = node_kinds(&result.syntax());
        assert!(kinds.contains(&SyntaxKind::Rule));
        assert!(kinds.contains(&SyntaxKind::SelectorList));
        assert!(kinds.contains(&SyntaxKind::DeclarationList));
        assert!(kinds.contains(&SyntaxKind::Declaration));
        assert!(kinds.contains(&SyntaxKind::PropertyName));
        assert!(kinds.contains(&SyntaxKind::Value));
    }

    #[test]
    fn exposes_css_syntax_parser_entry_points() {
        let rule_list = parse_entry_point(
            ".button { color: red; } @media (width >= 1px) { .card { color: blue; } }",
            StyleDialect::Css,
            ParseEntryPoint::RuleList,
        );
        let rule = parse_entry_point(
            ".button { color: red; }",
            StyleDialect::Css,
            ParseEntryPoint::Rule,
        );
        let declaration_list = parse_entry_point(
            "color: red; width: calc(1px + 2px);",
            StyleDialect::Css,
            ParseEntryPoint::DeclarationList,
        );
        let declaration = parse_entry_point(
            "color: red;",
            StyleDialect::Css,
            ParseEntryPoint::Declaration,
        );
        let value = parse_entry_point(
            "clamp(1rem, calc(2px + 3px), 4rem)",
            StyleDialect::Css,
            ParseEntryPoint::Value,
        );
        let component_value = parse_entry_point(
            "calc(100% - var(--gap))",
            StyleDialect::Css,
            ParseEntryPoint::ComponentValue,
        );
        let component_value_list = parse_entry_point(
            "red + calc(1px + 2px) [data-state]",
            StyleDialect::Css,
            ParseEntryPoint::ComponentValueList,
        );
        let comma_separated_component_value_list = parse_entry_point(
            "red, calc(1px + 2px), [data-state]",
            StyleDialect::Css,
            ParseEntryPoint::CommaSeparatedComponentValueList,
        );
        let simple_block = parse_entry_point(
            "{ color: red; [data-state] }",
            StyleDialect::Css,
            ParseEntryPoint::SimpleBlock,
        );
        let unclosed_simple_block = parse_entry_point(
            "{ color: red",
            StyleDialect::Css,
            ParseEntryPoint::SimpleBlock,
        );

        assert!(rule_list.errors().is_empty());
        assert!(rule.errors().is_empty());
        assert!(declaration_list.errors().is_empty());
        assert!(declaration.errors().is_empty());
        assert!(value.errors().is_empty());
        assert!(component_value.errors().is_empty());
        assert!(component_value_list.errors().is_empty());
        assert!(comma_separated_component_value_list.errors().is_empty());
        assert!(simple_block.errors().is_empty());
        assert_eq!(unclosed_simple_block.errors().len(), 1);
        assert!(node_kinds(&rule_list.syntax()).contains(&SyntaxKind::RuleList));
        assert!(node_kinds(&rule.syntax()).contains(&SyntaxKind::Rule));
        assert!(node_kinds(&declaration_list.syntax()).contains(&SyntaxKind::DeclarationList));
        assert!(node_kinds(&declaration.syntax()).contains(&SyntaxKind::Declaration));
        assert!(node_kinds(&value.syntax()).contains(&SyntaxKind::Value));
        assert!(node_kinds(&value.syntax()).contains(&SyntaxKind::CalcFunction));
        assert!(node_kinds(&component_value.syntax()).contains(&SyntaxKind::ComponentValue));
        assert!(node_kinds(&component_value.syntax()).contains(&SyntaxKind::FunctionCall));
        assert!(
            node_kinds(&component_value_list.syntax()).contains(&SyntaxKind::ComponentValueList)
        );
        assert!(
            node_kinds(&comma_separated_component_value_list.syntax())
                .contains(&SyntaxKind::CommaSeparatedComponentValueList)
        );
        assert!(node_kinds(&simple_block.syntax()).contains(&SyntaxKind::SimpleBlock));
        assert!(node_kinds(&simple_block.syntax()).contains(&SyntaxKind::ComponentValue));
        assert!(
            node_kinds(&unclosed_simple_block.syntax()).contains(&SyntaxKind::BogusSimpleBlock)
        );
    }

    #[test]
    fn tokenizes_multibyte_source_without_boundary_errors() {
        let result = parse(".카드 { --간격: \"좋음\"; }", StyleDialect::Css);

        assert!(
            result.errors().is_empty(),
            "unexpected parse errors: {:?}",
            result.errors()
        );
        assert!(result.token_count() >= 8);
    }

    #[test]
    fn reports_unterminated_constructs_without_panicking() {
        let comment = parse("/* open", StyleDialect::Css);
        let string = parse(".a { content: \"open; }", StyleDialect::Css);
        let block = parse(".a { color: red", StyleDialect::Css);

        assert_eq!(
            comment.errors().first().map(|error| error.code),
            Some(ParseErrorCode::UnterminatedBlockComment),
        );
        assert_eq!(
            string.errors().first().map(|error| error.code),
            Some(ParseErrorCode::UnterminatedString),
        );
        assert_eq!(
            block.errors().first().map(|error| error.code),
            Some(ParseErrorCode::UnexpectedCharacter),
        );
        assert!(node_kinds(&block.syntax()).contains(&SyntaxKind::BogusTrivia));
    }

    #[test]
    fn classifies_initial_dialect_tokens() {
        let scss = parse("$gap: 1rem;", StyleDialect::Scss);
        let less = parse("@gap: 1rem;", StyleDialect::Less);
        let less_at_rule = parse("@media screen {}", StyleDialect::Less);
        let scss_kinds = node_kinds(&scss.syntax());
        let less_kinds = node_kinds(&less.syntax());

        assert_eq!(scss.syntax().kind(), SyntaxKind::Root);
        assert_eq!(less.syntax().kind(), SyntaxKind::Root);
        assert_eq!(less_at_rule.syntax().kind(), SyntaxKind::Root);
        assert!(scss.errors().is_empty());
        assert!(less.errors().is_empty());
        assert!(less_at_rule.errors().is_empty());
        assert!(scss_kinds.contains(&SyntaxKind::ScssVariableDeclaration));
        assert!(less_kinds.contains(&SyntaxKind::LessVariableDeclaration));
    }

    #[test]
    fn exposes_lex_result_for_tokenizer_gates() {
        let scss = lex("$gap: 1rem;", StyleDialect::Scss);
        let less = lex("@gap: 1rem;", StyleDialect::Less);
        let less_at_rule = lex("@media screen {}", StyleDialect::Less);
        let css_slashes = lex("// not a css comment", StyleDialect::Css);
        let scss_slashes = lex("// scss comment", StyleDialect::Scss);

        assert_eq!(
            scss.tokens().first().map(|token| token.kind),
            Some(SyntaxKind::ScssVariable)
        );
        assert_eq!(
            scss.tokens().first().map(|token| token.text.as_str()),
            Some("$gap")
        );
        assert_eq!(
            less.tokens().first().map(|token| token.kind),
            Some(SyntaxKind::LessVariable)
        );
        assert_eq!(
            less_at_rule.tokens().first().map(|token| token.kind),
            Some(SyntaxKind::AtKeyword),
        );
        assert_eq!(
            css_slashes.tokens().first().map(|token| token.kind),
            Some(SyntaxKind::Slash)
        );
        assert_eq!(
            scss_slashes.tokens().first().map(|token| token.kind),
            Some(SyntaxKind::LineComment),
        );
    }

    #[test]
    fn tokenizes_css_attribute_matchers_as_single_tokens() {
        let result = lex(
            ".a[data-state~=\"active\"][lang|=\"en\"][href^=\"/docs\"][href$=\".pdf\"][class*=\"btn\"] { width += 1px; }",
            StyleDialect::Css,
        );
        let kinds: Vec<SyntaxKind> = result.tokens().iter().map(|token| token.kind).collect();

        assert!(result.errors().is_empty());
        assert!(kinds.contains(&SyntaxKind::IncludesMatch));
        assert!(kinds.contains(&SyntaxKind::DashMatch));
        assert!(kinds.contains(&SyntaxKind::PrefixMatch));
        assert!(kinds.contains(&SyntaxKind::SuffixMatch));
        assert!(kinds.contains(&SyntaxKind::SubstringMatch));
        assert!(kinds.contains(&SyntaxKind::PlusEquals));
    }

    #[test]
    fn tokenizes_important_annotation_as_single_token() {
        let result = lex(".a { color: red !IMPORTANT; }", StyleDialect::Css);
        let kinds: Vec<SyntaxKind> = result.tokens().iter().map(|token| token.kind).collect();

        assert!(result.errors().is_empty());
        assert!(kinds.contains(&SyntaxKind::Important));
        assert!(!kinds.contains(&SyntaxKind::Delim));
    }

    #[test]
    fn tokenizes_cdo_cdc_and_ignores_them_at_top_level() {
        let result = parse("<!-- .a { color: red; } -->", StyleDialect::Css);
        let token_kinds = token_kinds(&result.syntax());

        assert!(result.errors().is_empty());
        assert!(token_kinds.contains(&SyntaxKind::Cdo));
        assert!(token_kinds.contains(&SyntaxKind::Cdc));
        assert!(node_kinds(&result.syntax()).contains(&SyntaxKind::Rule));
    }

    #[test]
    fn tokenizes_css_identifier_escapes_without_unexpected_errors() {
        let result = parse(".\\31 0 { color: var(--\\67 ap); }", StyleDialect::Css);
        let token_kinds = token_kinds(&result.syntax());

        assert!(result.errors().is_empty());
        assert!(token_kinds.contains(&SyntaxKind::Ident));
        assert!(token_kinds.contains(&SyntaxKind::CustomPropertyName));
        assert!(node_kinds(&result.syntax()).contains(&SyntaxKind::ClassSelector));
    }

    #[test]
    fn tokenizes_bare_hash_as_delim_and_hash_names_as_hash() {
        let bare = lex("# { color: red; }", StyleDialect::Css);
        let named = lex("#main { color: red; }", StyleDialect::Css);
        let escaped = lex("#\\31 0 { color: red; }", StyleDialect::Css);
        let bare_kinds: Vec<SyntaxKind> = bare.tokens().iter().map(|token| token.kind).collect();
        let named_kinds: Vec<SyntaxKind> = named.tokens().iter().map(|token| token.kind).collect();
        let escaped_kinds: Vec<SyntaxKind> =
            escaped.tokens().iter().map(|token| token.kind).collect();

        assert!(bare.errors().is_empty());
        assert!(named.errors().is_empty());
        assert!(escaped.errors().is_empty());
        assert!(bare_kinds.contains(&SyntaxKind::Delim));
        assert!(!bare_kinds.contains(&SyntaxKind::Hash));
        assert!(named_kinds.contains(&SyntaxKind::Hash));
        assert!(escaped_kinds.contains(&SyntaxKind::Hash));
    }

    #[test]
    fn tokenizes_dash_started_idents_and_custom_properties_by_ident_rules() {
        let vendor = lex("-webkit-transform", StyleDialect::Css);
        let custom = lex("--brand", StyleDialect::Css);
        let escaped_custom = lex("--\\31 0", StyleDialect::Css);
        let bare_dash = lex("--:", StyleDialect::Css);
        let vendor_kinds: Vec<SyntaxKind> =
            vendor.tokens().iter().map(|token| token.kind).collect();
        let custom_kinds: Vec<SyntaxKind> =
            custom.tokens().iter().map(|token| token.kind).collect();
        let escaped_custom_kinds: Vec<SyntaxKind> = escaped_custom
            .tokens()
            .iter()
            .map(|token| token.kind)
            .collect();
        let bare_dash_kinds: Vec<SyntaxKind> =
            bare_dash.tokens().iter().map(|token| token.kind).collect();

        assert!(vendor.errors().is_empty());
        assert!(custom.errors().is_empty());
        assert!(escaped_custom.errors().is_empty());
        assert!(bare_dash.errors().is_empty());
        assert!(vendor_kinds.contains(&SyntaxKind::Ident));
        assert!(!vendor_kinds.contains(&SyntaxKind::Minus));
        assert!(custom_kinds.contains(&SyntaxKind::CustomPropertyName));
        assert!(escaped_custom_kinds.contains(&SyntaxKind::CustomPropertyName));
        assert!(!bare_dash_kinds.contains(&SyntaxKind::CustomPropertyName));
        assert!(bare_dash_kinds.contains(&SyntaxKind::Ident));
    }

    #[test]
    fn tokenizes_signed_and_leading_dot_numbers_as_single_numeric_tokens() {
        let signed_number = lex("+1.5", StyleDialect::Css);
        let signed_dimension = lex("-2px", StyleDialect::Css);
        let leading_dot = lex(".5", StyleDialect::Css);
        let spaced_plus = lex("+ 1.5", StyleDialect::Css);
        let trailing_dot = lex("1.", StyleDialect::Css);
        let signed_number_kinds: Vec<SyntaxKind> = signed_number
            .tokens()
            .iter()
            .map(|token| token.kind)
            .collect();
        let signed_dimension_kinds: Vec<SyntaxKind> = signed_dimension
            .tokens()
            .iter()
            .map(|token| token.kind)
            .collect();
        let leading_dot_kinds: Vec<SyntaxKind> = leading_dot
            .tokens()
            .iter()
            .map(|token| token.kind)
            .collect();
        let spaced_plus_kinds: Vec<SyntaxKind> = spaced_plus
            .tokens()
            .iter()
            .map(|token| token.kind)
            .collect();
        let trailing_dot_kinds: Vec<SyntaxKind> = trailing_dot
            .tokens()
            .iter()
            .map(|token| token.kind)
            .collect();

        assert!(signed_number.errors().is_empty());
        assert!(signed_dimension.errors().is_empty());
        assert!(leading_dot.errors().is_empty());
        assert!(spaced_plus.errors().is_empty());
        assert!(trailing_dot.errors().is_empty());
        assert_eq!(signed_number_kinds, vec![SyntaxKind::Number]);
        assert_eq!(signed_dimension_kinds, vec![SyntaxKind::Dimension]);
        assert_eq!(leading_dot_kinds, vec![SyntaxKind::Number]);
        assert!(spaced_plus_kinds.contains(&SyntaxKind::Plus));
        assert!(spaced_plus_kinds.contains(&SyntaxKind::Number));
        assert_eq!(
            trailing_dot_kinds,
            vec![SyntaxKind::Number, SyntaxKind::Dot]
        );
    }

    #[test]
    fn tokenizes_exponent_numbers_before_dimension_suffixes() {
        let exponent = lex("1e3", StyleDialect::Css);
        let signed_exponent = lex("1e-3", StyleDialect::Css);
        let exponent_dimension = lex("1e3px", StyleDialect::Css);
        let plain_dimension = lex("1em", StyleDialect::Css);
        let exponent_kinds: Vec<SyntaxKind> =
            exponent.tokens().iter().map(|token| token.kind).collect();
        let signed_exponent_kinds: Vec<SyntaxKind> = signed_exponent
            .tokens()
            .iter()
            .map(|token| token.kind)
            .collect();
        let exponent_dimension_kinds: Vec<SyntaxKind> = exponent_dimension
            .tokens()
            .iter()
            .map(|token| token.kind)
            .collect();
        let plain_dimension_kinds: Vec<SyntaxKind> = plain_dimension
            .tokens()
            .iter()
            .map(|token| token.kind)
            .collect();

        assert!(exponent.errors().is_empty());
        assert!(signed_exponent.errors().is_empty());
        assert!(exponent_dimension.errors().is_empty());
        assert!(plain_dimension.errors().is_empty());
        assert_eq!(exponent_kinds, vec![SyntaxKind::Number]);
        assert_eq!(signed_exponent_kinds, vec![SyntaxKind::Number]);
        assert_eq!(exponent_dimension_kinds, vec![SyntaxKind::Dimension]);
        assert_eq!(plain_dimension_kinds, vec![SyntaxKind::Dimension]);
    }

    #[test]
    fn tokenizes_null_and_bom_without_unexpected_errors() {
        let result = parse("\u{feff}.a\0b { content: \0; }", StyleDialect::Css);
        let lexed = lex(
            "\u{feff}.a\0b { background: url(foo\0bar); }",
            StyleDialect::Css,
        );
        let token_kinds = token_kinds(&result.syntax());
        let ident = lexed
            .tokens()
            .iter()
            .find(|token| token.kind == SyntaxKind::Ident)
            .map(|token| token.text.as_str());
        let url = lexed
            .tokens()
            .iter()
            .find(|token| token.kind == SyntaxKind::Url)
            .map(|token| token.text.as_str());

        assert!(result.errors().is_empty());
        assert!(lexed.errors().is_empty());
        assert_eq!(
            lexed.tokens().first().map(|token| token.kind),
            Some(SyntaxKind::Dot)
        );
        assert_eq!(ident, Some("a\u{fffd}b"));
        assert_eq!(url, Some("url(foo\u{fffd}bar)"));
        assert!(
            !lexed
                .tokens()
                .iter()
                .any(|token| token.text.contains('\0') || token.text.contains('\u{feff}'))
        );
        assert!(token_kinds.contains(&SyntaxKind::Whitespace));
        assert!(token_kinds.contains(&SyntaxKind::Ident));
        assert!(node_kinds(&result.syntax()).contains(&SyntaxKind::ClassSelector));
    }

    #[test]
    fn tokenizes_unquoted_urls_and_bad_urls() {
        let good = lex(".a { background: url(images/bg.png); }", StyleDialect::Css);
        let bad = lex(".a { background: url(foo\"bar); }", StyleDialect::Css);
        let bad_whitespace = lex(".a { background: url(foo bar); }", StyleDialect::Css);
        let bad_escape = lex(".a { background: url(foo\\\nbar); }", StyleDialect::Css);
        let trailing_whitespace = lex(".a { background: url(foo \n ); }", StyleDialect::Css);
        let quoted = lex(
            ".a { background: url(\"images/bg.png\"); }",
            StyleDialect::Css,
        );
        let good_kinds: Vec<SyntaxKind> = good.tokens().iter().map(|token| token.kind).collect();
        let bad_kinds: Vec<SyntaxKind> = bad.tokens().iter().map(|token| token.kind).collect();
        let bad_whitespace_kinds: Vec<SyntaxKind> = bad_whitespace
            .tokens()
            .iter()
            .map(|token| token.kind)
            .collect();
        let bad_escape_kinds: Vec<SyntaxKind> =
            bad_escape.tokens().iter().map(|token| token.kind).collect();
        let trailing_whitespace_kinds: Vec<SyntaxKind> = trailing_whitespace
            .tokens()
            .iter()
            .map(|token| token.kind)
            .collect();
        let quoted_kinds: Vec<SyntaxKind> =
            quoted.tokens().iter().map(|token| token.kind).collect();

        assert!(good.errors().is_empty());
        assert!(good_kinds.contains(&SyntaxKind::Url));
        assert!(bad_kinds.contains(&SyntaxKind::BadUrl));
        assert!(!bad.errors().is_empty());
        assert!(bad_whitespace_kinds.contains(&SyntaxKind::BadUrl));
        assert!(!bad_whitespace.errors().is_empty());
        assert!(bad_escape_kinds.contains(&SyntaxKind::BadUrl));
        assert!(!bad_escape.errors().is_empty());
        assert!(trailing_whitespace.errors().is_empty());
        assert!(trailing_whitespace_kinds.contains(&SyntaxKind::Url));
        assert!(quoted_kinds.contains(&SyntaxKind::Ident));
        assert!(quoted_kinds.contains(&SyntaxKind::String));
        assert!(!quoted_kinds.contains(&SyntaxKind::Url));
    }

    #[test]
    fn tokenizes_unicode_ranges() {
        let result = lex(
            "@font-face { unicode-range: U+00A0-00FF, u+4??; }",
            StyleDialect::Css,
        );
        let kinds: Vec<SyntaxKind> = result.tokens().iter().map(|token| token.kind).collect();

        assert!(result.errors().is_empty());
        assert_eq!(
            kinds
                .iter()
                .filter(|kind| **kind == SyntaxKind::UnicodeRange)
                .count(),
            2
        );
    }

    #[test]
    fn tokenizes_scss_interpolation_delimiters() {
        let scss = lex(
            ".button-#{$variant} { color: #{$color}; }",
            StyleDialect::Scss,
        );
        let css = lex(".button-#{$variant} { color: red; }", StyleDialect::Css);
        let scss_kinds: Vec<SyntaxKind> = scss.tokens().iter().map(|token| token.kind).collect();
        let css_kinds: Vec<SyntaxKind> = css.tokens().iter().map(|token| token.kind).collect();

        assert!(scss.errors().is_empty());
        assert!(scss_kinds.contains(&SyntaxKind::ScssInterpolationStart));
        assert!(scss_kinds.contains(&SyntaxKind::ScssInterpolationEnd));
        assert!(!css_kinds.contains(&SyntaxKind::ScssInterpolationStart));
    }

    #[test]
    fn tokenizes_scss_placeholder_selectors() {
        let scss = lex("%button { color: red; }", StyleDialect::Scss);
        let css = lex("%button { color: red; }", StyleDialect::Css);
        let scss_kinds: Vec<SyntaxKind> = scss.tokens().iter().map(|token| token.kind).collect();
        let css_kinds: Vec<SyntaxKind> = css.tokens().iter().map(|token| token.kind).collect();

        assert!(scss.errors().is_empty());
        assert!(scss_kinds.contains(&SyntaxKind::ScssPlaceholder));
        assert!(css_kinds.contains(&SyntaxKind::Percent));
        assert!(!css_kinds.contains(&SyntaxKind::ScssPlaceholder));
    }

    #[test]
    fn tokenizes_sass_indented_block_markers() {
        let result = lex(
            ".card\n  color: red // comment\n  .title\n    color: blue\n",
            StyleDialect::Sass,
        );
        let kinds: Vec<SyntaxKind> = result.tokens().iter().map(|token| token.kind).collect();

        assert!(result.errors().is_empty());
        assert!(kinds.contains(&SyntaxKind::LineComment));
        assert!(kinds.contains(&SyntaxKind::SassIndentedNewline));
        assert!(kinds.contains(&SyntaxKind::SassOptionalSemicolon));
        assert_eq!(
            kinds
                .iter()
                .filter(|kind| **kind == SyntaxKind::SassIndent)
                .count(),
            2
        );
        assert_eq!(
            kinds
                .iter()
                .filter(|kind| **kind == SyntaxKind::SassDedent)
                .count(),
            2
        );
    }

    #[test]
    fn tokenizes_less_interpolation_delimiters() {
        let less = lex(
            ".button-@{variant} { color: @{color}; }",
            StyleDialect::Less,
        );
        let css = lex(".button-@{variant} { color: red; }", StyleDialect::Css);
        let less_kinds: Vec<SyntaxKind> = less.tokens().iter().map(|token| token.kind).collect();
        let css_kinds: Vec<SyntaxKind> = css.tokens().iter().map(|token| token.kind).collect();

        assert!(less.errors().is_empty());
        assert!(less_kinds.contains(&SyntaxKind::LessInterpolationStart));
        assert!(less_kinds.contains(&SyntaxKind::LessInterpolationEnd));
        assert!(!css_kinds.contains(&SyntaxKind::LessInterpolationStart));
    }

    #[test]
    fn tokenizes_less_escaped_strings() {
        let less = lex(".a { filter: ~\"alpha(opacity=50)\"; }", StyleDialect::Less);
        let css = lex(".a { filter: ~\"alpha(opacity=50)\"; }", StyleDialect::Css);
        let less_kinds: Vec<SyntaxKind> = less.tokens().iter().map(|token| token.kind).collect();
        let css_kinds: Vec<SyntaxKind> = css.tokens().iter().map(|token| token.kind).collect();

        assert!(less.errors().is_empty());
        assert!(less_kinds.contains(&SyntaxKind::LessEscapedString));
        assert!(!css_kinds.contains(&SyntaxKind::LessEscapedString));
        assert!(css_kinds.contains(&SyntaxKind::Tilde));
        assert!(css_kinds.contains(&SyntaxKind::String));
    }

    #[test]
    fn tokenizes_less_property_variables_without_breaking_suffix_matchers() {
        let less = lex(
            ".a { background: $color; [data-x$=y] {} }",
            StyleDialect::Less,
        );
        let scss = lex(".a { background: $color; }", StyleDialect::Scss);
        let less_kinds: Vec<SyntaxKind> = less.tokens().iter().map(|token| token.kind).collect();
        let scss_kinds: Vec<SyntaxKind> = scss.tokens().iter().map(|token| token.kind).collect();

        assert!(less.errors().is_empty());
        assert!(scss.errors().is_empty());
        assert!(less_kinds.contains(&SyntaxKind::LessPropertyVariableToken));
        assert!(less_kinds.contains(&SyntaxKind::SuffixMatch));
        assert!(!less_kinds.contains(&SyntaxKind::ScssVariable));
        assert!(scss_kinds.contains(&SyntaxKind::ScssVariable));
    }

    #[test]
    fn tokenizes_newline_bad_strings() {
        let result = lex(".a { content: \"bad\nstill-here: red; }", StyleDialect::Css);
        let kinds: Vec<SyntaxKind> = result.tokens().iter().map(|token| token.kind).collect();

        assert!(kinds.contains(&SyntaxKind::BadString));
        assert!(
            result
                .errors()
                .iter()
                .any(|error| error.code == ParseErrorCode::UnterminatedString)
        );
    }

    #[test]
    fn exposes_recovery_token_sets() {
        assert!(RECOVERY_TOP.contains(SyntaxKind::AtKeyword));
        assert!(RECOVERY_DECLARATION.contains(SyntaxKind::Semicolon));
        assert!(RECOVERY_SELECTOR.contains(SyntaxKind::LeftBrace));
        assert!(!RECOVERY_SELECTOR.is_empty());
    }

    #[test]
    fn builds_at_rule_and_bogus_nodes_for_partial_input() {
        let at_rule = parse("@media screen { .a { color: red; } }", StyleDialect::Css);
        let missing_colon = parse(".a { color red; }", StyleDialect::Css);
        let missing_block = parse(".a color: red;", StyleDialect::Css);

        assert!(node_kinds(&at_rule.syntax()).contains(&SyntaxKind::AtRule));
        assert!(node_kinds(&missing_colon.syntax()).contains(&SyntaxKind::BogusDeclaration));
        assert!(node_kinds(&missing_block.syntax()).contains(&SyntaxKind::BogusRule));
    }

    #[test]
    fn builds_bogus_nodes_for_selector_and_value_recovery() {
        let missing_class_name = parse(". { color: red; }", StyleDialect::Css);
        let missing_attribute_end = parse(".a[data-active { color: red; }", StyleDialect::Css);
        let missing_value_rhs = parse(".a { width: calc(1 + ); }", StyleDialect::Css);
        let unexpected_value_token = parse(".a { color: @; }", StyleDialect::Css);

        assert_eq!(
            missing_class_name.errors().first().map(|error| error.code),
            Some(ParseErrorCode::ExpectedSelectorName)
        );
        assert_eq!(
            missing_attribute_end
                .errors()
                .first()
                .map(|error| error.code),
            Some(ParseErrorCode::UnterminatedAttributeSelector)
        );
        assert!(
            missing_value_rhs
                .errors()
                .iter()
                .any(|error| error.code == ParseErrorCode::ExpectedValue)
        );
        assert!(node_kinds(&missing_class_name.syntax()).contains(&SyntaxKind::BogusSelector));
        assert!(node_kinds(&missing_attribute_end.syntax()).contains(&SyntaxKind::BogusSelector));
        assert!(node_kinds(&missing_value_rhs.syntax()).contains(&SyntaxKind::BogusValue));
        assert!(node_kinds(&unexpected_value_token.syntax()).contains(&SyntaxKind::BogusValue));
    }

    #[test]
    fn recovers_empty_declaration_values_without_rejecting_custom_properties() {
        let result = parse(".a { color: ; width: ; --empty: ; }", StyleDialect::Css);
        let kinds = node_kinds(&result.syntax());
        let empty_value_errors = result
            .errors()
            .iter()
            .filter(|error| error.message == "expected declaration value")
            .count();
        let bogus_value_count = kinds
            .iter()
            .filter(|kind| **kind == SyntaxKind::BogusValue)
            .count();

        assert_eq!(empty_value_errors, 2);
        assert_eq!(bogus_value_count, 2);
        assert!(kinds.contains(&SyntaxKind::CustomPropertyValue));
    }

    #[test]
    fn recovers_empty_variable_values_without_rejecting_less_detached_rulesets() {
        let scss = parse("$gap: ;", StyleDialect::Scss);
        let less = parse("@gap: ; @ruleset: { color: red; };", StyleDialect::Less);
        let scss_kinds = node_kinds(&scss.syntax());
        let less_kinds = node_kinds(&less.syntax());
        let empty_value_errors = scss
            .errors()
            .iter()
            .chain(less.errors())
            .filter(|error| error.message == "expected variable value")
            .count();

        assert_eq!(empty_value_errors, 2);
        assert!(scss_kinds.contains(&SyntaxKind::BogusValue));
        assert!(less_kinds.contains(&SyntaxKind::BogusValue));
        assert!(less_kinds.contains(&SyntaxKind::LessDetachedRulesetNode));
    }

    #[test]
    fn recovers_missing_semicolons_between_declarations() {
        let result = parse(
            ".a { color: red background: blue; margin: 0 padding: 1rem; }",
            StyleDialect::Css,
        );
        let custom_property = parse(
            ".a { --token: red background: blue; color: red; }",
            StyleDialect::Css,
        );
        let kinds = node_kinds(&result.syntax());
        let custom_property_kinds = node_kinds(&custom_property.syntax());
        let declaration_count = kinds
            .iter()
            .filter(|kind| **kind == SyntaxKind::Declaration)
            .count();
        let custom_property_declaration_count = custom_property_kinds
            .iter()
            .filter(|kind| **kind == SyntaxKind::Declaration)
            .count();
        let missing_semicolon_errors = result
            .errors()
            .iter()
            .filter(|error| error.message == "expected semicolon between declarations")
            .count();

        assert_eq!(declaration_count, 4);
        assert_eq!(missing_semicolon_errors, 2);
        assert_eq!(custom_property_declaration_count, 2);
        assert!(custom_property.errors().is_empty());
        assert!(custom_property_kinds.contains(&SyntaxKind::CustomPropertyValue));
    }

    #[test]
    fn populates_core_bogus_nodes_for_recoverable_structures() {
        let missing_function_close =
            parse(".a { width: calc(1 + ; color: red; }", StyleDialect::Css);
        let missing_media_close = parse(
            "@media (min-width: { .a { color: red; } }",
            StyleDialect::Css,
        );
        let mixed_media_close = parse(
            "@media screen, (min-width: { .a { color: red; } }",
            StyleDialect::Css,
        );
        let missing_supports_close = parse(
            "@supports (display: { .a { color: red; } }",
            StyleDialect::Css,
        );
        let missing_container_close = parse(
            "@container (inline-size > { .a { color: red; } }",
            StyleDialect::Css,
        );
        let missing_unknown_prelude_close =
            parse("@unknown (min-width: { color: red; }", StyleDialect::Css);
        let missing_scope_close = parse("@scope (.a { .b { color: red; } }", StyleDialect::Css);
        let empty_layer_statement = parse("@layer ;", StyleDialect::Css);
        let missing_keyframe_block =
            parse("@keyframes fade { from opacity: 0; }", StyleDialect::Css);
        let unclosed_rule = parse(".a { color: red;", StyleDialect::Css);

        assert!(
            node_kinds(&missing_function_close.syntax()).contains(&SyntaxKind::BogusFunctionCall)
        );
        assert!(
            node_kinds(&missing_function_close.syntax())
                .contains(&SyntaxKind::BogusFunctionArguments)
        );
        assert!(node_kinds(&missing_media_close.syntax()).contains(&SyntaxKind::BogusMediaQuery));
        assert!(node_kinds(&mixed_media_close.syntax()).contains(&SyntaxKind::MediaQuery));
        assert!(node_kinds(&mixed_media_close.syntax()).contains(&SyntaxKind::BogusMediaQuery));
        assert!(
            node_kinds(&missing_supports_close.syntax())
                .contains(&SyntaxKind::BogusSupportsCondition)
        );
        assert!(
            node_kinds(&missing_container_close.syntax())
                .contains(&SyntaxKind::BogusContainerCondition)
        );
        assert!(
            node_kinds(&missing_unknown_prelude_close.syntax())
                .contains(&SyntaxKind::BogusAtRulePrelude)
        );
        assert!(node_kinds(&missing_scope_close.syntax()).contains(&SyntaxKind::BogusScopeRange));
        assert!(node_kinds(&empty_layer_statement.syntax()).contains(&SyntaxKind::BogusLayerName));
        assert!(
            node_kinds(&missing_keyframe_block.syntax()).contains(&SyntaxKind::BogusKeyframeBlock)
        );
        assert!(node_kinds(&unclosed_rule.syntax()).contains(&SyntaxKind::BogusDeclarationList));
        assert!(node_kinds(&unclosed_rule.syntax()).contains(&SyntaxKind::BogusTrivia));
    }

    #[test]
    fn populates_dialect_and_selector_bogus_nodes() {
        let invalid_compound = parse("%bad { color: red; }", StyleDialect::Css);
        let dangling_combinator = parse(".a > { color: red; }", StyleDialect::Css);
        let missing_property = parse(".a { : red; }", StyleDialect::Css);
        let missing_colon_recovery = parse("$gap 1rem;", StyleDialect::Scss);
        let unexpected_value_token = parse(".a { width: ?; }", StyleDialect::Css);
        let missing_at_rule_name = parse("@ ;", StyleDialect::Css);
        let missing_scss_variable_colon = parse("$gap;", StyleDialect::Scss);
        let missing_less_variable_colon = parse("@gap;", StyleDialect::Less);
        let missing_scss_blocks =
            parse("@mixin card; @function double; @if $x;", StyleDialect::Scss);
        let inconsistent_sass_indentation =
            parse(".card\n  color: red\n color: blue\n", StyleDialect::Sass);
        let missing_less_mixin_block = parse(".theme(@tone);", StyleDialect::Less);
        let missing_less_guard_condition =
            parse(".theme() when { color: red; }", StyleDialect::Less);

        assert!(
            node_kinds(&invalid_compound.syntax()).contains(&SyntaxKind::BogusCompoundSelector)
        );
        assert!(node_kinds(&dangling_combinator.syntax()).contains(&SyntaxKind::BogusCombinator));
        assert!(node_kinds(&missing_property.syntax()).contains(&SyntaxKind::BogusPropertyName));
        assert!(node_kinds(&missing_colon_recovery.syntax()).contains(&SyntaxKind::BogusRecovery));
        assert!(node_kinds(&unexpected_value_token.syntax()).contains(&SyntaxKind::BogusToken));
        assert!(node_kinds(&missing_at_rule_name.syntax()).contains(&SyntaxKind::BogusAtRule));
        assert!(
            node_kinds(&missing_scss_variable_colon.syntax())
                .contains(&SyntaxKind::BogusScssVariable)
        );
        assert!(
            node_kinds(&missing_less_variable_colon.syntax())
                .contains(&SyntaxKind::BogusLessVariable)
        );
        assert!(node_kinds(&missing_scss_blocks.syntax()).contains(&SyntaxKind::BogusScssMixin));
        assert!(node_kinds(&missing_scss_blocks.syntax()).contains(&SyntaxKind::BogusScssFunction));
        assert!(node_kinds(&missing_scss_blocks.syntax()).contains(&SyntaxKind::BogusScssControl));
        assert!(
            node_kinds(&inconsistent_sass_indentation.syntax())
                .contains(&SyntaxKind::BogusSassIndentation)
        );
        assert!(
            node_kinds(&missing_less_mixin_block.syntax()).contains(&SyntaxKind::BogusLessMixin)
        );
        assert!(
            node_kinds(&missing_less_guard_condition.syntax())
                .contains(&SyntaxKind::BogusLessGuard)
        );
    }

    #[test]
    fn populates_every_declared_bogus_kind_in_recovery_corpus() {
        let mut actual = BTreeSet::new();
        let mut collect = |result: ParseResult| {
            actual.extend(
                node_kinds(&result.syntax())
                    .into_iter()
                    .filter(|kind| kind.is_bogus()),
            );
        };

        collect(parse("{ color: red; }", StyleDialect::Css));
        collect(parse(". { color: red; }", StyleDialect::Css));
        collect(parse("%bad { color: red; }", StyleDialect::Css));
        collect(parse(".a > { color: red; }", StyleDialect::Css));
        collect(parse(".a { : red; width: ?; }", StyleDialect::Css));
        collect(parse(
            ".a { width: ; height: calc(1 + ; }",
            StyleDialect::Css,
        ));
        collect(parse(".a { color: [red; }", StyleDialect::Css));
        collect(parse(".a { font-family: system, ; }", StyleDialect::Css));
        collect(parse("@ ;", StyleDialect::Css));
        collect(parse(
            "@unknown (min-width: { color: red; }",
            StyleDialect::Css,
        ));
        collect(parse(
            "@media screen, (min-width: { .a { color: red; } }",
            StyleDialect::Css,
        ));
        collect(parse(
            "@supports (display: { .a { color: red; } }",
            StyleDialect::Css,
        ));
        collect(parse(
            "@container (inline-size > { .a { color: red; } }",
            StyleDialect::Css,
        ));
        collect(parse("@layer ;", StyleDialect::Css));
        collect(parse(
            "@scope (.a { .b { color: red; } }",
            StyleDialect::Css,
        ));
        collect(parse(
            "@keyframes fade { from opacity: 0; }",
            StyleDialect::Css,
        ));
        collect(parse(
            "@value from; .bad { composes: from; } .missing { composes base; }",
            StyleDialect::Scss,
        ));
        collect(parse(
            "@use \"theme\" with ($gap: 1rem; .card { color: red; }",
            StyleDialect::Scss,
        ));
        collect(parse(
            "@mixin card; @function double; @if $x;",
            StyleDialect::Scss,
        ));
        collect(parse("$gap;", StyleDialect::Scss));
        collect(parse(".a { content: \"unterminated\n }", StyleDialect::Css));
        collect(parse(".a { color: #{$tone; }", StyleDialect::Scss));
        collect(parse(
            ".card\n  color: red\n color: blue\n",
            StyleDialect::Sass,
        ));
        collect(parse("@gap;", StyleDialect::Less));
        collect(parse(".theme(@tone);", StyleDialect::Less));
        collect(parse(".theme() when { color: red; }", StyleDialect::Less));
        collect(parse("@detached: { .a { color: red; }", StyleDialect::Less));
        collect(parse("$gap 1rem;", StyleDialect::Scss));
        collect(parse_entry_point(
            "[red",
            StyleDialect::Css,
            ParseEntryPoint::SimpleBlock,
        ));
        collect(parse_entry_point(
            "red, ;",
            StyleDialect::Css,
            ParseEntryPoint::CommaSeparatedComponentValueList,
        ));

        let declared = SyntaxKind::ALL
            .iter()
            .copied()
            .filter(|kind| kind.is_bogus())
            .collect::<BTreeSet<_>>();
        let missing = declared.difference(&actual).copied().collect::<Vec<_>>();

        assert!(missing.is_empty(), "missing bogus kinds: {missing:?}");
    }

    #[test]
    fn parses_css_module_value_and_composes_cst_nodes() {
        let result = parse(
            "@value primary: #fff; @value accent: primary; @value secondary as localSecondary from \"./tokens.module.scss\"; .btn { composes: base utility from \"./base.module.scss\"; }",
            StyleDialect::Scss,
        );
        let kinds = node_kinds(&result.syntax());

        assert!(result.errors().is_empty());
        assert!(kinds.contains(&SyntaxKind::CssModuleExportBlock));
        assert!(kinds.contains(&SyntaxKind::CssModuleImportBlock));
        assert!(kinds.contains(&SyntaxKind::TokenDefinition));
        assert!(kinds.contains(&SyntaxKind::TokenReference));
        assert!(kinds.contains(&SyntaxKind::CssModuleComposesDeclaration));
        assert!(kinds.contains(&SyntaxKind::CssModuleComposesTarget));
        assert!(kinds.contains(&SyntaxKind::CssModuleFromClause));
    }

    #[test]
    fn extracts_css_module_value_style_facts() {
        let facts = collect_style_facts(
            "@value primary: #fff; @value accent: primary; @value secondary as localSecondary from \"./tokens.module.scss\"; .btn { color: accent; }",
            StyleDialect::Css,
        );
        let definitions = facts
            .css_module_values
            .iter()
            .filter(|value| value.kind == ParsedCssModuleValueFactKind::Definition)
            .map(|value| value.name.as_str())
            .collect::<Vec<_>>();
        let references = facts
            .css_module_values
            .iter()
            .filter(|value| value.kind == ParsedCssModuleValueFactKind::Reference)
            .map(|value| value.name.as_str())
            .collect::<Vec<_>>();
        let import_sources = facts
            .css_module_values
            .iter()
            .filter(|value| value.kind == ParsedCssModuleValueFactKind::ImportSource)
            .map(|value| value.name.as_str())
            .collect::<Vec<_>>();

        assert_eq!(facts.css_module_value_count, 6);
        assert_eq!(definitions, vec!["primary", "accent", "localSecondary"]);
        assert_eq!(references, vec!["primary", "secondary"]);
        assert_eq!(import_sources, vec!["./tokens.module.scss"]);
        assert_eq!(facts.css_module_value_import_edge_count, 1);
        assert_eq!(
            facts.css_module_value_import_edges[0].remote_name,
            "secondary"
        );
        assert_eq!(
            facts.css_module_value_import_edges[0].local_name,
            "localSecondary"
        );
        assert_eq!(
            facts.css_module_value_import_edges[0].import_source,
            "./tokens.module.scss"
        );
        assert_eq!(facts.css_module_value_definition_edge_count, 1);
        assert_eq!(
            facts.css_module_value_definition_edges[0].definition_name,
            "accent"
        );
        assert_eq!(
            facts.css_module_value_definition_edges[0].reference_names,
            vec!["primary"]
        );
    }

    #[test]
    fn extracts_css_module_composes_style_facts() {
        let facts = collect_style_facts(
            ".btn { composes: base utility from \"./base.module.scss\"; } .global { composes: reset from global; }",
            StyleDialect::Css,
        );
        let targets = facts
            .css_module_composes
            .iter()
            .filter(|composes| composes.kind == ParsedCssModuleComposesFactKind::Target)
            .map(|composes| composes.name.as_str())
            .collect::<Vec<_>>();
        let import_sources = facts
            .css_module_composes
            .iter()
            .filter(|composes| composes.kind == ParsedCssModuleComposesFactKind::ImportSource)
            .map(|composes| composes.name.as_str())
            .collect::<Vec<_>>();

        assert_eq!(facts.css_module_composes_count, 5);
        assert_eq!(targets, vec!["base", "utility", "reset"]);
        assert_eq!(import_sources, vec!["./base.module.scss", "global"]);
        assert_eq!(facts.css_module_composes_edge_count, 2);
        assert_eq!(
            facts.css_module_composes_edges[0].kind,
            ParsedCssModuleComposesEdgeKind::External
        );
        assert_eq!(
            facts.css_module_composes_edges[0].owner_selector_names,
            vec!["btn"]
        );
        assert_eq!(
            facts.css_module_composes_edges[0].target_names,
            vec!["base", "utility"]
        );
        assert_eq!(
            facts.css_module_composes_edges[0].import_source.as_deref(),
            Some("./base.module.scss")
        );
        assert_eq!(
            facts.css_module_composes_edges[1].kind,
            ParsedCssModuleComposesEdgeKind::Global
        );
        assert_eq!(
            facts.css_module_composes_edges[1].owner_selector_names,
            vec!["global"]
        );
        assert_eq!(
            facts.css_module_composes_edges[1].target_names,
            vec!["reset"]
        );
        assert_eq!(
            facts.css_module_composes_edges[1].import_source.as_deref(),
            Some("global")
        );
    }

    #[test]
    fn parses_icss_import_export_blocks() {
        let result = parse(
            ":export { primary: #fff; } :import(\"./tokens.css\") { imported: primary; } .btn { composes: imported; }",
            StyleDialect::Css,
        );
        let invalid = parse(":import { imported: primary; }", StyleDialect::Css);
        let kinds = node_kinds(&result.syntax());

        assert!(result.errors().is_empty());
        assert!(kinds.contains(&SyntaxKind::CssModuleExportBlock));
        assert!(kinds.contains(&SyntaxKind::CssModuleImportBlock));
        assert!(
            invalid
                .errors()
                .iter()
                .any(|error| error.message == "expected ICSS import source")
        );
    }

    #[test]
    fn extracts_icss_style_facts() {
        let facts = collect_style_facts(
            ":export { primary: #fff; secondary: accent; } :import(\"./tokens.css\") { imported: primary; tone: themeTone; }",
            StyleDialect::Css,
        );
        let export_names = facts
            .icss
            .iter()
            .filter(|icss| icss.kind == ParsedIcssFactKind::ExportName)
            .map(|icss| icss.name.as_str())
            .collect::<Vec<_>>();
        let import_local_names = facts
            .icss
            .iter()
            .filter(|icss| icss.kind == ParsedIcssFactKind::ImportLocalName)
            .map(|icss| icss.name.as_str())
            .collect::<Vec<_>>();
        let import_remote_names = facts
            .icss
            .iter()
            .filter(|icss| icss.kind == ParsedIcssFactKind::ImportRemoteName)
            .map(|icss| icss.name.as_str())
            .collect::<Vec<_>>();
        let import_sources = facts
            .icss
            .iter()
            .filter(|icss| icss.kind == ParsedIcssFactKind::ImportSource)
            .map(|icss| icss.name.as_str())
            .collect::<Vec<_>>();

        assert_eq!(facts.icss_count, 7);
        assert_eq!(export_names, vec!["primary", "secondary"]);
        assert_eq!(import_local_names, vec!["imported", "tone"]);
        assert_eq!(import_remote_names, vec!["primary", "themeTone"]);
        assert_eq!(import_sources, vec!["./tokens.css"]);
        assert_eq!(facts.icss_import_edge_count, 2);
        assert_eq!(facts.icss_import_edges[0].local_name, "imported");
        assert_eq!(facts.icss_import_edges[0].remote_name, "primary");
        assert_eq!(facts.icss_import_edges[0].import_source, "./tokens.css");
        assert_eq!(facts.icss_import_edges[1].local_name, "tone");
        assert_eq!(facts.icss_import_edges[1].remote_name, "themeTone");
        assert_eq!(facts.icss_import_edges[1].import_source, "./tokens.css");
        assert_eq!(facts.icss_export_edge_count, 1);
        assert_eq!(facts.icss_export_edges[0].export_name, "secondary");
        assert_eq!(facts.icss_export_edges[0].reference_names, vec!["accent"]);
    }

    #[test]
    fn recovers_css_module_value_and_composes_bogus_nodes() {
        let result = parse(
            "@value from; .bad { composes: from; } .missing { composes base; } .invalid { composes: base from 123; } @value bad as alias from 123; .multi { composes: a from \"./a.css\", b from \"./b.css\"; }",
            StyleDialect::Scss,
        );
        let kinds = node_kinds(&result.syntax());
        let invalid_from_source_count = result
            .errors()
            .iter()
            .filter(|error| error.message == "invalid CSS Modules from-clause source")
            .count();
        let multiple_from_count = result
            .errors()
            .iter()
            .filter(|error| error.message == "multiple composes from clauses are not allowed")
            .count();

        assert!(kinds.contains(&SyntaxKind::BogusCssModuleBlock));
        assert!(kinds.contains(&SyntaxKind::BogusFromClause));
        assert!(kinds.contains(&SyntaxKind::BogusComposesTarget));
        assert!(kinds.contains(&SyntaxKind::BogusComposesDeclaration));
        assert_eq!(invalid_from_source_count, 2);
        assert_eq!(multiple_from_count, 1);
    }

    #[test]
    fn validates_composes_outside_css_module_global_scope() {
        let invalid = parse(
            ":global(.reset) { composes: base; } :global { .utility { composes: base; } } :local(.ok) { composes: base; }",
            StyleDialect::Css,
        );
        let outer_local = parse(
            ":local { :global(.ok) { composes: base; } }",
            StyleDialect::Css,
        );
        let mixed_local_global = parse(".foo :global(.bar) { composes: base; }", StyleDialect::Css);
        let global_composes_count = invalid
            .errors()
            .iter()
            .filter(|error| error.message == "composes is not allowed inside :global scope")
            .count();

        assert_eq!(global_composes_count, 2);
        assert!(
            !outer_local
                .errors()
                .iter()
                .any(|error| error.message == "composes is not allowed inside :global scope")
        );
        assert!(
            !mixed_local_global
                .errors()
                .iter()
                .any(|error| error.message == "composes is not allowed inside :global scope")
        );
    }

    #[test]
    fn parses_registered_group_at_rule_blocks() {
        let result = parse(
            "@media screen and (min-width: 40rem) { .card { color: red; } }",
            StyleDialect::Css,
        );
        let kinds = node_kinds(&result.syntax());

        assert!(result.errors().is_empty());
        assert!(kinds.contains(&SyntaxKind::AtRule));
        assert!(kinds.contains(&SyntaxKind::MediaRule));
        assert!(kinds.contains(&SyntaxKind::RuleList));
        assert!(kinds.contains(&SyntaxKind::Rule));
        assert!(kinds.contains(&SyntaxKind::ClassSelector));
    }

    #[test]
    fn parses_conditional_at_rule_preludes() {
        let result = parse(
            "@media screen and (min-width: 40rem), print { .card { color: red; } } @supports (display: grid) { .grid { display: grid; } } @container card (inline-size > 40rem) { .item { color: blue; } }",
            StyleDialect::Css,
        );
        let kinds = node_kinds(&result.syntax());

        assert!(result.errors().is_empty());
        assert!(kinds.contains(&SyntaxKind::MediaQueryList));
        assert_eq!(
            kinds
                .iter()
                .filter(|kind| **kind == SyntaxKind::MediaQuery)
                .count(),
            2
        );
        assert!(kinds.contains(&SyntaxKind::MediaFeature));
        assert!(kinds.contains(&SyntaxKind::SupportsCondition));
        assert!(kinds.contains(&SyntaxKind::ContainerCondition));
    }

    #[test]
    fn validates_media_query_list_preludes() {
        let result = parse(
            "@media { .a { color: red; } } @media , screen { .b { color: blue; } } @media screen, { .c { color: green; } } @media 1 { .d { color: black; } } @media screen and (min-width: 40rem), print { .e { color: white; } }",
            StyleDialect::Css,
        );
        let kinds = node_kinds(&result.syntax());
        let invalid_media_errors = result
            .errors()
            .iter()
            .filter(|error| error.message == "invalid @media prelude")
            .count();
        let bogus_media_queries = kinds
            .iter()
            .filter(|kind| **kind == SyntaxKind::BogusMediaQuery)
            .count();

        assert_eq!(invalid_media_errors, 4);
        assert_eq!(bogus_media_queries, 4);
        assert!(kinds.contains(&SyntaxKind::MediaQuery));
    }

    #[test]
    fn validates_supports_rule_preludes() {
        let result = parse(
            "@supports { .a { color: red; } } @supports display: grid { .b { color: blue; } } @supports not { .c { color: green; } } @supports (display: grid) { .d { color: black; } } @supports selector(:has(*)) { .e { color: white; } }",
            StyleDialect::Css,
        );
        let kinds = node_kinds(&result.syntax());
        let invalid_supports_errors = result
            .errors()
            .iter()
            .filter(|error| error.message == "invalid @supports prelude")
            .count();
        let bogus_supports_conditions = kinds
            .iter()
            .filter(|kind| **kind == SyntaxKind::BogusSupportsCondition)
            .count();

        assert_eq!(invalid_supports_errors, 3);
        assert_eq!(bogus_supports_conditions, 3);
        assert!(kinds.contains(&SyntaxKind::SupportsCondition));
    }

    #[test]
    fn validates_container_rule_preludes() {
        let result = parse(
            "@container { .a { color: red; } } @container card { .b { color: blue; } } @container 1 (width > 0) { .c { color: green; } } @container style(--theme: dark) { .d { color: white; } } @container card style(--theme: dark) { .e { color: black; } }",
            StyleDialect::Css,
        );
        let kinds = node_kinds(&result.syntax());
        let invalid_container_errors = result
            .errors()
            .iter()
            .filter(|error| error.message == "invalid @container prelude")
            .count();
        let bogus_container_conditions = kinds
            .iter()
            .filter(|kind| **kind == SyntaxKind::BogusContainerCondition)
            .count();

        assert_eq!(invalid_container_errors, 3);
        assert_eq!(bogus_container_conditions, 3);
        assert!(kinds.contains(&SyntaxKind::ContainerCondition));
    }

    #[test]
    fn classifies_css_at_rules_case_insensitively() {
        let source = "@MEDIA (width >= 1px) { .card { color: red; } } @KEYFRAMES fade { from { opacity: 0; } to { opacity: 1; } }";
        let result = parse(source, StyleDialect::Css);
        let facts = collect_style_facts(source, StyleDialect::Css);
        let kinds = node_kinds(&result.syntax());
        let at_rule_names: Vec<&str> = facts
            .at_rules
            .iter()
            .map(|at_rule| at_rule.name.as_str())
            .collect();

        assert!(result.errors().is_empty());
        assert!(kinds.contains(&SyntaxKind::MediaRule));
        assert!(kinds.contains(&SyntaxKind::KeyframesRule));
        assert!(
            facts
                .selectors
                .iter()
                .any(|selector| selector.name == "card")
        );
        assert_eq!(at_rule_names, vec!["@media", "@keyframes"]);
    }

    #[test]
    fn parses_import_layer_supports_media_prelude() {
        let result = parse(
            "@import url(\"theme.css\") layer(app.theme) supports(display: grid) screen and (min-width: 40rem);",
            StyleDialect::Css,
        );
        let less = parse(
            "@import (reference) \"theme.less\" screen and (min-width: 40rem);",
            StyleDialect::Less,
        );
        let kinds = node_kinds(&result.syntax());
        let less_kinds = node_kinds(&less.syntax());

        assert!(result.errors().is_empty());
        assert!(less.errors().is_empty());
        assert!(kinds.contains(&SyntaxKind::ImportRule));
        assert!(kinds.contains(&SyntaxKind::UrlValue));
        assert!(kinds.contains(&SyntaxKind::LayerName));
        assert!(kinds.contains(&SyntaxKind::SupportsCondition));
        assert!(kinds.contains(&SyntaxKind::MediaQueryList));
        assert!(kinds.contains(&SyntaxKind::MediaFeature));
        assert!(less_kinds.contains(&SyntaxKind::ImportRule));
        assert!(less_kinds.contains(&SyntaxKind::AtRulePrelude));
        assert!(less_kinds.contains(&SyntaxKind::MediaQueryList));
    }

    #[test]
    fn validates_import_sources() {
        let result = parse(
            "@import ; @import layer(app); @import 1; @import url(foo bar);",
            StyleDialect::Css,
        );
        let kinds = node_kinds(&result.syntax());
        let invalid_import_errors = result
            .errors()
            .iter()
            .filter(|error| error.message == "invalid @import source")
            .count();
        let bogus_preludes = kinds
            .iter()
            .filter(|kind| **kind == SyntaxKind::BogusAtRulePrelude)
            .count();

        assert_eq!(invalid_import_errors, 4);
        assert_eq!(bogus_preludes, 4);
    }

    #[test]
    fn validates_import_optional_tails() {
        let result = parse(
            "@import \"a.css\" layer(); @import \"b.css\" layer(1); @import \"c.css\" supports(); @import \"d.css\" supports screen; @import \"ok.css\" layer(app.theme) supports(display: grid) screen;",
            StyleDialect::Css,
        );
        let kinds = node_kinds(&result.syntax());
        let invalid_layer_tail_errors = result
            .errors()
            .iter()
            .filter(|error| error.message == "invalid @import layer tail")
            .count();
        let invalid_supports_tail_errors = result
            .errors()
            .iter()
            .filter(|error| error.message == "invalid @import supports tail")
            .count();
        let bogus_layer_names = kinds
            .iter()
            .filter(|kind| **kind == SyntaxKind::BogusLayerName)
            .count();
        let bogus_supports_conditions = kinds
            .iter()
            .filter(|kind| **kind == SyntaxKind::BogusSupportsCondition)
            .count();

        assert_eq!(invalid_layer_tail_errors, 2);
        assert_eq!(invalid_supports_tail_errors, 2);
        assert_eq!(bogus_layer_names, 2);
        assert_eq!(bogus_supports_conditions, 2);
        assert!(kinds.contains(&SyntaxKind::LayerName));
        assert!(kinds.contains(&SyntaxKind::SupportsCondition));
        assert!(kinds.contains(&SyntaxKind::MediaQueryList));
    }

    #[test]
    fn parses_layer_and_scope_preludes() {
        let result = parse(
            "@layer reset, app.ui; @layer components { .card { color: red; } } @layer { .anon { color: blue; } } @scope (.card) to (.card-content) { .title { color: red; } }",
            StyleDialect::Css,
        );
        let kinds = node_kinds(&result.syntax());

        assert!(result.errors().is_empty());
        assert!(kinds.contains(&SyntaxKind::LayerRule));
        assert!(kinds.contains(&SyntaxKind::LayerName));
        assert!(kinds.contains(&SyntaxKind::ScopeRule));
        assert!(kinds.contains(&SyntaxKind::ScopeRange));
        assert!(kinds.contains(&SyntaxKind::RuleList));
    }

    #[test]
    fn validates_layer_rule_preludes() {
        let result = parse(
            "@layer , reset; @layer app.; @layer 1; @layer ok.name;",
            StyleDialect::Css,
        );
        let kinds = node_kinds(&result.syntax());
        let invalid_layer_errors = result
            .errors()
            .iter()
            .filter(|error| error.message == "invalid @layer prelude")
            .count();
        let bogus_layer_names = kinds
            .iter()
            .filter(|kind| **kind == SyntaxKind::BogusLayerName)
            .count();

        assert_eq!(invalid_layer_errors, 3);
        assert_eq!(bogus_layer_names, 3);
        assert!(kinds.contains(&SyntaxKind::LayerName));
    }

    #[test]
    fn validates_scope_rule_preludes() {
        let result = parse(
            "@scope { .a { color: red; } } @scope .a { .b { color: blue; } } @scope (.a) to { .c { color: green; } } @scope (.a) to (.b) { .d { color: black; } }",
            StyleDialect::Css,
        );
        let kinds = node_kinds(&result.syntax());
        let invalid_scope_errors = result
            .errors()
            .iter()
            .filter(|error| error.message == "invalid @scope prelude")
            .count();
        let bogus_scope_ranges = kinds
            .iter()
            .filter(|kind| **kind == SyntaxKind::BogusScopeRange)
            .count();

        assert_eq!(invalid_scope_errors, 3);
        assert_eq!(bogus_scope_ranges, 3);
        assert!(kinds.contains(&SyntaxKind::ScopeRange));
    }

    #[test]
    fn validates_page_rule_preludes() {
        let result = parse(
            "@page { margin: 1cm; } @page :first { margin: 2cm; } @page chapter:left, appendix:right { margin: 3cm; } @page 1 { margin: 4cm; } @page chapter, { margin: 5cm; } @page chapter first { margin: 6cm; }",
            StyleDialect::Css,
        );
        let kinds = node_kinds(&result.syntax());
        let invalid_page_errors = result
            .errors()
            .iter()
            .filter(|error| error.message == "invalid @page prelude")
            .count();
        let bogus_preludes = kinds
            .iter()
            .filter(|kind| **kind == SyntaxKind::BogusAtRulePrelude)
            .count();

        assert_eq!(invalid_page_errors, 3);
        assert_eq!(bogus_preludes, 3);
        assert!(kinds.contains(&SyntaxKind::PageRule));
        assert!(kinds.contains(&SyntaxKind::AtRulePrelude));
    }

    #[test]
    fn parses_registered_keyframes_and_declaration_at_rules() {
        let keyframes = parse(
            "@keyframes fade { from { opacity: 0; } to { opacity: 1; } }",
            StyleDialect::Css,
        );
        let font_face = parse(
            "@font-face { font-family: \"Demo\"; src: url(demo.woff2); }",
            StyleDialect::Css,
        );
        let page_margin = parse(
            "@page :first { margin: 1cm; @top-left { content: \"A\"; } @bottom-center { content: counter(page); } }",
            StyleDialect::Css,
        );
        let conditional_l5 = parse(
            "@when media(width >= 1px) { .a { color: red; } } @else { .b { color: blue; } }",
            StyleDialect::Css,
        );
        let modern_declaration_rules = parse(
            "@counter-style thumbs { system: cyclic; symbols: \"yes\"; suffix: \" \"; } @font-palette-values --brand { font-family: Demo; base-palette: 1; } @color-profile --display-p3 { src: url(p3.icc); } @position-try --popover { inset-area: top; }",
            StyleDialect::Css,
        );
        let font_feature_values = parse(
            "@font-feature-values Demo { @stylistic { nice: 1; } @styleset { alt: 2; } @character-variant { nice: 3 4; } @swash { fancy: 1; } @ornaments { leaf: 1; } @annotation { circled: 1; } @historical-forms { old: 1; } } @view-transition { navigation: auto; }",
            StyleDialect::Css,
        );
        let less_css_at_rules = parse(
            "@font-feature-values Demo { @styleset { alt: 2; } } @view-transition { navigation: auto; }",
            StyleDialect::Less,
        );
        let nesting_and_custom_media = parse(
            ".card { @nest &__icon { color: red; &--active { color: blue; } } } @custom-media --narrow (width < 40rem);",
            StyleDialect::Css,
        );
        let keyframe_kinds = node_kinds(&keyframes.syntax());
        let font_face_kinds = node_kinds(&font_face.syntax());
        let page_margin_kinds = node_kinds(&page_margin.syntax());
        let conditional_l5_kinds = node_kinds(&conditional_l5.syntax());
        let modern_declaration_kinds = node_kinds(&modern_declaration_rules.syntax());
        let font_feature_value_kinds = node_kinds(&font_feature_values.syntax());
        let less_css_at_rule_kinds = node_kinds(&less_css_at_rules.syntax());
        let nesting_and_custom_media_kinds = node_kinds(&nesting_and_custom_media.syntax());

        assert!(keyframes.errors().is_empty());
        assert!(font_face.errors().is_empty());
        assert!(page_margin.errors().is_empty());
        assert!(conditional_l5.errors().is_empty());
        assert!(modern_declaration_rules.errors().is_empty());
        assert!(font_feature_values.errors().is_empty());
        assert!(less_css_at_rules.errors().is_empty());
        assert!(nesting_and_custom_media.errors().is_empty());
        assert!(keyframe_kinds.contains(&SyntaxKind::KeyframesRule));
        assert!(keyframe_kinds.contains(&SyntaxKind::AtRulePrelude));
        assert!(keyframe_kinds.contains(&SyntaxKind::KeyframeBlock));
        assert!(font_face_kinds.contains(&SyntaxKind::FontFaceRule));
        assert!(font_face_kinds.contains(&SyntaxKind::DeclarationList));
        assert!(page_margin_kinds.contains(&SyntaxKind::PageRule));
        assert!(page_margin_kinds.contains(&SyntaxKind::PageMarginRule));
        assert!(conditional_l5_kinds.contains(&SyntaxKind::WhenRule));
        assert!(conditional_l5_kinds.contains(&SyntaxKind::ElseRule));
        assert!(conditional_l5_kinds.contains(&SyntaxKind::RuleList));
        assert!(modern_declaration_kinds.contains(&SyntaxKind::CounterStyleRule));
        assert!(modern_declaration_kinds.contains(&SyntaxKind::FontPaletteValuesRule));
        assert!(modern_declaration_kinds.contains(&SyntaxKind::ColorProfileRule));
        assert!(modern_declaration_kinds.contains(&SyntaxKind::PositionTryRule));
        assert!(modern_declaration_kinds.contains(&SyntaxKind::DeclarationList));
        assert!(font_feature_value_kinds.contains(&SyntaxKind::FontFeatureValuesRule));
        assert!(font_feature_value_kinds.contains(&SyntaxKind::FontFeatureValuesStylisticRule));
        assert!(font_feature_value_kinds.contains(&SyntaxKind::FontFeatureValuesStylesetRule));
        assert!(
            font_feature_value_kinds.contains(&SyntaxKind::FontFeatureValuesCharacterVariantRule)
        );
        assert!(font_feature_value_kinds.contains(&SyntaxKind::FontFeatureValuesSwashRule));
        assert!(font_feature_value_kinds.contains(&SyntaxKind::FontFeatureValuesOrnamentsRule));
        assert!(font_feature_value_kinds.contains(&SyntaxKind::FontFeatureValuesAnnotationRule));
        assert!(
            font_feature_value_kinds.contains(&SyntaxKind::FontFeatureValuesHistoricalFormsRule)
        );
        assert!(font_feature_value_kinds.contains(&SyntaxKind::ViewTransitionRule));
        assert!(less_css_at_rule_kinds.contains(&SyntaxKind::FontFeatureValuesRule));
        assert!(less_css_at_rule_kinds.contains(&SyntaxKind::FontFeatureValuesStylesetRule));
        assert!(less_css_at_rule_kinds.contains(&SyntaxKind::ViewTransitionRule));
        assert!(nesting_and_custom_media_kinds.contains(&SyntaxKind::NestRule));
        assert!(nesting_and_custom_media_kinds.contains(&SyntaxKind::CustomMediaRule));
        assert!(nesting_and_custom_media_kinds.contains(&SyntaxKind::DeclarationList));
    }

    #[test]
    fn validates_property_at_rule_names() {
        let valid = parse(
            "@property --accent { syntax: \"<color>\"; inherits: false; initial-value: red; }",
            StyleDialect::Css,
        );
        let dynamic = parse(
            "@property #{$name} { syntax: \"<color>\"; inherits: false; initial-value: red; }",
            StyleDialect::Scss,
        );
        let invalid = parse(
            "@property accent { syntax: \"<color>\"; inherits: false; initial-value: red; }",
            StyleDialect::Css,
        );
        let invalid_property_name_count = invalid
            .errors()
            .iter()
            .filter(|error| error.message == "invalid @property name")
            .count();

        assert!(valid.errors().is_empty());
        assert!(dynamic.errors().is_empty());
        assert_eq!(invalid_property_name_count, 1);
    }

    #[test]
    fn validates_named_declaration_at_rule_preludes() {
        let valid = parse(
            "@counter-style thumbs { system: cyclic; symbols: \"yes\"; } @font-palette-values --brand { font-family: Demo; } @color-profile --display-p3 { src: url(p3.icc); } @position-try --popover { inset-area: top; } @custom-media --narrow (width < 40rem);",
            StyleDialect::Css,
        );
        let dynamic = parse(
            "@counter-style #{$style} { system: cyclic; symbols: \"yes\"; } @font-palette-values #{$palette} { font-family: Demo; } @custom-media #{$query} (width < 40rem);",
            StyleDialect::Scss,
        );
        let invalid = parse(
            "@counter-style --bad { system: cyclic; } @font-palette-values brand { font-family: Demo; } @color-profile display-p3 { src: url(p3.icc); } @position-try popover { inset-area: top; } @custom-media narrow (width < 40rem); @custom-media --missing;",
            StyleDialect::Css,
        );
        let custom_property_name_errors = invalid
            .errors()
            .iter()
            .filter(|error| error.message == "invalid at-rule custom property name")
            .count();
        let custom_media_prelude_errors = invalid
            .errors()
            .iter()
            .filter(|error| error.message == "invalid @custom-media prelude")
            .count();
        let counter_style_name_errors = invalid
            .errors()
            .iter()
            .filter(|error| error.message == "invalid @counter-style name")
            .count();

        assert!(valid.errors().is_empty());
        assert!(dynamic.errors().is_empty());
        assert_eq!(custom_property_name_errors, 3);
        assert_eq!(custom_media_prelude_errors, 2);
        assert_eq!(counter_style_name_errors, 1);
    }

    #[test]
    fn validates_charset_and_namespace_at_rule_preludes() {
        let valid = parse(
            "@charset \"UTF-8\"; @namespace \"http://www.w3.org/1999/xhtml\"; @namespace svg url(\"http://www.w3.org/2000/svg\"); @namespace math url(http://www.w3.org/1998/Math/MathML);",
            StyleDialect::Css,
        );
        let dynamic = parse(
            "@namespace #{$url}; @namespace svg #{$url};",
            StyleDialect::Scss,
        );
        let invalid = parse("@charset UTF-8; @namespace svg;", StyleDialect::Css);
        let charset_errors = invalid
            .errors()
            .iter()
            .filter(|error| error.message == "invalid @charset prelude")
            .count();
        let namespace_errors = invalid
            .errors()
            .iter()
            .filter(|error| error.message == "invalid @namespace prelude")
            .count();

        assert!(valid.errors().is_empty());
        assert!(dynamic.errors().is_empty());
        assert_eq!(charset_errors, 1);
        assert_eq!(namespace_errors, 1);
    }

    #[test]
    fn validates_keyframes_at_rule_names() {
        let valid = parse(
            "@keyframes fade { from { opacity: 0; } } @keyframes \"slide\" { to { opacity: 1; } }",
            StyleDialect::Css,
        );
        let dynamic = parse(
            "@keyframes #{$animation-name} { from { opacity: 0; } }",
            StyleDialect::Scss,
        );
        let invalid = parse(
            "@keyframes 50% { from { opacity: 0; } } @keyframes fade extra { to { opacity: 1; } }",
            StyleDialect::Css,
        );
        let invalid_name_errors = invalid
            .errors()
            .iter()
            .filter(|error| error.message == "invalid @keyframes name")
            .count();

        assert!(valid.errors().is_empty());
        assert!(dynamic.errors().is_empty());
        assert_eq!(invalid_name_errors, 2);
    }

    #[test]
    fn validates_keyframe_selector_lists() {
        let valid = parse(
            "@keyframes fade { from { opacity: 0; } 50%, 75% { opacity: .5; } to { opacity: 1; } }",
            StyleDialect::Css,
        );
        let dynamic = parse(
            "@keyframes fade { #{$step} { opacity: .5; } }",
            StyleDialect::Scss,
        );
        let invalid = parse(
            "@keyframes fade { middle { opacity: .5; } 120px { opacity: 1; } 50%, { opacity: .8; } }",
            StyleDialect::Css,
        );
        let invalid_selector_errors = invalid
            .errors()
            .iter()
            .filter(|error| error.message == "invalid keyframe selector")
            .count();

        assert!(valid.errors().is_empty());
        assert!(dynamic.errors().is_empty());
        assert_eq!(invalid_selector_errors, 3);
    }

    #[test]
    fn validates_empty_block_at_rule_preludes() {
        let valid = parse(
            "@font-face { font-family: Demo; } @starting-style { .card { opacity: 0; } } @view-transition { navigation: auto; } @page { @top-left { content: \"A\"; } } @font-feature-values Demo { @styleset { alt: 2; } }",
            StyleDialect::Css,
        );
        let invalid = parse(
            "@font-face Demo { font-family: Demo; } @starting-style demo { .card { opacity: 0; } } @view-transition demo { navigation: auto; } @page { @top-left header { content: \"A\"; } } @font-feature-values Demo { @styleset alt { alt: 2; } }",
            StyleDialect::Css,
        );
        let unexpected_prelude_errors = invalid
            .errors()
            .iter()
            .filter(|error| error.message == "unexpected at-rule prelude")
            .count();

        assert!(valid.errors().is_empty());
        assert_eq!(unexpected_prelude_errors, 5);
    }

    #[test]
    fn validates_font_feature_values_preludes() {
        let valid = parse(
            "@font-feature-values Demo, \"Brand Font\" { @styleset { alt: 2; } }",
            StyleDialect::Css,
        );
        let dynamic = parse(
            "@font-feature-values #{$family} { @styleset { alt: 2; } }",
            StyleDialect::Scss,
        );
        let invalid = parse(
            "@font-feature-values { @styleset { alt: 2; } } @font-feature-values 123 { @styleset { alt: 2; } }",
            StyleDialect::Css,
        );
        let invalid_family_name_errors = invalid
            .errors()
            .iter()
            .filter(|error| error.message == "invalid @font-feature-values family name")
            .count();

        assert!(valid.errors().is_empty());
        assert!(dynamic.errors().is_empty());
        assert_eq!(invalid_family_name_errors, 2);
    }

    #[test]
    fn classifies_initial_scss_at_rule_nodes() {
        let module_rules = parse(
            "@use \"sass:map\"; @forward \"tokens\";",
            StyleDialect::Scss,
        );
        let mixin_rule = parse("@mixin card($gap) { padding: $gap; }", StyleDialect::Scss);
        let module_kinds = node_kinds(&module_rules.syntax());
        let mixin_kinds = node_kinds(&mixin_rule.syntax());

        assert!(module_rules.errors().is_empty());
        assert!(mixin_rule.errors().is_empty());
        assert!(module_kinds.contains(&SyntaxKind::ScssUseRule));
        assert!(module_kinds.contains(&SyntaxKind::ScssForwardRule));
        assert!(mixin_kinds.contains(&SyntaxKind::ScssMixinDeclaration));
    }

    #[test]
    fn parses_scss_module_config_preludes() {
        let result = parse(
            "@use \"theme\" as * with ($gap: 1rem, $enabled: true); @forward \"tokens\" as token-* show $color, mixin with ($color: red);",
            StyleDialect::Scss,
        );
        let kinds = node_kinds(&result.syntax());
        let config_count = kinds
            .iter()
            .filter(|kind| **kind == SyntaxKind::ScssModuleConfig)
            .count();

        assert!(result.errors().is_empty());
        assert!(kinds.contains(&SyntaxKind::ScssUseRule));
        assert!(kinds.contains(&SyntaxKind::ScssForwardRule));
        assert_eq!(config_count, 2);
    }

    #[test]
    fn validates_scss_module_prelude_clauses() {
        let invalid = parse(
            "@use as *; @use \"theme\" as ; @use \"theme\" show foo; @forward \"tokens\" hide ; @forward \"tokens\" with $gap;",
            StyleDialect::Scss,
        );

        assert_eq!(
            invalid
                .errors()
                .iter()
                .filter(|error| error.message == "expected SCSS module source")
                .count(),
            1
        );
        assert_eq!(
            invalid
                .errors()
                .iter()
                .filter(|error| error.message == "expected SCSS module namespace")
                .count(),
            1
        );
        assert_eq!(
            invalid
                .errors()
                .iter()
                .filter(|error| error.message == "unexpected SCSS module visibility clause")
                .count(),
            1
        );
        assert_eq!(
            invalid
                .errors()
                .iter()
                .filter(|error| error.message == "expected SCSS module visibility name")
                .count(),
            1
        );
        assert_eq!(
            invalid
                .errors()
                .iter()
                .filter(|error| error.message == "expected SCSS module configuration")
                .count(),
            1
        );
    }

    #[test]
    fn recovers_unclosed_scss_module_config_as_bogus() {
        let result = parse(
            "@use \"theme\" with ($gap: 1rem; .card { color: red; }",
            StyleDialect::Scss,
        );
        let kinds = node_kinds(&result.syntax());

        assert!(
            result
                .errors()
                .iter()
                .any(|error| error.message == "unterminated parenthesized prelude")
        );
        assert!(kinds.contains(&SyntaxKind::BogusScssModuleConfig));
        assert!(!kinds.contains(&SyntaxKind::ScssModuleConfig));
    }

    #[test]
    fn parses_scss_placeholder_selectors_and_extend_refs() {
        let result = parse(
            "%button { color: red; } .primary { @extend %button; }",
            StyleDialect::Scss,
        );
        let kinds = node_kinds(&result.syntax());

        assert!(result.errors().is_empty());
        assert!(kinds.contains(&SyntaxKind::ScssPlaceholderSelector));
        assert!(kinds.contains(&SyntaxKind::ScssExtendRule));
        assert!(token_kinds(&result.syntax()).contains(&SyntaxKind::ScssPlaceholder));
    }

    #[test]
    fn parses_structured_scss_at_rule_bodies() {
        let result = parse(
            "@mixin card($gap) { .item { gap: $gap; } } @function double($x) { @return $x * 2; } @if $enabled { .on { color: green; } } @for $i from 1 through 3 { .n { order: $i; } } @each $k, $v in $map { .e { color: $v; } } @while $enabled { .w { color: red; } }",
            StyleDialect::Scss,
        );
        let kinds = node_kinds(&result.syntax());

        assert!(result.errors().is_empty());
        assert!(kinds.contains(&SyntaxKind::ScssMixinDeclaration));
        assert!(kinds.contains(&SyntaxKind::ScssFunctionDeclaration));
        assert!(kinds.contains(&SyntaxKind::ScssReturnRule));
        assert!(kinds.contains(&SyntaxKind::ScssControlIf));
        assert!(kinds.contains(&SyntaxKind::ScssControlFor));
        assert!(kinds.contains(&SyntaxKind::ScssControlEach));
        assert!(kinds.contains(&SyntaxKind::ScssControlWhile));
        assert!(kinds.contains(&SyntaxKind::DeclarationList));
        assert!(kinds.contains(&SyntaxKind::Rule));
        assert!(kinds.contains(&SyntaxKind::ClassSelector));
        assert!(kinds.contains(&SyntaxKind::ScssVariableReference));
    }

    #[test]
    fn validates_scss_control_preludes() {
        let invalid = parse(
            "@if { .a { color: red; } } @while { .b { color: red; } } @for i from 1 through 3 { .c { color: red; } } @for $i from 1 { .d { color: red; } } @each item of $items { .e { color: red; } }",
            StyleDialect::Scss,
        );
        let invalid_control_prelude_count = invalid
            .errors()
            .iter()
            .filter(|error| error.message == "invalid SCSS control prelude")
            .count();

        assert_eq!(invalid_control_prelude_count, 5);
    }

    #[test]
    fn extracts_scss_control_block_style_facts() {
        let facts = collect_style_facts(
            "@if $enabled { .on { color: green; } } @for $i from 1 through 3 { .n { order: $i; } } @each $k, $v in $map { .e { color: $v; } } @while $enabled { .w { color: red; } }",
            StyleDialect::Scss,
        );
        let class_names = facts
            .selectors
            .iter()
            .filter(|selector| selector.kind == ParsedSelectorFactKind::Class)
            .map(|selector| selector.name.as_str())
            .collect::<Vec<_>>();

        assert_eq!(class_names, vec!["on", "n", "e", "w"]);
    }

    #[test]
    fn extracts_scss_include_content_block_style_facts() {
        let source =
            ".card { @include interactive($tone) using ($state) { &--active { color: red; } } }";
        let parsed = parse(source, StyleDialect::Scss);
        let facts = collect_style_facts(source, StyleDialect::Scss);
        let class_names = facts
            .selectors
            .iter()
            .filter(|selector| selector.kind == ParsedSelectorFactKind::Class)
            .map(|selector| selector.name.as_str())
            .collect::<Vec<_>>();

        assert!(parsed.errors().is_empty());
        assert!(node_kinds(&parsed.syntax()).contains(&SyntaxKind::ScssIncludeRule));
        assert_eq!(class_names, vec!["card", "card--active"]);
    }

    #[test]
    fn parses_scss_nested_property_blocks() {
        let result = parse(
            ".card { font: { size: 1rem; weight: 600; } border: 1px solid { color: red; } }",
            StyleDialect::Scss,
        );
        let kinds = node_kinds(&result.syntax());
        let nested_property_count = kinds
            .iter()
            .filter(|kind| **kind == SyntaxKind::ScssNestedProperty)
            .count();

        assert!(result.errors().is_empty());
        assert_eq!(nested_property_count, 2);
        assert!(kinds.contains(&SyntaxKind::DeclarationList));
        assert!(kinds.contains(&SyntaxKind::Value));
        assert!(kinds.contains(&SyntaxKind::DimensionValue));
    }

    #[test]
    fn parses_sass_indented_nested_property_blocks() {
        let result = parse(
            ".card\n  font:\n    size: 1rem\n    weight: 600\n",
            StyleDialect::Sass,
        );
        let kinds = node_kinds(&result.syntax());

        assert!(result.errors().is_empty());
        assert!(kinds.contains(&SyntaxKind::ScssNestedProperty));
        assert!(kinds.contains(&SyntaxKind::SassIndentedBlock));
        assert!(kinds.contains(&SyntaxKind::DeclarationList));
        assert!(kinds.contains(&SyntaxKind::DimensionValue));
    }

    #[test]
    fn parses_scss_utility_at_rules() {
        let result = parse(
            "@mixin slot { @content; } @at-root { .rooted { color: red; } } @warn $message; @debug $message; @error $message;",
            StyleDialect::Scss,
        );
        let kinds = node_kinds(&result.syntax());

        assert!(result.errors().is_empty());
        assert!(kinds.contains(&SyntaxKind::ScssContentRule));
        assert!(kinds.contains(&SyntaxKind::ScssAtRootRule));
        assert!(kinds.contains(&SyntaxKind::ScssWarnRule));
        assert!(kinds.contains(&SyntaxKind::ScssDebugRule));
        assert!(kinds.contains(&SyntaxKind::ScssErrorRule));
        assert!(kinds.contains(&SyntaxKind::Rule));
    }

    #[test]
    fn structures_css_value_function_calls() {
        let result = parse(".a { width: calc(var(--gap) + 1rem); }", StyleDialect::Css);
        let kinds = node_kinds(&result.syntax());

        assert!(result.errors().is_empty());
        assert!(kinds.contains(&SyntaxKind::Value));
        assert!(kinds.contains(&SyntaxKind::FunctionCall));
        assert!(kinds.contains(&SyntaxKind::FunctionArguments));
        assert!(kinds.contains(&SyntaxKind::CalcFunction));
        assert!(kinds.contains(&SyntaxKind::VarFunction));
        assert!(kinds.contains(&SyntaxKind::BinaryExpression));
    }

    #[test]
    fn structures_modern_css_value_functions() {
        let result = parse(
            ".a { color: color-mix(in oklch, var(--brand), white 20%); accent-color: device-cmyk(0 1 1 0); width: clamp(1rem, 2vw, 3rem); content: attr(data-label string, \"x\"); padding: env(safe-area-inset-top); background-image: linear-gradient(red, blue); transform: translateX(1rem) rotate(10deg); filter: blur(2px) brightness(1.1); image-set: image-set(url(a.png) 1x); offset-path: path(\"M0,0 L1,1\"); }",
            StyleDialect::Css,
        );
        let kinds = node_kinds(&result.syntax());

        assert!(result.errors().is_empty());
        assert!(kinds.contains(&SyntaxKind::ColorValue));
        assert!(kinds.contains(&SyntaxKind::MathFunction));
        assert!(kinds.contains(&SyntaxKind::AttrFunction));
        assert!(kinds.contains(&SyntaxKind::EnvFunction));
        assert!(kinds.contains(&SyntaxKind::VarFunction));
        assert!(kinds.contains(&SyntaxKind::GradientFunction));
        assert!(kinds.contains(&SyntaxKind::TransformFunction));
        assert!(kinds.contains(&SyntaxKind::FilterFunction));
        assert!(kinds.contains(&SyntaxKind::ImageFunction));
        assert!(kinds.contains(&SyntaxKind::ShapeFunction));
    }

    #[test]
    fn validates_color_function_micro_grammars() {
        let valid = parse(
            ".a { color: color-mix(in srgb, red, blue 30%); background: light-dark(white, black); border-color: contrast-color(red); }",
            StyleDialect::Css,
        );
        let dynamic = parse(
            ".a { color: color-mix(#{$space}, red, blue); }",
            StyleDialect::Scss,
        );
        let invalid = parse(
            ".a { color: color-mix(srgb, red, blue); background: light-dark(white); border-color: contrast-color(red, blue); outline-color: color-mix(in srgb, red); }",
            StyleDialect::Css,
        );
        let invalid_argument_head_count = invalid
            .errors()
            .iter()
            .filter(|error| error.message == "invalid function argument head")
            .count();
        let invalid_argument_count = invalid
            .errors()
            .iter()
            .filter(|error| error.message == "invalid function argument count")
            .count();

        assert!(valid.errors().is_empty());
        assert!(dynamic.errors().is_empty());
        assert_eq!(invalid_argument_head_count, 1);
        assert_eq!(invalid_argument_count, 3);
    }

    #[test]
    fn classifies_css_value_functions_case_insensitively() {
        let result = parse(
            ".a { width: CALC(1px + 2px); color: COLOR-MIX(in srgb, red, blue); transform: TRANSLATEX(1px); filter: BLUR(2px); clip-path: POLYGON(0 0, 100% 0, 100% 100%); }",
            StyleDialect::Css,
        );
        let kinds = node_kinds(&result.syntax());

        assert!(result.errors().is_empty());
        assert!(kinds.contains(&SyntaxKind::CalcFunction));
        assert!(kinds.contains(&SyntaxKind::ColorValue));
        assert!(kinds.contains(&SyntaxKind::TransformFunction));
        assert!(kinds.contains(&SyntaxKind::FilterFunction));
        assert!(kinds.contains(&SyntaxKind::ShapeFunction));
    }

    #[test]
    fn validates_values_l4_math_function_argument_counts() {
        let valid = parse(
            ".a { width: calc(1px + 2px); min-width: min(1px, 2px); max-width: max(1px); margin: round(nearest, 10px, 3px); padding: hypot(3px, 4px); opacity: log(8, 2); }",
            StyleDialect::Css,
        );
        let invalid = parse(
            ".a { width: calc(1px, 2px); min-width: min(); max-width: clamp(1px, 2px); margin: mod(10px); padding: sin(); opacity: atan2(1); }",
            StyleDialect::Css,
        );
        let invalid_argument_count = invalid
            .errors()
            .iter()
            .filter(|error| error.message == "invalid function argument count")
            .count();

        assert!(valid.errors().is_empty());
        assert_eq!(invalid_argument_count, 6);
    }

    #[test]
    fn validates_values_l4_math_function_empty_arguments() {
        let valid_fallback = parse(
            ".a { color: var(--brand,); padding: env(safe-area-inset-top,); }",
            StyleDialect::Css,
        );
        let invalid = parse(
            ".a { width: min(, 1px); height: max(1px,); inset: clamp(1px, , 3px); }",
            StyleDialect::Css,
        );
        let empty_argument_count = invalid
            .errors()
            .iter()
            .filter(|error| error.message == "empty function argument")
            .count();

        assert!(valid_fallback.errors().is_empty());
        assert_eq!(empty_argument_count, 3);
    }

    #[test]
    fn validates_var_env_attr_function_argument_heads() {
        let valid = parse(
            ".a { color: var(--brand, red, blue); padding: env(safe-area-inset-top, 0px); content: attr(data-label string, \"x\"); }",
            StyleDialect::Css,
        );
        let dynamic = parse(
            ".a { color: var(#{$name}); padding: env($area); content: attr(#{$attribute}); }",
            StyleDialect::Scss,
        );
        let invalid = parse(
            ".a { color: var(color); padding: env(, 0px); content: attr(123); }",
            StyleDialect::Css,
        );
        let invalid_head_count = invalid
            .errors()
            .iter()
            .filter(|error| error.message == "invalid function argument head")
            .count();

        assert!(valid.errors().is_empty());
        assert!(dynamic.errors().is_empty());
        assert_eq!(invalid_head_count, 3);
    }

    #[test]
    fn structures_css_value_atoms_and_function_argument_lists() {
        let result = parse(
            ".a { color: #fff; width: clamp(1rem, calc(2px + 3px), 4rem); opacity: 50%; z-index: 1; font-family: system, \"Demo\"; unicode-range: U+00A0-00FF; }",
            StyleDialect::Css,
        );
        let kinds = node_kinds(&result.syntax());
        let dimension_value_count = kinds
            .iter()
            .filter(|kind| **kind == SyntaxKind::DimensionValue)
            .count();
        let number_value_count = kinds
            .iter()
            .filter(|kind| **kind == SyntaxKind::NumberValue)
            .count();
        let percentage_value_count = kinds
            .iter()
            .filter(|kind| **kind == SyntaxKind::PercentageValue)
            .count();

        assert!(result.errors().is_empty());
        assert!(kinds.contains(&SyntaxKind::ColorValue));
        assert!(kinds.contains(&SyntaxKind::ValueList));
        assert!(kinds.contains(&SyntaxKind::CalcFunction));
        assert!(kinds.contains(&SyntaxKind::BinaryExpression));
        assert!(kinds.contains(&SyntaxKind::IdentifierValue));
        assert!(kinds.contains(&SyntaxKind::StringValue));
        assert!(kinds.contains(&SyntaxKind::UnicodeRangeValue));
        assert!(dimension_value_count >= 4);
        assert!(number_value_count >= 1);
        assert!(percentage_value_count >= 1);
    }

    #[test]
    fn parses_custom_property_values_as_component_value_lists() {
        let result = parse(
            ".a { --api: { display: none }; --empty: ; color: red; }",
            StyleDialect::Css,
        );
        let kinds = node_kinds(&result.syntax());
        let tokens = token_kinds(&result.syntax());
        let component_value_list_count = kinds
            .iter()
            .filter(|kind| **kind == SyntaxKind::ComponentValueList)
            .count();

        assert!(result.errors().is_empty());
        assert!(tokens.contains(&SyntaxKind::CustomPropertyName));
        assert!(kinds.contains(&SyntaxKind::CustomPropertyValue));
        assert!(kinds.contains(&SyntaxKind::SimpleBlock));
        assert_eq!(component_value_list_count, 2);
        assert!(!kinds.contains(&SyntaxKind::BogusValue));
    }

    #[test]
    fn structures_top_level_value_lists_without_function_comma_confusion() {
        let result = parse(
            ".a { font-family: system, sans-serif; color: color-mix(in oklch, red, blue); }",
            StyleDialect::Css,
        );
        let kinds = node_kinds(&result.syntax());

        assert!(result.errors().is_empty());
        assert!(kinds.contains(&SyntaxKind::ValueList));
        assert!(!kinds.contains(&SyntaxKind::BogusValueList));
        assert!(kinds.contains(&SyntaxKind::ColorValue));
    }

    #[test]
    fn structures_bracketed_value_atoms_and_recovery() {
        let closed = parse(
            ".grid { grid-template-columns: [full-start] minmax(0, 1fr) [full-end]; }",
            StyleDialect::Css,
        );
        let missing_close = parse(
            ".grid { grid-template-columns: [full-start 1fr; }",
            StyleDialect::Css,
        );

        assert!(closed.errors().is_empty());
        assert!(node_kinds(&closed.syntax()).contains(&SyntaxKind::BracketedValue));
        assert!(node_kinds(&missing_close.syntax()).contains(&SyntaxKind::BogusBracketedValue));
    }

    #[test]
    fn recovers_bogus_top_level_value_lists() {
        let result = parse(".a { font-family: system, ; }", StyleDialect::Css);
        let kinds = node_kinds(&result.syntax());

        assert!(kinds.contains(&SyntaxKind::BogusValueList));
    }

    #[test]
    fn keeps_important_annotation_in_declaration_values() {
        let result = parse(".a { color: red !important; }", StyleDialect::Css);
        let split = parse(
            ".a { color: red ! /* keep */ important; }",
            StyleDialect::Css,
        );
        let kinds = node_kinds(&result.syntax());
        let split_kinds = node_kinds(&split.syntax());

        assert!(result.errors().is_empty());
        assert!(split.errors().is_empty());
        assert!(kinds.contains(&SyntaxKind::Declaration));
        assert!(kinds.contains(&SyntaxKind::Value));
        assert!(kinds.contains(&SyntaxKind::ImportantAnnotation));
        assert!(split_kinds.contains(&SyntaxKind::ImportantAnnotation));
        assert!(token_kinds(&result.syntax()).contains(&SyntaxKind::Important));
        assert!(token_kinds(&split.syntax()).contains(&SyntaxKind::Ident));
    }

    #[test]
    fn structures_url_values() {
        let result = parse(
            ".a { background: url(images/bg.png); mask: url(\"icons/mask.svg\"); }",
            StyleDialect::Css,
        );
        let kinds = node_kinds(&result.syntax());
        let url_value_count = kinds
            .iter()
            .filter(|kind| **kind == SyntaxKind::UrlValue)
            .count();

        assert!(result.errors().is_empty());
        assert!(kinds.contains(&SyntaxKind::Value));
        assert!(kinds.contains(&SyntaxKind::FunctionCall));
        assert_eq!(url_value_count, 2);
        assert!(token_kinds(&result.syntax()).contains(&SyntaxKind::Url));
    }

    #[test]
    fn structures_bad_strings_as_bogus_values() {
        let result = parse(".a { content: \"bad\ncolor: red; }", StyleDialect::Css);
        let kinds = node_kinds(&result.syntax());

        assert!(
            result
                .errors()
                .iter()
                .any(|error| error.code == ParseErrorCode::UnterminatedString)
        );
        assert!(kinds.contains(&SyntaxKind::BogusValue));
        assert!(token_kinds(&result.syntax()).contains(&SyntaxKind::BadString));
    }

    #[test]
    fn structures_scss_interpolation_in_selector_property_and_value() {
        let result = parse(
            ".button-#{$variant} { #{$prop}: #{$value}; }",
            StyleDialect::Scss,
        );
        let kinds = node_kinds(&result.syntax());
        let interpolation_count = kinds
            .iter()
            .filter(|kind| **kind == SyntaxKind::Interpolation)
            .count();

        assert!(result.errors().is_empty());
        assert_eq!(interpolation_count, 3);
        assert!(kinds.contains(&SyntaxKind::ClassSelector));
        assert!(kinds.contains(&SyntaxKind::PropertyName));
        assert!(kinds.contains(&SyntaxKind::Value));
    }

    #[test]
    fn structures_less_interpolation_in_selector_property_and_value() {
        let result = parse(
            ".button-@{variant} { @{prop}: @{value}; }",
            StyleDialect::Less,
        );
        let kinds = node_kinds(&result.syntax());
        let interpolation_count = kinds
            .iter()
            .filter(|kind| **kind == SyntaxKind::Interpolation)
            .count();

        assert!(result.errors().is_empty());
        assert_eq!(interpolation_count, 3);
        assert!(kinds.contains(&SyntaxKind::ClassSelector));
        assert!(kinds.contains(&SyntaxKind::PropertyName));
        assert!(kinds.contains(&SyntaxKind::Value));
    }

    #[test]
    fn structures_less_escaped_strings_as_values() {
        let result = parse(".a { filter: ~\"alpha(opacity=50)\"; }", StyleDialect::Less);
        let kinds = node_kinds(&result.syntax());
        let token_kinds = token_kinds(&result.syntax());

        assert!(result.errors().is_empty());
        assert!(kinds.contains(&SyntaxKind::Value));
        assert!(token_kinds.contains(&SyntaxKind::LessEscapedString));
    }

    #[test]
    fn structures_less_property_variables_as_values() {
        let result = parse(".a { color: red; background: $color; }", StyleDialect::Less);
        let kinds = node_kinds(&result.syntax());
        let token_kinds = token_kinds(&result.syntax());

        assert!(result.errors().is_empty());
        assert!(kinds.contains(&SyntaxKind::LessPropertyVariable));
        assert!(token_kinds.contains(&SyntaxKind::LessPropertyVariableToken));
    }

    #[test]
    fn structures_unclosed_interpolation_as_bogus() {
        let scss = parse(".button-#{$variant", StyleDialect::Scss);
        let less = parse(".button-@{variant", StyleDialect::Less);

        assert!(node_kinds(&scss.syntax()).contains(&SyntaxKind::BogusInterpolation));
        assert!(node_kinds(&less.syntax()).contains(&SyntaxKind::BogusInterpolation));
        assert!(
            scss.errors()
                .iter()
                .any(|error| error.code == ParseErrorCode::UnexpectedCharacter)
        );
        assert!(
            less.errors()
                .iter()
                .any(|error| error.code == ParseErrorCode::UnexpectedCharacter)
        );
    }

    #[test]
    fn structures_css_value_unary_and_precedence_expressions() {
        let result = parse(".a { margin: -(1rem + 2px) * 3; }", StyleDialect::Css);
        let kinds = node_kinds(&result.syntax());

        assert!(result.errors().is_empty());
        assert!(kinds.contains(&SyntaxKind::UnaryExpression));
        assert!(kinds.contains(&SyntaxKind::ParenthesizedExpression));
        assert!(kinds.contains(&SyntaxKind::BinaryExpression));
    }

    #[test]
    fn structures_dialect_variable_references_in_values() {
        let scss = parse(".a { margin: $gap; }", StyleDialect::Scss);
        let less = parse(".a { margin: @gap; }", StyleDialect::Less);

        assert!(scss.errors().is_empty());
        assert!(less.errors().is_empty());
        assert!(node_kinds(&scss.syntax()).contains(&SyntaxKind::ScssVariableReference));
        assert!(node_kinds(&less.syntax()).contains(&SyntaxKind::LessVariableReference));
    }

    #[test]
    fn structures_scss_variable_flags() {
        let result = parse(
            "$gap: 1rem ! /* keep */ default !global;",
            StyleDialect::Scss,
        );
        let kinds = node_kinds(&result.syntax());
        let flag_count = kinds
            .iter()
            .filter(|kind| **kind == SyntaxKind::ScssVariableFlag)
            .count();

        assert!(result.errors().is_empty());
        assert!(kinds.contains(&SyntaxKind::ScssVariableDeclaration));
        assert_eq!(flag_count, 2);
    }

    #[test]
    fn parses_less_mixin_declarations_calls_and_guards() {
        let result = parse(
            ".theme(@color) when (iscolor(@color)) { color: @color; .rounded(); } .card { .theme(#fff); }",
            StyleDialect::Less,
        );
        let kinds = node_kinds(&result.syntax());

        assert!(result.errors().is_empty());
        assert!(kinds.contains(&SyntaxKind::LessMixinDeclaration));
        assert!(kinds.contains(&SyntaxKind::LessMixinGuard));
        assert!(kinds.contains(&SyntaxKind::LessMixinCall));
        assert!(kinds.contains(&SyntaxKind::LessVariableReference));
        assert!(kinds.contains(&SyntaxKind::Rule));
    }

    #[test]
    fn parses_less_extend_pseudo_class_without_mixin_confusion() {
        let less = parse(
            ".nav:extend(.inline all) { color: red; }",
            StyleDialect::Less,
        );
        let css = parse(
            ".nav:extend(.inline all) { color: red; }",
            StyleDialect::Css,
        );
        let less_kinds = node_kinds(&less.syntax());
        let css_kinds = node_kinds(&css.syntax());

        assert!(less.errors().is_empty());
        assert!(css.errors().is_empty());
        assert!(less_kinds.contains(&SyntaxKind::Rule));
        assert!(less_kinds.contains(&SyntaxKind::LessExtendRule));
        assert!(less_kinds.contains(&SyntaxKind::PseudoSelectorArgument));
        assert!(!less_kinds.contains(&SyntaxKind::LessMixinDeclaration));
        assert!(!css_kinds.contains(&SyntaxKind::LessExtendRule));
        assert!(css_kinds.contains(&SyntaxKind::PseudoClassSelector));
    }

    #[test]
    fn parses_less_detached_ruleset_variable_values() {
        let result = parse(
            "@rules: { color: red; .rounded(); }; .card { color: blue; }",
            StyleDialect::Less,
        );
        let kinds = node_kinds(&result.syntax());

        assert!(result.errors().is_empty());
        assert!(kinds.contains(&SyntaxKind::LessVariableDeclaration));
        assert!(kinds.contains(&SyntaxKind::LessDetachedRulesetNode));
        assert!(kinds.contains(&SyntaxKind::DeclarationList));
        assert!(kinds.contains(&SyntaxKind::Declaration));
        assert!(kinds.contains(&SyntaxKind::LessMixinCall));
        assert!(kinds.contains(&SyntaxKind::Rule));
    }

    #[test]
    fn recovers_unclosed_less_detached_rulesets_as_bogus() {
        let result = parse("@rules: { color: red;", StyleDialect::Less);
        let kinds = node_kinds(&result.syntax());

        assert!(kinds.contains(&SyntaxKind::BogusLessDetachedRuleset));
        assert!(
            result
                .errors()
                .iter()
                .any(|error| error.code == ParseErrorCode::UnexpectedCharacter)
        );
    }

    #[test]
    fn parses_less_namespace_access_calls() {
        let result = parse(
            ".card { #bundle > .rounded(); color: blue; }",
            StyleDialect::Less,
        );
        let kinds = node_kinds(&result.syntax());

        assert!(result.errors().is_empty());
        assert!(kinds.contains(&SyntaxKind::LessNamespaceAccess));
        assert!(kinds.contains(&SyntaxKind::LessMixinCall));
        assert!(kinds.contains(&SyntaxKind::Declaration));
    }

    #[test]
    fn keeps_nested_selectors_separate_from_less_namespace_access() {
        let result = parse(
            ".card { #child > .leaf { color: red; } }",
            StyleDialect::Less,
        );
        let kinds = node_kinds(&result.syntax());

        assert!(result.errors().is_empty());
        assert!(kinds.contains(&SyntaxKind::Rule));
        assert!(!kinds.contains(&SyntaxKind::LessNamespaceAccess));
    }

    #[test]
    fn extracts_initial_style_facts_from_parser_surface() {
        let facts = collect_style_facts(
            "@use \"tokens\"; $gap: 1rem; %surface { color: red; } .card#main { --space: $gap; }",
            StyleDialect::Scss,
        );

        assert_eq!(facts.product, "omena-parser.style-facts");
        assert_eq!(facts.dialect, StyleDialect::Scss);
        assert_eq!(facts.selector_count, 3);
        assert_eq!(facts.variable_count, 3);
        assert_eq!(facts.at_rule_count, 1);
        assert!(facts.selectors.iter().any(|selector| {
            selector.kind == ParsedSelectorFactKind::Class && selector.name == "card"
        }));
        assert!(facts.selectors.iter().any(|selector| {
            selector.kind == ParsedSelectorFactKind::Id && selector.name == "main"
        }));
        assert!(facts.selectors.iter().any(|selector| {
            selector.kind == ParsedSelectorFactKind::Placeholder && selector.name == "surface"
        }));
        assert!(facts.variables.iter().any(|variable| {
            variable.kind == ParsedVariableFactKind::ScssDeclaration && variable.name == "$gap"
        }));
        assert!(facts.variables.iter().any(|variable| {
            variable.kind == ParsedVariableFactKind::ScssReference && variable.name == "$gap"
        }));
        assert!(facts.variables.iter().any(|variable| {
            variable.kind == ParsedVariableFactKind::CustomPropertyDeclaration
                && variable.name == "--space"
        }));
        assert_eq!(facts.at_rules[0].node_kind, Some(SyntaxKind::ScssUseRule));
    }

    #[test]
    fn extracts_sass_symbol_style_facts() {
        let facts = collect_style_facts(
            "@mixin tone($color) { color: $color; } @function double($x) { @return $x * 2; } .card { @include tone(red); width: double(2px); }",
            StyleDialect::Scss,
        );
        let symbol_kinds = facts
            .sass_symbols
            .iter()
            .map(|symbol| (symbol.kind, symbol.name.as_str(), symbol.role))
            .collect::<Vec<_>>();

        assert_eq!(facts.sass_symbol_count, 8);
        assert!(symbol_kinds.contains(&(
            ParsedSassSymbolFactKind::MixinDeclaration,
            "tone",
            "declaration"
        )));
        assert!(symbol_kinds.contains(&(
            ParsedSassSymbolFactKind::MixinInclude,
            "tone",
            "include"
        )));
        assert!(symbol_kinds.contains(&(
            ParsedSassSymbolFactKind::FunctionDeclaration,
            "double",
            "declaration"
        )));
        assert!(symbol_kinds.contains(&(ParsedSassSymbolFactKind::FunctionCall, "double", "call")));
        assert!(symbol_kinds.contains(&(
            ParsedSassSymbolFactKind::VariableDeclaration,
            "color",
            "declaration"
        )));
        assert!(symbol_kinds.contains(&(
            ParsedSassSymbolFactKind::VariableReference,
            "color",
            "reference"
        )));
    }

    #[test]
    fn extracts_animation_name_style_facts() {
        let facts = collect_style_facts(
            "@keyframes fade { from { opacity: 0; } to { opacity: 1; } } @keyframes \"slide\" { to { opacity: 1; } } .card { animation-name: fade, \"slide\", none; }",
            StyleDialect::Css,
        );
        let keyframe_names = facts
            .animations
            .iter()
            .filter(|animation| animation.kind == ParsedAnimationFactKind::KeyframesDeclaration)
            .map(|animation| animation.name.as_str())
            .collect::<Vec<_>>();
        let reference_names = facts
            .animations
            .iter()
            .filter(|animation| animation.kind == ParsedAnimationFactKind::AnimationNameReference)
            .map(|animation| animation.name.as_str())
            .collect::<Vec<_>>();

        assert_eq!(facts.animation_count, 4);
        assert_eq!(keyframe_names, vec!["fade", "slide"]);
        assert_eq!(reference_names, vec!["fade", "slide"]);
    }

    #[test]
    fn extracts_animation_shorthand_style_facts() {
        let facts = collect_style_facts(
            "@keyframes fade { to { opacity: 1; } } @keyframes \"slide\" { to { opacity: 1; } } .card { animation: 1s ease-in fade, \"slide\" 2s linear both, none 1s, var(--anim) 1s; }",
            StyleDialect::Css,
        );
        let keyframe_names = facts
            .animations
            .iter()
            .filter(|animation| animation.kind == ParsedAnimationFactKind::KeyframesDeclaration)
            .map(|animation| animation.name.as_str())
            .collect::<Vec<_>>();
        let reference_names = facts
            .animations
            .iter()
            .filter(|animation| animation.kind == ParsedAnimationFactKind::AnimationNameReference)
            .map(|animation| animation.name.as_str())
            .collect::<Vec<_>>();

        assert_eq!(facts.animation_count, 4);
        assert_eq!(keyframe_names, vec!["fade", "slide"]);
        assert_eq!(reference_names, vec!["fade", "slide"]);
    }

    #[test]
    fn keeps_at_rule_header_dashed_idents_out_of_custom_property_facts() {
        let facts = collect_style_facts(
            "@property --accent { syntax: \"<color>\"; inherits: false; initial-value: red; } @font-palette-values --brand { font-family: Demo; } @color-profile --display-p3 { src: url(p3.icc); } @position-try --popover { inset-area: top; }",
            StyleDialect::Css,
        );
        let custom_properties: Vec<&str> = facts
            .variables
            .iter()
            .filter(|variable| {
                matches!(
                    variable.kind,
                    ParsedVariableFactKind::CustomPropertyDeclaration
                        | ParsedVariableFactKind::CustomPropertyReference
                )
            })
            .map(|variable| variable.name.as_str())
            .collect();

        assert_eq!(custom_properties, vec!["--accent"]);
    }

    #[test]
    fn extracts_all_top_level_classes_from_complex_selector_headers() {
        let facts = collect_style_facts(
            "#app.theme > .card:has(> .icon) { color: red; }",
            StyleDialect::Css,
        );
        let class_names: Vec<&str> = facts
            .selectors
            .iter()
            .filter(|selector| selector.kind == ParsedSelectorFactKind::Class)
            .map(|selector| selector.name.as_str())
            .collect();

        assert_eq!(class_names, vec!["theme", "card"]);
    }

    #[test]
    fn extracts_css_nesting_at_rule_selector_facts() {
        let facts = collect_style_facts(
            ".card { @nest &__icon { color: red; &--active { color: blue; } } }",
            StyleDialect::Css,
        );
        let class_names: Vec<&str> = facts
            .selectors
            .iter()
            .filter(|selector| selector.kind == ParsedSelectorFactKind::Class)
            .map(|selector| selector.name.as_str())
            .collect();

        assert_eq!(
            class_names,
            vec!["card", "card__icon", "card__icon--active"]
        );
    }

    #[test]
    fn parses_mid_typing_char_boundary_edits_without_panicking() {
        let fixtures = [
            (
                StyleDialect::Css,
                ".card { color: color-mix(in oklch, red, blue); }",
            ),
            (
                StyleDialect::Scss,
                "@use \"tokens\" with ($gap: 1rem); .card { &__아이콘 { color: $gap; } }",
            ),
            (
                StyleDialect::Sass,
                ".card\n  color: red\n  &__icon\n    color: blue\n",
            ),
            (
                StyleDialect::Less,
                "@tone: red; .card() when (iscolor(@tone)) { color: @tone; }",
            ),
        ];
        let insertions = [" ", "{", "}", ":", "@media (", "한"];

        for (dialect, source) in fixtures {
            for offset in char_boundary_offsets(source) {
                for insertion in insertions {
                    let mut edited = source.to_string();
                    edited.insert_str(offset, insertion);
                    let _ = parse(&edited, dialect);
                }
            }
        }
    }

    #[test]
    fn parses_deterministic_malformed_byte_corpus_without_panicking() {
        let mut byte_fixtures = vec![
            Vec::new(),
            b"\0".to_vec(),
            b"\xef\xbb\xbf.card { color: red; }".to_vec(),
            b".a { content: \"unterminated".to_vec(),
            b".a { background: url(foo bar) }".to_vec(),
            b"@media screen { .a { color: red".to_vec(),
            b".a { --x: { [ ( ; }".to_vec(),
            vec![0xff, b'.', b'a', b' ', b'{', b'}'],
            vec![0xe1, 0x84, b'.', b'a', b'{', b'c', b':', b'r'],
        ];
        for seed in 0..32u32 {
            byte_fixtures.push(deterministic_byte_fixture(seed));
        }

        for bytes in byte_fixtures {
            let source = String::from_utf8_lossy(&bytes).into_owned();
            for dialect in [
                StyleDialect::Css,
                StyleDialect::Scss,
                StyleDialect::Sass,
                StyleDialect::Less,
            ] {
                let parse_result = std::panic::catch_unwind(|| parse(&source, dialect));
                assert!(
                    parse_result.is_ok(),
                    "parse panicked for dialect={dialect:?} source={source:?}"
                );
                let Ok(parse_result) = parse_result else {
                    continue;
                };

                let lex_result = std::panic::catch_unwind(|| lex(&source, dialect));
                assert!(
                    lex_result.is_ok(),
                    "lex panicked for dialect={dialect:?} source={source:?}"
                );
                let Ok(lex_result) = lex_result else {
                    continue;
                };

                assert_eq!(parse_result.syntax().kind(), SyntaxKind::Root);
                assert_lex_ranges_are_char_boundaries(&source, lex_result.tokens());
            }
        }
    }

    #[test]
    fn preserves_lossless_cst_text_for_valid_corpus() {
        let fixtures = [
            (
                StyleDialect::Css,
                ".card { color: red; --space: calc(1rem + 2px); }",
            ),
            (
                StyleDialect::Scss,
                "@use \"tokens\"; .card { &__icon { color: $accent; } }",
            ),
            (
                StyleDialect::Sass,
                ".card\n  color: red\n  &__icon\n    color: blue\n",
            ),
            (
                StyleDialect::Less,
                "@tone: red; .card() when (iscolor(@tone)) { color: @tone; }",
            ),
        ];

        for (dialect, source) in fixtures {
            let result = parse(source, dialect);
            let syntax = result.syntax();

            assert_eq!(syntax.kind(), SyntaxKind::Root);
            assert_eq!(source_text(&syntax).as_deref(), Some(source));
            assert_eq!(result.source_text().as_deref(), Some(source));

            let reparsed = parse(&result.source_text().unwrap_or_default(), dialect);
            assert_eq!(reparsed.source_text().as_deref(), Some(source));
            assert_eq!(reparsed.syntax().kind(), SyntaxKind::Root);
        }
    }

    #[test]
    fn extracts_nested_bem_style_facts_with_parent_context() {
        let facts = collect_style_facts(
            ".card { &__icon { &--small { color: red; } } --space: 1rem; color: var(--space); }",
            StyleDialect::Scss,
        );
        let class_names: Vec<&str> = facts
            .selectors
            .iter()
            .filter(|selector| selector.kind == ParsedSelectorFactKind::Class)
            .map(|selector| selector.name.as_str())
            .collect();
        let custom_properties: Vec<&str> = facts
            .variables
            .iter()
            .map(|variable| variable.name.as_str())
            .collect();

        assert_eq!(class_names, vec!["card", "card__icon", "card__icon--small"]);
        assert!(custom_properties.contains(&"--space"));
        assert!(!custom_properties.contains(&"--small"));
        assert_eq!(facts.error_count, 0);
    }

    #[test]
    fn ignores_non_defining_selector_function_arguments() {
        let facts = collect_style_facts(
            ".btn:is(.active, .primary):has(#target, %surface) { color: red; }",
            StyleDialect::Scss,
        );
        let class_names: Vec<&str> = facts
            .selectors
            .iter()
            .filter(|selector| selector.kind == ParsedSelectorFactKind::Class)
            .map(|selector| selector.name.as_str())
            .collect();
        let id_names: Vec<&str> = facts
            .selectors
            .iter()
            .filter(|selector| selector.kind == ParsedSelectorFactKind::Id)
            .map(|selector| selector.name.as_str())
            .collect();
        let placeholder_names: Vec<&str> = facts
            .selectors
            .iter()
            .filter(|selector| selector.kind == ParsedSelectorFactKind::Placeholder)
            .map(|selector| selector.name.as_str())
            .collect();

        assert_eq!(class_names, vec!["btn"]);
        assert!(id_names.is_empty());
        assert!(placeholder_names.is_empty());
    }

    #[test]
    fn filters_css_module_global_scope_selector_facts() {
        let facts = collect_style_facts(
            ":global { .reset { color: red; } } :global(.standalone) { color: red; } .card :global(.child) { color: red; } :local(.button) { color: blue; }",
            StyleDialect::Css,
        );
        let outer_local = collect_style_facts(
            ":local { :global { .kept { color: green; } } }",
            StyleDialect::Css,
        );
        let class_names = facts
            .selectors
            .iter()
            .filter(|selector| selector.kind == ParsedSelectorFactKind::Class)
            .map(|selector| selector.name.as_str())
            .collect::<Vec<_>>();
        let outer_local_class_names = outer_local
            .selectors
            .iter()
            .filter(|selector| selector.kind == ParsedSelectorFactKind::Class)
            .map(|selector| selector.name.as_str())
            .collect::<Vec<_>>();

        assert_eq!(class_names, vec!["card", "button"]);
        assert_eq!(outer_local_class_names, vec!["kept"]);
    }

    #[test]
    fn extracts_css_module_local_id_selector_facts() {
        let facts = collect_style_facts(
            ":local(#panel) { color: red; } :global(#reset) { color: red; } .card :global(#child) { color: blue; }",
            StyleDialect::Css,
        );
        let class_names = facts
            .selectors
            .iter()
            .filter(|selector| selector.kind == ParsedSelectorFactKind::Class)
            .map(|selector| selector.name.as_str())
            .collect::<Vec<_>>();
        let id_names = facts
            .selectors
            .iter()
            .filter(|selector| selector.kind == ParsedSelectorFactKind::Id)
            .map(|selector| selector.name.as_str())
            .collect::<Vec<_>>();

        assert_eq!(class_names, vec!["card"]);
        assert_eq!(id_names, vec!["panel"]);
    }

    #[test]
    fn extracts_css_module_local_selector_list_facts() {
        let facts = collect_style_facts(
            ":local(.button, .link:hover) { color: red; } :global(.reset, .theme) { color: blue; }",
            StyleDialect::Css,
        );
        let class_names = facts
            .selectors
            .iter()
            .filter(|selector| selector.kind == ParsedSelectorFactKind::Class)
            .map(|selector| selector.name.as_str())
            .collect::<Vec<_>>();

        assert_eq!(class_names, vec!["button", "link"]);
    }

    #[test]
    fn keeps_trailing_local_selector_group_classes() {
        let facts = collect_style_facts(
            ":local(.button) .icon, :local(.card).active { color: red; }",
            StyleDialect::Css,
        );
        let mut class_names = facts
            .selectors
            .iter()
            .filter(|selector| selector.kind == ParsedSelectorFactKind::Class)
            .map(|selector| selector.name.as_str())
            .collect::<Vec<_>>();
        class_names.sort_unstable();

        assert_eq!(class_names, vec!["active", "button", "card", "icon"]);
    }

    #[test]
    fn parses_functional_pseudo_selector_lists_with_bogus_item_recovery() {
        let result = parse(
            ".btn:is(#it/typo, .ok):where(.wide, .compact) { color: red; }",
            StyleDialect::Css,
        );
        let kinds = node_kinds(&result.syntax());
        let selector_list_count = kinds
            .iter()
            .filter(|kind| **kind == SyntaxKind::SelectorList)
            .count();
        let class_selector_count = kinds
            .iter()
            .filter(|kind| **kind == SyntaxKind::ClassSelector)
            .count();

        assert!(kinds.contains(&SyntaxKind::Rule));
        assert!(kinds.contains(&SyntaxKind::Declaration));
        assert!(kinds.contains(&SyntaxKind::PseudoSelectorArgument));
        assert!(kinds.contains(&SyntaxKind::BogusSelector));
        assert!(!kinds.contains(&SyntaxKind::BogusRule));
        assert!(selector_list_count >= 3);
        assert!(class_selector_count >= 4);
        assert!(
            result
                .errors()
                .iter()
                .any(|error| error.message == "invalid selector in selector list")
        );
    }

    #[test]
    fn parses_not_arguments_as_strict_selector_lists() {
        let forgiving = parse(".btn:is(#it/typo, .ok) { color: red; }", StyleDialect::Css);
        let strict = parse(".btn:not(#it/typo, .ok) { color: red; }", StyleDialect::Css);
        let forgiving_kinds = node_kinds(&forgiving.syntax());
        let strict_kinds = node_kinds(&strict.syntax());

        assert!(forgiving_kinds.contains(&SyntaxKind::BogusSelector));
        assert!(!forgiving_kinds.contains(&SyntaxKind::BogusSelectorList));
        assert!(strict_kinds.contains(&SyntaxKind::BogusSelector));
        assert!(strict_kinds.contains(&SyntaxKind::BogusSelectorList));
    }

    #[test]
    fn parses_nth_child_of_selector_lists_as_cst_nodes() {
        let result = parse(
            ".grid > :nth-child(2n + 1 of .item, [data-active]) { color: red; }",
            StyleDialect::Css,
        );
        let kinds = node_kinds(&result.syntax());
        let selector_list_count = kinds
            .iter()
            .filter(|kind| **kind == SyntaxKind::SelectorList)
            .count();

        assert!(result.errors().is_empty());
        assert!(kinds.contains(&SyntaxKind::NthSelectorArgument));
        assert!(kinds.contains(&SyntaxKind::NthSelectorFormula));
        assert!(kinds.contains(&SyntaxKind::NthSelectorOfSelectorList));
        assert!(kinds.contains(&SyntaxKind::ClassSelector));
        assert!(kinds.contains(&SyntaxKind::AttributeSelector));
        assert!(selector_list_count >= 2);
    }

    #[test]
    fn parses_nth_of_type_arguments_as_formula_cst_nodes() {
        let result = parse("li:nth-of-type(2n + 1) { color: red; }", StyleDialect::Css);
        let kinds = node_kinds(&result.syntax());

        assert!(result.errors().is_empty());
        assert!(kinds.contains(&SyntaxKind::NthSelectorArgument));
        assert!(kinds.contains(&SyntaxKind::NthSelectorFormula));
        assert!(!kinds.contains(&SyntaxKind::NthSelectorOfSelectorList));
    }

    #[test]
    fn parses_has_arguments_as_relative_selector_lists() {
        let result = parse(
            ".card:has(> .icon, + [data-active], :has(~ .nested)) { color: red; }",
            StyleDialect::Css,
        );
        let kinds = node_kinds(&result.syntax());
        let relative_selector_count = kinds
            .iter()
            .filter(|kind| **kind == SyntaxKind::RelativeSelector)
            .count();
        let relative_list_count = kinds
            .iter()
            .filter(|kind| **kind == SyntaxKind::RelativeSelectorList)
            .count();

        assert!(result.errors().is_empty());
        assert_eq!(relative_list_count, 2);
        assert_eq!(relative_selector_count, 4);
        assert!(kinds.contains(&SyntaxKind::Combinator));
        assert!(kinds.contains(&SyntaxKind::AttributeSelector));
        assert!(kinds.contains(&SyntaxKind::PseudoClassSelector));
    }

    #[test]
    fn parses_lang_and_dir_arguments_as_cst_nodes() {
        let result = parse(
            ":lang(en-US, \"ko\") .card:dir(rtl) { color: red; }",
            StyleDialect::Css,
        );
        let kinds = node_kinds(&result.syntax());
        let language_tag_count = kinds
            .iter()
            .filter(|kind| **kind == SyntaxKind::LanguageTag)
            .count();

        assert!(
            result.errors().is_empty(),
            "unexpected parse errors: {:?}",
            result.errors()
        );
        assert!(kinds.contains(&SyntaxKind::LanguageSelectorArgument));
        assert!(kinds.contains(&SyntaxKind::DirectionalitySelectorArgument));
        assert_eq!(language_tag_count, 2);
    }

    #[test]
    fn decomposes_selector_lists_into_selector_nodes() {
        let result = parse(
            ".card:hover > #title, article.card || .icon[data-active] { color: red; }",
            StyleDialect::Css,
        );
        let kinds = node_kinds(&result.syntax());

        assert!(result.errors().is_empty());
        assert!(kinds.contains(&SyntaxKind::Selector));
        assert!(kinds.contains(&SyntaxKind::ComplexSelector));
        assert!(kinds.contains(&SyntaxKind::CompoundSelector));
        assert!(kinds.contains(&SyntaxKind::ClassSelector));
        assert!(kinds.contains(&SyntaxKind::IdSelector));
        assert!(kinds.contains(&SyntaxKind::TypeSelector));
        assert!(kinds.contains(&SyntaxKind::PseudoClassSelector));
        assert!(kinds.contains(&SyntaxKind::AttributeSelector));
        assert!(kinds.contains(&SyntaxKind::Combinator));
    }

    #[test]
    fn parses_namespace_qualified_selectors() {
        let result = parse(
            "@namespace svg url(\"http://www.w3.org/2000/svg\"); svg|a, *|button, |main, svg|*, *|* { color: red; }",
            StyleDialect::Css,
        );
        let kinds = node_kinds(&result.syntax());
        let namespace_prefix_count = kinds
            .iter()
            .filter(|kind| **kind == SyntaxKind::NamespacePrefix)
            .count();
        let type_selector_count = kinds
            .iter()
            .filter(|kind| **kind == SyntaxKind::TypeSelector)
            .count();
        let universal_selector_count = kinds
            .iter()
            .filter(|kind| **kind == SyntaxKind::UniversalSelector)
            .count();

        assert!(result.errors().is_empty());
        assert_eq!(namespace_prefix_count, 5);
        assert_eq!(type_selector_count, 3);
        assert_eq!(universal_selector_count, 2);
    }

    #[test]
    fn decomposes_attribute_matchers_into_cst_nodes() {
        let result = parse(
            ".a[data-state~=\"active\"][lang|=\"en\"][href^=\"/docs\"][href$=\".pdf\"][class*=\"btn\"][data-mode=\"x\" i] { color: red; }",
            StyleDialect::Css,
        );
        let kinds = node_kinds(&result.syntax());
        let matcher_count = kinds
            .iter()
            .filter(|kind| **kind == SyntaxKind::AttributeMatcher)
            .count();
        let name_count = kinds
            .iter()
            .filter(|kind| **kind == SyntaxKind::AttributeName)
            .count();
        let value_count = kinds
            .iter()
            .filter(|kind| **kind == SyntaxKind::AttributeValue)
            .count();

        assert!(result.errors().is_empty());
        assert!(kinds.contains(&SyntaxKind::AttributeSelector));
        assert_eq!(matcher_count, 6);
        assert_eq!(name_count, 6);
        assert_eq!(value_count, 6);
        assert!(kinds.contains(&SyntaxKind::AttributeModifier));
    }

    #[test]
    fn decomposes_css_module_scope_functions_into_cst_nodes() {
        let result = parse(
            ":local(.button) { color: red; } :global(.reset) { box-sizing: border-box; }",
            StyleDialect::Css,
        );
        let kinds = node_kinds(&result.syntax());

        assert!(result.errors().is_empty());
        assert!(kinds.contains(&SyntaxKind::PseudoClassSelector));
        assert!(kinds.contains(&SyntaxKind::PseudoSelectorArgument));
        assert!(kinds.contains(&SyntaxKind::CssModuleLocalBlock));
        assert!(kinds.contains(&SyntaxKind::CssModuleGlobalBlock));
    }

    #[test]
    fn decomposes_nested_and_pseudo_element_selectors() {
        let result = parse("&::before { content: \"\"; }", StyleDialect::Scss);
        let kinds = node_kinds(&result.syntax());

        assert!(result.errors().is_empty());
        assert!(kinds.contains(&SyntaxKind::NestingSelectorNode));
        assert!(kinds.contains(&SyntaxKind::PseudoElementSelector));
    }

    #[test]
    fn parses_sass_indented_blocks_as_rule_declaration_lists() {
        let result = parse(
            ".card\n  color: red\n  .title\n    color: blue\n",
            StyleDialect::Sass,
        );
        let kinds = node_kinds(&result.syntax());

        assert!(result.errors().is_empty());
        assert!(kinds.contains(&SyntaxKind::SassIndentedBlock));
        assert!(kinds.contains(&SyntaxKind::Rule));
        assert!(kinds.contains(&SyntaxKind::DeclarationList));
        assert!(kinds.contains(&SyntaxKind::Declaration));
        assert!(kinds.contains(&SyntaxKind::ClassSelector));
    }

    #[test]
    fn extracts_sass_indented_nested_bem_style_facts() {
        let facts = collect_style_facts(".card\n  &__icon\n    color: red\n", StyleDialect::Sass);
        let class_names: Vec<&str> = facts
            .selectors
            .iter()
            .filter(|selector| selector.kind == ParsedSelectorFactKind::Class)
            .map(|selector| selector.name.as_str())
            .collect();

        assert_eq!(class_names, vec!["card", "card__icon"]);
        assert_eq!(facts.error_count, 0);
    }

    #[test]
    fn exposes_typed_cst_wrapper_slice() {
        let result = parse(
            ".card { color: red; --accent: blue; } @media (width >= 1px) { .button { color: var(--accent); } }",
            StyleDialect::Css,
        );
        let cst = result.cst();
        let stylesheet = cst.stylesheet();
        let rules = cst.rules();
        let selectors = cst.selectors();
        let declarations = cst.declarations();
        let values = cst.values();
        let component_values = parse_entry_point(
            "calc(1px + 2px)",
            StyleDialect::Css,
            ParseEntryPoint::ComponentValue,
        )
        .cst()
        .component_values();
        let simple_blocks = parse_entry_point(
            "{ color: red; (width >= 1px) }",
            StyleDialect::Css,
            ParseEntryPoint::SimpleBlock,
        )
        .cst()
        .simple_blocks();
        let component_value_lists = parse_entry_point(
            "red calc(1px + 2px)",
            StyleDialect::Css,
            ParseEntryPoint::ComponentValueList,
        )
        .cst()
        .component_value_lists();
        let comma_separated_component_value_lists = parse_entry_point(
            "red, calc(1px + 2px)",
            StyleDialect::Css,
            ParseEntryPoint::CommaSeparatedComponentValueList,
        )
        .cst()
        .comma_separated_component_value_lists();
        let custom_property_values = result.cst().custom_property_values();
        let at_rules = cst.at_rules();

        assert_eq!(
            stylesheet.as_ref().map(TypedCstNode::kind),
            Some(SyntaxKind::Stylesheet)
        );
        assert_eq!(rules.len(), 2);
        assert_eq!(selectors.len(), 2);
        assert_eq!(declarations.len(), 3);
        assert_eq!(values.len(), 3);
        assert!(!component_values.is_empty());
        assert!(!simple_blocks.is_empty());
        assert!(!component_value_lists.is_empty());
        assert!(!comma_separated_component_value_lists.is_empty());
        assert_eq!(custom_property_values.len(), 1);
        assert!(!at_rules.is_empty());
        assert!(
            at_rules
                .iter()
                .any(|at_rule| at_rule.kind() == SyntaxKind::MediaRule)
        );
        assert!(
            stylesheet
                .and_then(|node| RuleCstNode::cast(node.into_syntax()))
                .is_none()
        );
    }

    #[test]
    fn exposes_typed_bogus_cst_wrapper_slice() {
        let result = parse(".card { color: @; width: ?; }", StyleDialect::Css);
        let cst = result.cst();
        let bogus_kinds: Vec<SyntaxKind> =
            cst.bogus_nodes().iter().map(TypedCstNode::kind).collect();

        assert!(cst.has_bogus_nodes());
        assert!(bogus_kinds.contains(&SyntaxKind::BogusValue));
        assert!(bogus_kinds.contains(&SyntaxKind::BogusToken));
        assert!(bogus_kinds.iter().all(|kind| kind.is_bogus()));
    }

    #[test]
    fn summarizes_green_field_parser_boundary() {
        let summary = summarize_parser_boundary();

        assert_eq!(summary.product, "omena-parser.boundary");
        assert_eq!(summary.dialect_count, 4);
        assert_eq!(summary.shared_name_kind_count, 8);
        assert!(summary.ready_surfaces.contains(&"selectorCstSkeleton"));
        assert!(summary.ready_surfaces.contains(&"lexedTokenTextSurface"));
        assert!(summary.ready_surfaces.contains(&"atRuleRegistrySkeleton"));
        assert!(
            summary
                .ready_surfaces
                .contains(&"prattValueExpressionSkeleton")
        );
        assert!(
            summary
                .ready_surfaces
                .contains(&"attributeMatcherTokenization")
        );
        assert!(summary.ready_surfaces.contains(&"attributeMatcherCstNodes"));
        assert!(
            summary
                .ready_surfaces
                .contains(&"attributeNameValueModifierCstNodes")
        );
        assert!(
            summary
                .ready_surfaces
                .contains(&"specializedValueFunctionCstNodes")
        );
        assert!(
            summary
                .ready_surfaces
                .contains(&"caseInsensitiveFunctionRegistry")
        );
        assert!(
            summary
                .ready_surfaces
                .contains(&"caseInsensitiveAtRuleRegistry")
        );
        assert!(summary.ready_surfaces.contains(&"identifierValueCstNodes"));
        assert!(summary.ready_surfaces.contains(&"stringValueCstNodes"));
        assert!(
            summary
                .ready_surfaces
                .contains(&"unicodeRangeValueCstNodes")
        );
        assert!(
            summary
                .ready_surfaces
                .contains(&"cssModuleScopeFunctionCstNodes")
        );
        assert!(
            summary
                .ready_surfaces
                .contains(&"cssModuleGlobalSelectorFactFiltering")
        );
        assert!(
            summary
                .ready_surfaces
                .contains(&"cssModuleLocalIdSelectorFacts")
        );
        assert!(summary.ready_surfaces.contains(&"cssModuleValueStyleFacts"));
        assert!(
            summary
                .ready_surfaces
                .contains(&"cssModuleComposesStyleFacts")
        );
        assert!(summary.ready_surfaces.contains(&"icssStyleFacts"));
        assert!(summary.ready_surfaces.contains(&"animationNameStyleFacts"));
        assert!(
            summary
                .ready_surfaces
                .contains(&"animationShorthandStyleFacts")
        );
        assert!(
            summary
                .ready_surfaces
                .contains(&"scssStructuredBlockAtRules")
        );
        assert!(
            summary
                .ready_surfaces
                .contains(&"scssControlPreludeValidation")
        );
        assert!(
            summary
                .ready_surfaces
                .contains(&"scssControlStyleFactExtraction")
        );
        assert!(
            summary
                .ready_surfaces
                .contains(&"scssIncludeContentBlockStyleFacts")
        );
        assert!(summary.ready_surfaces.contains(&"scssUtilityAtRules"));
        assert!(summary.ready_surfaces.contains(&"scssVariableFlagCstNodes"));
        assert!(
            summary
                .ready_surfaces
                .contains(&"scssModulePreludeSourceValidation")
        );
        assert!(
            summary
                .ready_surfaces
                .contains(&"scssModulePreludeClauseValidation")
        );
        assert!(
            summary
                .ready_surfaces
                .contains(&"lessMixinDeclarationCstNodes")
        );
        assert!(summary.ready_surfaces.contains(&"lessMixinCallCstNodes"));
        assert!(summary.ready_surfaces.contains(&"lessMixinGuardCstNodes"));
        assert!(summary.ready_surfaces.contains(&"lessExtendPseudoCstNodes"));
        assert!(
            summary
                .ready_surfaces
                .contains(&"lessDetachedRulesetCstNodes")
        );
        assert!(
            summary
                .ready_surfaces
                .contains(&"lessNamespaceAccessCstNodes")
        );
        assert!(
            summary
                .ready_surfaces
                .contains(&"lessPropertyVariableTokenization")
        );
        assert!(
            summary
                .ready_surfaces
                .contains(&"lessPropertyVariableCstNodes")
        );
        assert!(
            summary
                .ready_surfaces
                .contains(&"lessEscapedStringTokenization")
        );
        assert!(
            summary
                .ready_surfaces
                .contains(&"lessEscapedStringValueCstNodes")
        );
        assert!(
            summary
                .ready_surfaces
                .contains(&"importantAnnotationTokenization")
        );
        assert!(summary.ready_surfaces.contains(&"urlTokenization"));
        assert!(summary.ready_surfaces.contains(&"urlValueCstNodes"));
        assert!(
            summary
                .ready_surfaces
                .contains(&"quotedUrlFunctionValueCstNodes")
        );
        assert!(
            summary
                .ready_surfaces
                .contains(&"conditionalAtRulePreludeCstNodes")
        );
        assert!(
            summary
                .ready_surfaces
                .contains(&"supportsAtRulePreludeValidation")
        );
        assert!(
            summary
                .ready_surfaces
                .contains(&"conditionalLevel5AtRuleCstNodes")
        );
        assert!(summary.ready_surfaces.contains(&"mediaQueryCstNodes"));
        assert!(summary.ready_surfaces.contains(&"mediaQueryListValidation"));
        assert!(summary.ready_surfaces.contains(&"importPreludeCstNodes"));
        assert!(
            summary
                .ready_surfaces
                .contains(&"importSourcePreludeValidation")
        );
        assert!(
            summary
                .ready_surfaces
                .contains(&"importTailPreludeValidation")
        );
        assert!(
            summary
                .ready_surfaces
                .contains(&"customMediaPreludeValidation")
        );
        assert!(
            summary
                .ready_surfaces
                .contains(&"propertyAtRuleNameValidation")
        );
        assert!(
            summary
                .ready_surfaces
                .contains(&"namedAtRulePreludeValidation")
        );
        assert!(
            summary
                .ready_surfaces
                .contains(&"containerAtRulePreludeValidation")
        );
        assert!(
            summary
                .ready_surfaces
                .contains(&"charsetNamespaceAtRulePreludeValidation")
        );
        assert!(
            summary
                .ready_surfaces
                .contains(&"keyframesAtRuleNameValidation")
        );
        assert!(
            summary
                .ready_surfaces
                .contains(&"emptyBlockAtRulePreludeValidation")
        );
        assert!(
            summary
                .ready_surfaces
                .contains(&"layerScopePreludeCstNodes")
        );
        assert!(
            summary
                .ready_surfaces
                .contains(&"layerAtRulePreludeValidation")
        );
        assert!(
            summary
                .ready_surfaces
                .contains(&"scopeAtRulePreludeValidation")
        );
        assert!(
            summary
                .ready_surfaces
                .contains(&"pageAtRulePreludeValidation")
        );
        assert!(summary.ready_surfaces.contains(&"pageMarginAtRuleCstNodes"));
        assert!(
            summary
                .ready_surfaces
                .contains(&"modernDeclarationAtRuleCstNodes")
        );
        assert!(
            summary
                .ready_surfaces
                .contains(&"fontFeatureValuesAtRuleCstNodes")
        );
        assert!(
            summary
                .ready_surfaces
                .contains(&"fontFeatureValuesPreludeValidation")
        );
        assert!(
            summary
                .ready_surfaces
                .contains(&"keyframeSelectorListValidation")
        );
        assert!(
            summary
                .ready_surfaces
                .contains(&"viewTransitionAtRuleCstNodes")
        );
        assert!(
            summary
                .ready_surfaces
                .contains(&"genericAtRulePreludeCstNodes")
        );
        assert!(
            summary
                .ready_surfaces
                .contains(&"bogusAtRulePreludeCstNodes")
        );
        assert!(summary.ready_surfaces.contains(&"nestingAtRuleCstNodes"));
        assert!(
            summary
                .ready_surfaces
                .contains(&"customMediaAtRuleCstNodes")
        );
        assert!(summary.ready_surfaces.contains(&"cssColorFunctionCstNodes"));
        assert!(
            summary
                .ready_surfaces
                .contains(&"colorFunctionArgumentChecks")
        );
        assert!(summary.ready_surfaces.contains(&"gradientFunctionCstNodes"));
        assert!(
            summary
                .ready_surfaces
                .contains(&"transformFunctionCstNodes")
        );
        assert!(summary.ready_surfaces.contains(&"filterFunctionCstNodes"));
        assert!(summary.ready_surfaces.contains(&"imageFunctionCstNodes"));
        assert!(summary.ready_surfaces.contains(&"shapeFunctionCstNodes"));
        assert!(summary.ready_surfaces.contains(&"envAttrFunctionCstNodes"));
        assert!(summary.ready_surfaces.contains(&"mathFunctionCstNodes"));
        assert!(summary.ready_surfaces.contains(&"mathFunctionArityChecks"));
        assert!(
            summary
                .ready_surfaces
                .contains(&"mathFunctionEmptyArgumentChecks")
        );
        assert!(
            summary
                .ready_surfaces
                .contains(&"varEnvAttrFunctionHeadChecks")
        );
        assert!(
            summary
                .ready_surfaces
                .contains(&"scssInterpolationTokenization")
        );
        assert!(
            summary
                .ready_surfaces
                .contains(&"scssInterpolationCstNodes")
        );
        assert!(
            summary
                .ready_surfaces
                .contains(&"lessInterpolationTokenization")
        );
        assert!(
            summary
                .ready_surfaces
                .contains(&"lessInterpolationCstNodes")
        );
        assert!(
            summary
                .ready_surfaces
                .contains(&"interpolationBogusRecovery")
        );
        assert!(summary.ready_surfaces.contains(&"unicodeRangeTokenization"));
        assert!(summary.ready_surfaces.contains(&"badStringTokenRecovery"));
        assert!(summary.ready_surfaces.contains(&"badStringValueBogusNodes"));
        assert!(
            summary
                .ready_surfaces
                .contains(&"emptyDeclarationValueRecovery")
        );
        assert!(
            summary
                .ready_surfaces
                .contains(&"emptyVariableValueRecovery")
        );
        assert!(
            summary
                .ready_surfaces
                .contains(&"missingSemicolonDeclarationRecovery")
        );
        assert!(summary.ready_surfaces.contains(&"coreBogusPopulationSlice"));
        assert!(
            summary
                .ready_surfaces
                .contains(&"dialectBogusPopulationSlice")
        );
        assert!(summary.ready_surfaces.contains(&"cssModuleValueCstNodes"));
        assert!(
            summary
                .ready_surfaces
                .contains(&"cssModuleComposesCstNodes")
        );
        assert!(summary.ready_surfaces.contains(&"icssModuleBlockCstNodes"));
        assert!(
            summary
                .ready_surfaces
                .contains(&"icssImportSourceValidation")
        );
        assert!(
            summary
                .ready_surfaces
                .contains(&"cssModuleFromClauseSourceValidation")
        );
        assert!(
            summary
                .ready_surfaces
                .contains(&"cssModuleComposesMultipleFromValidation")
        );
        assert!(
            summary
                .ready_surfaces
                .contains(&"cssModuleGlobalComposesValidation")
        );
        assert!(summary.ready_surfaces.contains(&"cssModuleBogusRecovery"));
        assert!(summary.ready_surfaces.contains(&"valueListCstNodes"));
        assert!(summary.ready_surfaces.contains(&"valueListBogusRecovery"));
        assert!(
            summary
                .ready_surfaces
                .contains(&"genericRecoveryBogusNodes")
        );
        assert!(
            summary
                .ready_surfaces
                .contains(&"lightningCssDifferentialCorpusSlice")
        );
        assert!(
            summary
                .ready_surfaces
                .contains(&"midTypingNoPanicPropertySlice")
        );
        assert!(
            summary
                .ready_surfaces
                .contains(&"deterministicPanicFreeCorpus")
        );
        assert!(
            summary
                .ready_surfaces
                .contains(&"losslessCstTextRoundTripSmoke")
        );
        assert!(
            summary
                .ready_surfaces
                .contains(&"parseResultSourceTextSurface")
        );
        assert!(
            summary
                .ready_surfaces
                .contains(&"parseSourceParseRoundTripSmoke")
        );
        assert!(
            summary
                .ready_surfaces
                .contains(&"typedNumericValueAtomCstNodes")
        );
        assert!(summary.ready_surfaces.contains(&"bracketedValueCstNodes"));
        assert!(
            summary
                .ready_surfaces
                .contains(&"importantAnnotationCstNodes")
        );
        assert!(
            summary
                .ready_surfaces
                .contains(&"splitImportantAnnotationCstNodes")
        );
        assert!(
            summary
                .ready_surfaces
                .contains(&"unexpectedValueTokenBogusNodes")
        );
        assert!(summary.ready_surfaces.contains(&"cdoCdcTokenization"));
        assert!(
            summary
                .ready_surfaces
                .contains(&"cssIdentifierEscapeTokenization")
        );
        assert!(
            summary
                .ready_surfaces
                .contains(&"nullAndBomInputPreprocessingSlice")
        );
        assert!(
            summary
                .ready_surfaces
                .contains(&"hashDelimiterTokenization")
        );
        assert!(summary.ready_surfaces.contains(&"cssDashIdentTokenization"));
        assert!(
            summary
                .ready_surfaces
                .contains(&"signedNumericTokenization")
        );
        assert!(
            summary
                .ready_surfaces
                .contains(&"exponentNumericTokenization")
        );
        assert!(summary.ready_surfaces.contains(&"badUrlWhitespaceRecovery"));
        assert!(summary.ready_surfaces.contains(&"parserEntryPointApiSlice"));
        assert!(
            summary
                .ready_surfaces
                .contains(&"ruleListEntryPointApiSlice")
        );
        assert!(
            summary
                .ready_surfaces
                .contains(&"componentValueEntryPointApiSlice")
        );
        assert!(
            summary
                .ready_surfaces
                .contains(&"componentValueListEntryPointApiSlice")
        );
        assert!(
            summary
                .ready_surfaces
                .contains(&"commaSeparatedComponentValueListEntryPointApiSlice")
        );
        assert!(
            summary
                .ready_surfaces
                .contains(&"simpleBlockEntryPointApiSlice")
        );
        assert!(summary.ready_surfaces.contains(&"typedCstWrapperSlice"));
        assert!(
            summary
                .ready_surfaces
                .contains(&"typedBogusCstWrapperSlice")
        );
        assert!(summary.ready_surfaces.contains(&"componentValueCstNodes"));
        assert!(summary.ready_surfaces.contains(&"simpleBlockCstNodes"));
        assert!(summary.ready_surfaces.contains(&"fullBogusPopulation"));
        assert!(
            summary
                .ready_surfaces
                .contains(&"componentValueListCstNodes")
        );
        assert!(
            summary
                .ready_surfaces
                .contains(&"commaSeparatedComponentValueListCstNodes")
        );
        assert!(
            summary
                .ready_surfaces
                .contains(&"customPropertyAnyValueComponentList")
        );
        assert!(
            summary
                .ready_surfaces
                .contains(&"customPropertyValueCstNodes")
        );
        assert!(
            summary
                .ready_surfaces
                .contains(&"functionalPseudoSelectorListCstNodes")
        );
        assert!(
            summary
                .ready_surfaces
                .contains(&"strictNotPseudoSelectorListCstNodes")
        );
        assert!(
            summary
                .ready_surfaces
                .contains(&"nthSelectorOfSelectorListCstNodes")
        );
        assert!(
            summary
                .ready_surfaces
                .contains(&"nthSelectorFormulaCstNodes")
        );
        assert!(
            summary
                .ready_surfaces
                .contains(&"hasRelativeSelectorListCstNodes")
        );
        assert!(
            summary
                .ready_surfaces
                .contains(&"langDirSelectorArgumentCstNodes")
        );
        assert!(
            summary
                .ready_surfaces
                .contains(&"namespaceQualifiedSelectorCstNodes")
        );
        assert!(
            summary
                .ready_surfaces
                .contains(&"selectorFunctionArgumentFactExclusion")
        );
        assert!(
            summary
                .ready_surfaces
                .contains(&"missingBlockCloseBogusTrivia")
        );
        assert!(
            summary
                .ready_surfaces
                .contains(&"initialDialectStatementNodes")
        );
        assert!(
            summary
                .ready_surfaces
                .contains(&"scssNestedPropertyCstNodes")
        );
        assert!(summary.ready_surfaces.contains(&"scssModuleConfigCstNodes"));
        assert!(
            summary
                .ready_surfaces
                .contains(&"scssModuleConfigBogusRecovery")
        );
        assert!(
            summary
                .ready_surfaces
                .contains(&"scssPlaceholderSelectorCstNodes")
        );
        assert!(summary.ready_surfaces.contains(&"recoveryBogusSkeleton"));
        assert!(
            summary
                .ready_surfaces
                .contains(&"styleFactExtractionSurface")
        );
        assert!(
            summary
                .ready_surfaces
                .contains(&"lightningCssSelectorIdAndAtRuleDifferentialSlice")
        );
        assert!(summary.not_ready_surfaces.contains(&"productCutover"));
    }

    fn char_boundary_offsets(source: &str) -> Vec<usize> {
        source
            .char_indices()
            .map(|(offset, _)| offset)
            .chain(std::iter::once(source.len()))
            .collect()
    }

    fn deterministic_byte_fixture(seed: u32) -> Vec<u8> {
        let mut state = seed.wrapping_mul(1_664_525).wrapping_add(1_013_904_223);
        let len = (state as usize % 96) + 1;
        let mut bytes = Vec::with_capacity(len);
        for _ in 0..len {
            state = state.wrapping_mul(1_664_525).wrapping_add(1_013_904_223);
            bytes.push((state >> 24) as u8);
        }
        bytes
    }

    fn assert_lex_ranges_are_char_boundaries(source: &str, tokens: &[LexedToken]) {
        for token in tokens {
            let start = u32::from(token.range.start()) as usize;
            let end = u32::from(token.range.end()) as usize;
            assert!(
                source.is_char_boundary(start),
                "token start is not a char boundary: token={token:?} source={source:?}"
            );
            assert!(
                source.is_char_boundary(end),
                "token end is not a char boundary: token={token:?} source={source:?}"
            );
        }
    }

    fn source_text(node: &SyntaxNode<SyntaxKind>) -> Option<String> {
        let mut text = String::new();
        for token in node
            .descendants_with_tokens()
            .filter_map(|element| element.into_token())
        {
            if let Some(resolver) = token.resolver() {
                text.push_str(token.resolve_text(&**resolver));
            } else if let Some(static_text) = token.static_text() {
                text.push_str(static_text);
            } else {
                return None;
            }
        }
        Some(text)
    }

    fn node_kinds(node: &SyntaxNode<SyntaxKind>) -> Vec<SyntaxKind> {
        let mut kinds = vec![node.kind()];
        for child in node.children() {
            kinds.extend(node_kinds(child));
        }
        kinds
    }

    fn token_kinds(node: &SyntaxNode<SyntaxKind>) -> Vec<SyntaxKind> {
        node.descendants_with_tokens()
            .filter_map(|element| element.into_token().map(|token| token.kind()))
            .collect()
    }
}

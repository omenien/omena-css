//! Green-field parser substrate for omena-css.
//!
//! This crate is intentionally built next to `engine-style-parser`. It owns the
//! future cstree parser track, but it does not replace the current product path
//! until parser parity gates are met.

use cstree::{
    build::GreenNodeBuilder,
    green::GreenNode,
    syntax::SyntaxNode,
    text::{TextRange, TextSize},
};
use omena_interner::NameKind;
pub use omena_syntax::StyleDialect;
use omena_syntax::SyntaxKind;
use std::collections::BTreeSet;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParseResult {
    green: GreenNode,
    errors: Vec<ParseError>,
    token_count: usize,
    dialect: StyleDialect,
}

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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LexedToken {
    pub kind: SyntaxKind,
    pub range: TextRange,
}

impl ParseResult {
    pub fn green(&self) -> &GreenNode {
        &self.green
    }

    pub fn syntax(&self) -> SyntaxNode<SyntaxKind> {
        SyntaxNode::new_root(self.green.clone())
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
pub struct ParsedAtRuleFact {
    pub name: String,
    pub node_kind: Option<SyntaxKind>,
    pub range: TextRange,
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
    let extension = BuiltinDialectExtension::new(dialect);
    parse_with_extension(text, &extension)
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
            })
            .collect(),
        errors,
        dialect: extension.dialect(),
    }
}

pub fn parse_with_extension(text: &str, extension: &impl DialectExtension) -> ParseResult {
    let (tokens, errors) = tokenize(text, extension);
    let token_count = tokens.len();
    let mut parser = Parser::new(tokens, errors, extension.dialect());
    let green = parser.parse();

    ParseResult {
        green,
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
    let at_rules = collect_at_rule_facts_from_tokens(&tokens, extension.dialect());

    ParsedStyleFacts {
        product: "omena-parser.style-facts",
        dialect: extension.dialect(),
        selector_count: selectors.len(),
        selectors,
        variable_count: variables.len(),
        variables,
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
            "specializedValueFunctionCstNodes",
            "valueAtomCstNodes",
            "functionArgumentValueLists",
            "cssModuleScopeFunctionCstNodes",
            "scssStructuredBlockAtRules",
            "scssUtilityAtRules",
            "scssNestedPropertyCstNodes",
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
            "conditionalAtRulePreludeCstNodes",
            "conditionalLevel5AtRuleCstNodes",
            "mediaQueryCstNodes",
            "importPreludeCstNodes",
            "layerScopePreludeCstNodes",
            "pageMarginAtRuleCstNodes",
            "modernDeclarationAtRuleCstNodes",
            "fontFeatureValuesAtRuleCstNodes",
            "viewTransitionAtRuleCstNodes",
            "genericAtRulePreludeCstNodes",
            "bogusAtRulePreludeCstNodes",
            "nestingAtRuleCstNodes",
            "customMediaAtRuleCstNodes",
            "cssColorFunctionCstNodes",
            "gradientFunctionCstNodes",
            "transformFunctionCstNodes",
            "filterFunctionCstNodes",
            "imageFunctionCstNodes",
            "shapeFunctionCstNodes",
            "envAttrFunctionCstNodes",
            "mathFunctionCstNodes",
            "scssInterpolationTokenization",
            "scssInterpolationCstNodes",
            "lessInterpolationTokenization",
            "lessInterpolationCstNodes",
            "interpolationBogusRecovery",
            "unicodeRangeTokenization",
            "badStringTokenRecovery",
            "badStringValueBogusNodes",
            "coreBogusPopulationSlice",
            "dialectBogusPopulationSlice",
            "cssModuleValueCstNodes",
            "cssModuleComposesCstNodes",
            "cssModuleBogusRecovery",
            "valueListCstNodes",
            "valueListBogusRecovery",
            "genericRecoveryBogusNodes",
            "sassIndentedTokenization",
            "sassIndentedBlockCstNodes",
            "sassIndentedStyleFacts",
            "differentialCorpusSeed",
            "initialDialectStatementNodes",
            "recoveryBogusSkeleton",
            "styleFactExtractionSurface",
        ],
        not_ready_surfaces: vec![
            "fullRecursiveDescentGrammar",
            "fullPrattValueParser",
            "fullBogusPopulation",
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

    fn parse(&mut self) -> GreenNode {
        self.builder.start_node(SyntaxKind::Root);
        self.builder.start_node(SyntaxKind::Stylesheet);
        self.parse_stylesheet_items();
        self.builder.finish_node();
        self.builder.finish_node();

        let builder = std::mem::take(&mut self.builder);
        let (green, _) = builder.finish();
        green
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
        let kind = if self.current_starts_less_mixin_declaration() {
            SyntaxKind::LessMixinDeclaration
        } else if self.find_rule_block_open_before_recovery(&[
            SyntaxKind::Semicolon,
            SyntaxKind::SassOptionalSemicolon,
            SyntaxKind::RightBrace,
            SyntaxKind::SassDedent,
        ]) {
            SyntaxKind::Rule
        } else {
            SyntaxKind::BogusRule
        };

        self.builder.start_node(kind);
        if kind == SyntaxKind::LessMixinDeclaration {
            self.parse_less_mixin_header();
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
                self.error_at_current(
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
                self.error_at_current(
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

    fn parse_selector_list(&mut self) {
        let kind = if self.current_kind() == Some(SyntaxKind::LeftBrace) {
            SyntaxKind::BogusSelectorList
        } else {
            SyntaxKind::SelectorList
        };
        self.builder.start_node(kind);
        while !self.at_end() {
            match self.current_kind() {
                Some(SyntaxKind::Comma) => self.token_current(),
                Some(kind) if is_selector_boundary(kind) => break,
                Some(SyntaxKind::SassIndentedNewline) => self.token_current(),
                Some(_) => self.parse_selector(),
                None => break,
            }
        }
        self.builder.finish_node();
    }

    fn parse_selector(&mut self) {
        self.builder.start_node(SyntaxKind::Selector);
        self.builder.start_node(SyntaxKind::ComplexSelector);
        self.parse_complex_selector();
        self.builder.finish_node();
        self.builder.finish_node();
    }

    fn parse_complex_selector(&mut self) {
        let mut has_component = false;
        while !self.at_end() {
            match self.current_kind() {
                Some(kind) if is_selector_boundary(kind) => break,
                Some(SyntaxKind::Whitespace) => {
                    if has_component
                        && self
                            .next_non_trivia_kind()
                            .is_some_and(|kind| !is_selector_boundary(kind) && !is_combinator(kind))
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
                    self.parse_compound_selector();
                    has_component = true;
                }
                None => break,
            }
        }
    }

    fn parse_compound_selector(&mut self) {
        let starts_valid = self
            .current_kind()
            .is_some_and(|kind| selector_component_can_start(kind) || is_interpolation_start(kind));
        self.builder.start_node(if starts_valid {
            SyntaxKind::CompoundSelector
        } else {
            SyntaxKind::BogusCompoundSelector
        });
        let start = self.position;
        while !self.at_end() {
            match self.current_kind() {
                Some(kind)
                    if is_selector_boundary(kind)
                        || kind == SyntaxKind::Whitespace
                        || kind == SyntaxKind::SassIndentedNewline
                        || is_combinator(kind) =>
                {
                    break;
                }
                Some(SyntaxKind::Dot) => self.parse_class_selector(),
                Some(SyntaxKind::Hash) => self.parse_id_selector(),
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
        let mut closed = false;
        while !self.at_end() {
            match self.current_kind() {
                Some(SyntaxKind::RightBracket) => {
                    self.token_current();
                    closed = true;
                    break;
                }
                Some(kind) if is_attribute_matcher(kind) => self.parse_attribute_matcher(),
                Some(kind) if is_selector_boundary(kind) => break,
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

    fn parse_pseudo_selector(&mut self, kind: SyntaxKind) {
        self.builder.start_node(kind);
        self.token_current();
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
            while !self.at_end() {
                match self.current_kind() {
                    Some(SyntaxKind::RightParen) => break,
                    Some(kind) if is_selector_boundary(kind) => break,
                    Some(_) => self.token_current(),
                    None => break,
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
            if kind == SyntaxKind::LessVariableDeclaration
                && self.current_kind() == Some(SyntaxKind::LeftBrace)
            {
                self.parse_less_detached_ruleset();
            } else {
                self.builder.start_node(SyntaxKind::Value);
                self.parse_value_or_value_list_until(&[
                    SyntaxKind::Semicolon,
                    SyntaxKind::SassOptionalSemicolon,
                    SyntaxKind::RightBrace,
                    SyntaxKind::SassDedent,
                ]);
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
            self.builder.start_node(SyntaxKind::Value);
            if kind == SyntaxKind::CssModuleComposesDeclaration {
                self.parse_composes_value_until(&[
                    SyntaxKind::Semicolon,
                    SyntaxKind::SassOptionalSemicolon,
                    SyntaxKind::RightBrace,
                    SyntaxKind::SassDedent,
                ]);
            } else {
                self.parse_value_or_value_list_until(&[
                    SyntaxKind::Semicolon,
                    SyntaxKind::SassOptionalSemicolon,
                    SyntaxKind::RightBrace,
                    SyntaxKind::SassDedent,
                ]);
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

    fn parse_css_module_from_clause(&mut self, recovery: &[SyntaxKind]) {
        let has_source = self
            .non_trivia_token_from(self.position + 1)
            .is_some_and(|(_, kind)| !recovery.contains(&kind));
        self.builder.start_node(if has_source {
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
        }
        self.builder.finish_node();
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
            self.parse_scss_module_prelude();
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

    fn parse_scss_module_prelude(&mut self) {
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
            Some(SyntaxKind::Ident) if self.next_kind() == Some(SyntaxKind::LeftParen) => {
                self.parse_function_call(recovery)
            }
            Some(SyntaxKind::Number | SyntaxKind::Percentage | SyntaxKind::Dimension) => {
                self.builder.start_node(SyntaxKind::DimensionValue);
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
            Some(SyntaxKind::LeftParen) => self.parse_parenthesized_expression(recovery),
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
            Some(_) => self.token_current(),
            None => {
                self.empty_bogus_node(
                    SyntaxKind::BogusValue,
                    ParseErrorCode::ExpectedValue,
                    "expected value",
                );
            }
        }
    }

    fn eat_value_trivia(&mut self) {
        while matches!(self.current_kind(), Some(kind) if kind.is_trivia()) {
            self.token_current();
        }
    }

    fn parse_function_call(&mut self, recovery: &[SyntaxKind]) {
        let specialized_kind = self.current_text().and_then(specialized_function_kind);
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
        if specialized_kind.is_some() {
            self.builder.finish_node();
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
            SyntaxKind::SupportsRule => {
                self.parse_at_rule_prelude_node(SyntaxKind::SupportsCondition)
            }
            SyntaxKind::ContainerRule => {
                self.parse_at_rule_prelude_node(SyntaxKind::ContainerCondition)
            }
            SyntaxKind::ImportRule => self.parse_import_prelude(),
            SyntaxKind::LayerRule => self.parse_at_rule_prelude_node(SyntaxKind::LayerName),
            SyntaxKind::ScopeRule => self.parse_at_rule_prelude_node(SyntaxKind::ScopeRange),
            _ => self.consume_at_rule_prelude_tokens(),
        }
    }

    fn parse_media_query_list(&mut self) {
        self.builder.start_node(SyntaxKind::MediaQueryList);
        while !self.at_end() {
            match self.current_kind() {
                Some(kind) if is_at_rule_prelude_boundary(kind) => break,
                Some(SyntaxKind::Comma) => self.token_current(),
                Some(kind) if is_interpolation_start(kind) => self.parse_interpolation(
                    kind,
                    &[
                        SyntaxKind::Comma,
                        SyntaxKind::LeftBrace,
                        SyntaxKind::Semicolon,
                    ],
                ),
                Some(_) => self.parse_media_query(),
                None => break,
            }
        }
        self.builder.finish_node();
    }

    fn parse_media_query(&mut self) {
        self.builder
            .start_node(self.current_prelude_node_kind(SyntaxKind::MediaQuery));
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

    fn parse_import_prelude(&mut self) {
        self.eat_trivia();
        self.parse_import_source();
        while !self.at_end() {
            match self.current_kind() {
                Some(kind) if is_at_rule_prelude_boundary(kind) => break,
                Some(kind) if kind.is_trivia() => self.token_current(),
                Some(SyntaxKind::Ident) if self.current_text() == Some("layer") => {
                    self.parse_import_tail_node(SyntaxKind::LayerName)
                }
                Some(SyntaxKind::Ident)
                    if self.current_text() == Some("supports")
                        && self.next_kind() == Some(SyntaxKind::LeftParen) =>
                {
                    self.parse_import_tail_node(SyntaxKind::SupportsCondition)
                }
                Some(_) => {
                    self.parse_media_query_list();
                    break;
                }
                None => break,
            }
        }
    }

    fn parse_import_source(&mut self) {
        match self.current_kind() {
            Some(SyntaxKind::Url) => {
                self.builder.start_node(SyntaxKind::UrlValue);
                self.token_current();
                self.builder.finish_node();
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
            }
            Some(SyntaxKind::String) => self.token_current(),
            Some(_) => {}
            None => {}
        }
    }

    fn parse_import_tail_node(&mut self, kind: SyntaxKind) {
        self.builder
            .start_node(self.current_prelude_node_kind(kind));
        self.token_current();
        if self.current_kind() == Some(SyntaxKind::LeftParen) {
            self.parse_balanced_parenthesized_prelude(None);
        }
        self.builder.finish_node();
    }

    fn parse_at_rule_prelude_node(&mut self, kind: SyntaxKind) {
        self.builder
            .start_node(self.current_prelude_node_kind(kind));
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
        self.builder.finish_node();
    }

    fn consume_at_rule_prelude_tokens(&mut self) {
        if self
            .current_kind()
            .is_none_or(|kind| is_at_rule_prelude_boundary(kind))
        {
            return;
        }
        self.builder
            .start_node(self.current_generic_at_rule_prelude_node_kind());
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
        self.builder.finish_node();
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
            self.error_at_current(
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

    fn current_prelude_node_kind(&self, kind: SyntaxKind) -> SyntaxKind {
        if self.current_prelude_is_bogus(kind) {
            bogus_prelude_node_kind(kind).unwrap_or(kind)
        } else {
            kind
        }
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

    fn current_prelude_is_bogus(&self, kind: SyntaxKind) -> bool {
        let recovery = if kind == SyntaxKind::MediaQuery {
            &[
                SyntaxKind::Comma,
                SyntaxKind::LeftBrace,
                SyntaxKind::Semicolon,
            ][..]
        } else {
            &[SyntaxKind::LeftBrace, SyntaxKind::Semicolon][..]
        };

        if !self.current_prelude_parentheses_are_balanced_until(recovery) {
            return true;
        }
        kind == SyntaxKind::LayerName
            && self
                .non_trivia_token_from(self.position)
                .is_some_and(|(_, kind)| kind == SyntaxKind::Semicolon)
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
                '"' | '\'' => self.consume_string(current),
                'u' | 'U' if self.starts_unicode_range() => self.consume_unicode_range(),
                '0'..='9' => self.consume_number(),
                '-' if self.starts_with("--") => {
                    self.consume_name_like(SyntaxKind::CustomPropertyName)
                }
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
                '+' => self.consume_static(SyntaxKind::Plus, start, 1),
                '-' if self.starts_with("-=") => {
                    self.consume_static(SyntaxKind::MinusEquals, start, 2)
                }
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
                '#' => self.consume_name_like(SyntaxKind::Hash),
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
        self.consume_digits();
        if self.current_char() == Some('.') {
            self.offset += 1;
            self.consume_digits();
        }
        if self.current_char() == Some('%') {
            self.offset += 1;
            self.push(SyntaxKind::Percentage, start, self.offset);
            return;
        }
        if matches!(self.current_char(), Some(char) if is_name_start(char)) {
            while matches!(self.current_char(), Some(char) if is_name_continue(char)) {
                let char = self.current_char().unwrap_or_default();
                self.bump_char(char);
            }
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
        self.bump_current();
        while matches!(self.current_char(), Some(char) if is_name_continue(char)) {
            self.bump_current();
        }
        self.push(kind, start, self.offset);
    }

    fn consume_ident_like(&mut self) {
        let start = self.offset;
        while matches!(self.current_char(), Some(char) if is_name_continue(char)) {
            self.bump_current();
        }
        let ident = &self.text[start..self.offset];
        if ident.eq_ignore_ascii_case("url")
            && self.current_char() == Some('(')
            && !self.url_starts_with_quoted_argument()
        {
            self.consume_url_token(start);
            return;
        }
        self.push(SyntaxKind::Ident, start, self.offset);
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
                '"' | '\'' | '(' => {
                    self.consume_bad_url(start);
                    return;
                }
                '\\' => {
                    self.bump_current();
                    if self.current_char().is_some() {
                        self.bump_current();
                    }
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
            self.bump_current();
            if char == ')' {
                break;
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

fn is_name_start(char: char) -> bool {
    char == '_' || char == '-' || char.is_alphabetic() || !char.is_ascii()
}

fn is_name_continue(char: char) -> bool {
    is_name_start(char) || char.is_ascii_digit()
}

fn is_css_at_rule_name(text: &str) -> bool {
    matches!(
        text,
        "@charset"
            | "@container"
            | "@font-face"
            | "@font-feature-values"
            | "@font-palette-values"
            | "@import"
            | "@keyframes"
            | "@layer"
            | "@media"
            | "@namespace"
            | "@page"
            | "@property"
            | "@scope"
            | "@starting-style"
            | "@supports"
            | "@counter-style"
            | "@custom-media"
            | "@color-profile"
            | "@nest"
            | "@position-try"
            | "@view-transition"
            | "@stylistic"
            | "@styleset"
            | "@character-variant"
            | "@swash"
            | "@ornaments"
            | "@annotation"
            | "@historical-forms"
            | "@when"
            | "@else"
    )
}

fn is_interpolation_start(kind: SyntaxKind) -> bool {
    matches!(
        kind,
        SyntaxKind::ScssInterpolationStart | SyntaxKind::LessInterpolationStart
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
    collect_selector_facts_in_range(tokens, 0, tokens.len(), &[], &mut seen, &mut selectors);
    selectors
}

fn collect_selector_facts_in_range(
    tokens: &[Token<'_>],
    start: usize,
    end: usize,
    parent_branches: &[SelectorBranch],
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
                    let branches =
                        resolve_selector_header(tokens, index + 1, open, parent_branches);
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
                        seen,
                        selectors,
                    );
                } else if style_wrapper_at_rule(tokens[index].text) {
                    collect_selector_facts_in_range(
                        tokens,
                        open + 1,
                        close,
                        parent_branches,
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

        let branches = resolve_selector_header(tokens, index, open, parent_branches);
        for branch in &branches {
            push_selector_fact(
                selectors,
                seen,
                ParsedSelectorFactKind::Class,
                branch.name.clone(),
                branch.range,
            );
        }
        for id in collect_id_selector_facts_from_header(tokens, index, open) {
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

        collect_selector_facts_in_range(tokens, open + 1, close, &branches, seen, selectors);
        index = close + 1;
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
    if let Some(local_names) = collect_local_function_selector_names(tokens, start, end) {
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

fn collect_id_selector_facts_from_header(
    tokens: &[Token<'_>],
    start: usize,
    end: usize,
) -> Vec<(String, TextRange)> {
    let mut names = Vec::new();
    for token in &tokens[start..end] {
        if token.kind == SyntaxKind::Hash {
            names.push((token.text.trim_start_matches('#').to_string(), token.range));
        }
    }
    names
}

fn collect_placeholder_selector_facts_from_header(
    tokens: &[Token<'_>],
    start: usize,
    end: usize,
) -> Vec<(String, TextRange)> {
    tokens[start..end]
        .iter()
        .filter(|token| token.kind == SyntaxKind::ScssPlaceholder)
        .map(|token| (token.text.trim_start_matches('%').to_string(), token.range))
        .collect()
}

fn collect_variable_facts_from_tokens(tokens: &[Token<'_>]) -> Vec<ParsedVariableFact> {
    let mut variables = Vec::new();
    for (index, token) in tokens.iter().enumerate() {
        let kind = match token.kind {
            SyntaxKind::ScssVariable => {
                if next_non_trivia_token(tokens, index + 1)
                    .is_some_and(|candidate| candidate.kind == SyntaxKind::Colon)
                {
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
    matches!(
        name,
        "@media"
            | "@supports"
            | "@when"
            | "@else"
            | "@layer"
            | "@scope"
            | "@container"
            | "@starting-style"
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
            let node_kind = at_rule_spec(token.text)
                .or_else(|| match dialect {
                    StyleDialect::Scss | StyleDialect::Sass => scss_at_rule_spec(token.text),
                    StyleDialect::Css | StyleDialect::Less => None,
                })
                .map(|spec| spec.node_kind);
            ParsedAtRuleFact {
                name: token.text.to_string(),
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
    let (node_kind, block_kind) = match text {
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
        "@include" => (SyntaxKind::ScssIncludeRule, AtRuleBlockKind::Raw),
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

fn value_list_item_recovery(recovery: &[SyntaxKind]) -> Vec<SyntaxKind> {
    let mut kinds = vec![SyntaxKind::Comma];
    for kind in recovery {
        if !kinds.contains(kind) {
            kinds.push(*kind);
        }
    }
    kinds
}

fn bogus_prelude_node_kind(kind: SyntaxKind) -> Option<SyntaxKind> {
    match kind {
        SyntaxKind::MediaQuery => Some(SyntaxKind::BogusMediaQuery),
        SyntaxKind::SupportsCondition => Some(SyntaxKind::BogusSupportsCondition),
        SyntaxKind::ContainerCondition => Some(SyntaxKind::BogusContainerCondition),
        SyntaxKind::LayerName => Some(SyntaxKind::BogusLayerName),
        SyntaxKind::ScopeRange => Some(SyntaxKind::BogusScopeRange),
        _ => None,
    }
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
    match text {
        "var" => Some(SyntaxKind::VarFunction),
        "calc" => Some(SyntaxKind::CalcFunction),
        "env" => Some(SyntaxKind::EnvFunction),
        "attr" => Some(SyntaxKind::AttrFunction),
        "min" | "max" | "clamp" | "round" | "mod" | "rem" | "sin" | "cos" | "tan" | "asin"
        | "acos" | "atan" | "atan2" | "pow" | "sqrt" | "hypot" | "log" | "exp" | "abs" | "sign" => {
            Some(SyntaxKind::MathFunction)
        }
        "rgb" | "rgba" | "hsl" | "hsla" | "hwb" | "lab" | "lch" | "oklab" | "oklch" | "color"
        | "color-mix" | "light-dark" | "contrast-color" => Some(SyntaxKind::ColorValue),
        "linear-gradient"
        | "radial-gradient"
        | "conic-gradient"
        | "repeating-linear-gradient"
        | "repeating-radial-gradient"
        | "repeating-conic-gradient" => Some(SyntaxKind::GradientFunction),
        "matrix" | "matrix3d" | "translate" | "translate3d" | "translateX" | "translateY"
        | "translateZ" | "scale" | "scale3d" | "scaleX" | "scaleY" | "scaleZ" | "rotate"
        | "rotate3d" | "rotateX" | "rotateY" | "rotateZ" | "skew" | "skewX" | "skewY"
        | "perspective" => Some(SyntaxKind::TransformFunction),
        "blur" | "brightness" | "contrast" | "drop-shadow" | "grayscale" | "hue-rotate"
        | "invert" | "opacity" | "saturate" | "sepia" => Some(SyntaxKind::FilterFunction),
        "image" | "image-set" | "cross-fade" | "element" | "paint" => {
            Some(SyntaxKind::ImageFunction)
        }
        "path" | "shape" | "ray" | "inset" | "circle" | "ellipse" | "polygon" => {
            Some(SyntaxKind::ShapeFunction)
        }
        _ => None,
    }
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
        assert!(result.errors().is_empty());
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
    fn tokenizes_multibyte_source_without_boundary_errors() {
        let result = parse(".카드 { --간격: \"좋음\"; }", StyleDialect::Css);

        assert!(result.errors().is_empty());
        assert!(result.token_count() >= 8);
    }

    #[test]
    fn reports_unterminated_constructs_without_panicking() {
        let comment = parse("/* open", StyleDialect::Css);
        let string = parse(".a { content: \"open; }", StyleDialect::Css);

        assert_eq!(
            comment.errors().first().map(|error| error.code),
            Some(ParseErrorCode::UnterminatedBlockComment),
        );
        assert_eq!(
            string.errors().first().map(|error| error.code),
            Some(ParseErrorCode::UnterminatedString),
        );
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
    fn tokenizes_unquoted_urls_and_bad_urls() {
        let good = lex(".a { background: url(images/bg.png); }", StyleDialect::Css);
        let bad = lex(".a { background: url(foo\"bar); }", StyleDialect::Css);
        let quoted = lex(
            ".a { background: url(\"images/bg.png\"); }",
            StyleDialect::Css,
        );
        let good_kinds: Vec<SyntaxKind> = good.tokens().iter().map(|token| token.kind).collect();
        let bad_kinds: Vec<SyntaxKind> = bad.tokens().iter().map(|token| token.kind).collect();
        let quoted_kinds: Vec<SyntaxKind> =
            quoted.tokens().iter().map(|token| token.kind).collect();

        assert!(good.errors().is_empty());
        assert!(good_kinds.contains(&SyntaxKind::Url));
        assert!(bad_kinds.contains(&SyntaxKind::BadUrl));
        assert!(!bad.errors().is_empty());
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
            node_kinds(&missing_less_guard_condition.syntax())
                .contains(&SyntaxKind::BogusLessGuard)
        );
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
    fn recovers_css_module_value_and_composes_bogus_nodes() {
        let result = parse(
            "@value from; .bad { composes: from; } .missing { composes base; }",
            StyleDialect::Scss,
        );
        let kinds = node_kinds(&result.syntax());

        assert!(kinds.contains(&SyntaxKind::BogusCssModuleBlock));
        assert!(kinds.contains(&SyntaxKind::BogusFromClause));
        assert!(kinds.contains(&SyntaxKind::BogusComposesTarget));
        assert!(kinds.contains(&SyntaxKind::BogusComposesDeclaration));
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
    fn parses_import_layer_supports_media_prelude() {
        let result = parse(
            "@import url(\"theme.css\") layer(app.theme) supports(display: grid) screen and (min-width: 40rem);",
            StyleDialect::Css,
        );
        let kinds = node_kinds(&result.syntax());

        assert!(result.errors().is_empty());
        assert!(kinds.contains(&SyntaxKind::ImportRule));
        assert!(kinds.contains(&SyntaxKind::UrlValue));
        assert!(kinds.contains(&SyntaxKind::LayerName));
        assert!(kinds.contains(&SyntaxKind::SupportsCondition));
        assert!(kinds.contains(&SyntaxKind::MediaQueryList));
        assert!(kinds.contains(&SyntaxKind::MediaFeature));
    }

    #[test]
    fn parses_layer_and_scope_preludes() {
        let result = parse(
            "@layer reset, app.ui; @scope (.card) to (.card-content) { .title { color: red; } }",
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
            "@use \"theme\" as * with ($gap: 1rem, $enabled: true); @forward \"tokens\" with ($color: red);",
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
            "@mixin card($gap) { .item { gap: $gap; } } @function double($x) { @return $x * 2; } @if $enabled { .on { color: green; } }",
            StyleDialect::Scss,
        );
        let kinds = node_kinds(&result.syntax());

        assert!(result.errors().is_empty());
        assert!(kinds.contains(&SyntaxKind::ScssMixinDeclaration));
        assert!(kinds.contains(&SyntaxKind::ScssFunctionDeclaration));
        assert!(kinds.contains(&SyntaxKind::ScssReturnRule));
        assert!(kinds.contains(&SyntaxKind::ScssControlIf));
        assert!(kinds.contains(&SyntaxKind::DeclarationList));
        assert!(kinds.contains(&SyntaxKind::Rule));
        assert!(kinds.contains(&SyntaxKind::ClassSelector));
        assert!(kinds.contains(&SyntaxKind::ScssVariableReference));
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
            ".a { color: color-mix(in oklch, var(--brand), white 20%); width: clamp(1rem, 2vw, 3rem); content: attr(data-label string, \"x\"); padding: env(safe-area-inset-top); background-image: linear-gradient(red, blue); transform: translateX(1rem) rotate(10deg); filter: blur(2px) brightness(1.1); image-set: image-set(url(a.png) 1x); offset-path: path(\"M0,0 L1,1\"); }",
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
    fn structures_css_value_atoms_and_function_argument_lists() {
        let result = parse(
            ".a { color: #fff; width: clamp(1rem, calc(2px + 3px), 4rem); opacity: 50%; z-index: 1; }",
            StyleDialect::Css,
        );
        let kinds = node_kinds(&result.syntax());
        let dimension_value_count = kinds
            .iter()
            .filter(|kind| **kind == SyntaxKind::DimensionValue)
            .count();

        assert!(result.errors().is_empty());
        assert!(kinds.contains(&SyntaxKind::ColorValue));
        assert!(kinds.contains(&SyntaxKind::ValueList));
        assert!(kinds.contains(&SyntaxKind::CalcFunction));
        assert!(kinds.contains(&SyntaxKind::BinaryExpression));
        assert!(dimension_value_count >= 5);
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
    fn recovers_bogus_top_level_value_lists() {
        let result = parse(".a { font-family: system, ; }", StyleDialect::Css);
        let kinds = node_kinds(&result.syntax());

        assert!(kinds.contains(&SyntaxKind::BogusValueList));
    }

    #[test]
    fn keeps_important_annotation_in_declaration_values() {
        let result = parse(".a { color: red !important; }", StyleDialect::Css);
        let kinds = node_kinds(&result.syntax());

        assert!(result.errors().is_empty());
        assert!(kinds.contains(&SyntaxKind::Declaration));
        assert!(kinds.contains(&SyntaxKind::Value));
        assert!(token_kinds(&result.syntax()).contains(&SyntaxKind::Important));
    }

    #[test]
    fn structures_unquoted_url_values() {
        let result = parse(".a { background: url(images/bg.png); }", StyleDialect::Css);
        let kinds = node_kinds(&result.syntax());

        assert!(result.errors().is_empty());
        assert!(kinds.contains(&SyntaxKind::Value));
        assert!(kinds.contains(&SyntaxKind::UrlValue));
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
            ".btn:is(.active, .primary) { color: red; }",
            StyleDialect::Scss,
        );
        let class_names: Vec<&str> = facts
            .selectors
            .iter()
            .filter(|selector| selector.kind == ParsedSelectorFactKind::Class)
            .map(|selector| selector.name.as_str())
            .collect();

        assert_eq!(class_names, vec!["btn"]);
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
    fn decomposes_attribute_matchers_into_cst_nodes() {
        let result = parse(
            ".a[data-state~=\"active\"][lang|=\"en\"][href^=\"/docs\"][href$=\".pdf\"][class*=\"btn\"] { color: red; }",
            StyleDialect::Css,
        );
        let kinds = node_kinds(&result.syntax());
        let matcher_count = kinds
            .iter()
            .filter(|kind| **kind == SyntaxKind::AttributeMatcher)
            .count();

        assert!(result.errors().is_empty());
        assert!(kinds.contains(&SyntaxKind::AttributeSelector));
        assert_eq!(matcher_count, 5);
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
    fn summarizes_green_field_parser_boundary() {
        let summary = summarize_parser_boundary();

        assert_eq!(summary.product, "omena-parser.boundary");
        assert_eq!(summary.dialect_count, 4);
        assert_eq!(summary.shared_name_kind_count, 8);
        assert!(summary.ready_surfaces.contains(&"selectorCstSkeleton"));
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
                .contains(&"specializedValueFunctionCstNodes")
        );
        assert!(
            summary
                .ready_surfaces
                .contains(&"cssModuleScopeFunctionCstNodes")
        );
        assert!(
            summary
                .ready_surfaces
                .contains(&"scssStructuredBlockAtRules")
        );
        assert!(summary.ready_surfaces.contains(&"scssUtilityAtRules"));
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
                .contains(&"conditionalAtRulePreludeCstNodes")
        );
        assert!(
            summary
                .ready_surfaces
                .contains(&"conditionalLevel5AtRuleCstNodes")
        );
        assert!(summary.ready_surfaces.contains(&"mediaQueryCstNodes"));
        assert!(summary.ready_surfaces.contains(&"importPreludeCstNodes"));
        assert!(
            summary
                .ready_surfaces
                .contains(&"layerScopePreludeCstNodes")
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
        assert!(summary.not_ready_surfaces.contains(&"productCutover"));
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

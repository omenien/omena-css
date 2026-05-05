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
            "cssModuleScopeFunctionCstNodes",
            "scssStructuredBlockAtRules",
            "lessMixinDeclarationCstNodes",
            "lessMixinCallCstNodes",
            "lessMixinGuardCstNodes",
            "importantAnnotationTokenization",
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
                Some(SyntaxKind::RightBrace) => self.token_current(),
                Some(_) => self.parse_rule(),
                None => break,
            }
        }
    }

    fn parse_rule(&mut self) {
        let kind = if self.current_starts_less_mixin_declaration() {
            SyntaxKind::LessMixinDeclaration
        } else if self.find_before_recovery(
            SyntaxKind::LeftBrace,
            &[SyntaxKind::Semicolon, SyntaxKind::RightBrace],
        ) {
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
            self.builder.start_node(SyntaxKind::DeclarationList);
            self.parse_declaration_list();
            self.builder.finish_node();
            if self.current_kind() == Some(SyntaxKind::RightBrace) {
                self.token_current();
            }
        } else {
            self.consume_until_recovery(&[SyntaxKind::Semicolon, SyntaxKind::RightBrace]);
            if self.current_kind() == Some(SyntaxKind::Semicolon) {
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
                Some(SyntaxKind::LeftBrace | SyntaxKind::RightBrace | SyntaxKind::Semicolon) => {
                    break;
                }
                Some(SyntaxKind::Comma) => self.token_current(),
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
        self.builder.start_node(SyntaxKind::CompoundSelector);
        let start = self.position;
        while !self.at_end() {
            match self.current_kind() {
                Some(kind)
                    if is_selector_boundary(kind)
                        || kind == SyntaxKind::Whitespace
                        || is_combinator(kind) =>
                {
                    break;
                }
                Some(SyntaxKind::Dot) => self.parse_class_selector(),
                Some(SyntaxKind::Hash) => self.parse_id_selector(),
                Some(SyntaxKind::Ident) => self.parse_type_selector(),
                Some(SyntaxKind::Star) => self.parse_universal_selector(),
                Some(SyntaxKind::Ampersand) => self.parse_nesting_selector(),
                Some(SyntaxKind::LeftBracket) => self.parse_attribute_selector(),
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

    fn parse_combinator(&mut self) {
        self.builder.start_node(SyntaxKind::Combinator);
        self.token_current();
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
                Some(SyntaxKind::RightBrace) | None => break,
                Some(SyntaxKind::AtKeyword) if self.current_dialect_at_rule_spec().is_some() => {
                    self.parse_dialect_at_rule()
                }
                Some(SyntaxKind::AtKeyword) => self.parse_at_rule(),
                Some(_) if self.current_starts_less_mixin_call() => self.parse_less_mixin_call(),
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

    fn parse_variable_declaration(&mut self, kind: SyntaxKind) {
        self.builder.start_node(kind);
        self.token_current();
        if self.current_kind() == Some(SyntaxKind::Colon) {
            self.token_current();
            self.builder.start_node(SyntaxKind::Value);
            self.parse_value_until(&[SyntaxKind::Semicolon, SyntaxKind::RightBrace]);
            self.builder.finish_node();
        } else {
            self.consume_until_recovery(&[SyntaxKind::Semicolon, SyntaxKind::RightBrace]);
        }
        if self.current_kind() == Some(SyntaxKind::Semicolon) {
            self.token_current();
        }
        self.builder.finish_node();
    }

    fn parse_declaration(&mut self) {
        let kind = if self.find_before_recovery(
            SyntaxKind::Colon,
            &[
                SyntaxKind::Semicolon,
                SyntaxKind::RightBrace,
                SyntaxKind::LeftBrace,
            ],
        ) {
            SyntaxKind::Declaration
        } else {
            SyntaxKind::BogusDeclaration
        };
        self.builder.start_node(kind);
        self.builder.start_node(SyntaxKind::PropertyName);
        while !self.at_end() {
            match self.current_kind() {
                Some(SyntaxKind::Colon | SyntaxKind::Semicolon | SyntaxKind::RightBrace) => break,
                Some(_) => self.token_current(),
                None => break,
            }
        }
        self.builder.finish_node();

        if self.current_kind() == Some(SyntaxKind::Colon) {
            self.token_current();
            self.builder.start_node(SyntaxKind::Value);
            self.parse_value_until(&[SyntaxKind::Semicolon, SyntaxKind::RightBrace]);
            self.builder.finish_node();
        } else {
            self.consume_until_recovery(&[SyntaxKind::Semicolon, SyntaxKind::RightBrace]);
        }

        if self.current_kind() == Some(SyntaxKind::Semicolon) {
            self.token_current();
        }
        self.builder.finish_node();
    }

    fn parse_dialect_at_rule(&mut self) {
        let Some(spec) = self.current_dialect_at_rule_spec() else {
            self.parse_at_rule();
            return;
        };

        self.builder.start_node(spec.node_kind);
        if self.current_kind() == Some(SyntaxKind::AtKeyword) {
            self.token_current();
        }
        while !self.at_end() {
            match self.current_kind() {
                Some(SyntaxKind::Semicolon) => {
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
                Some(_) => self.token_current(),
                None => break,
            }
        }
        self.builder.finish_node();
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
            SyntaxKind::RightBrace,
        ]);
        if self.current_kind() == Some(SyntaxKind::Semicolon) {
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
                    self.builder.start_node(SyntaxKind::LessMixinGuard);
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
                self.parse_function_call()
            }
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
            Some(SyntaxKind::LeftParen) => self.parse_parenthesized_expression(),
            Some(kind) if recovery.contains(&kind) => {
                self.empty_bogus_node(
                    SyntaxKind::BogusValue,
                    ParseErrorCode::ExpectedValue,
                    "expected value",
                );
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

    fn parse_function_call(&mut self) {
        let specialized_kind = self.current_text().and_then(specialized_function_kind);

        self.builder.start_node(SyntaxKind::FunctionCall);
        if let Some(kind) = specialized_kind {
            self.builder.start_node(kind);
        }
        self.token_current();
        if self.current_kind() == Some(SyntaxKind::LeftParen) {
            self.token_current();
            self.builder.start_node(SyntaxKind::FunctionArguments);
            self.parse_value_until(&[SyntaxKind::RightParen]);
            self.builder.finish_node();
            if self.current_kind() == Some(SyntaxKind::RightParen) {
                self.token_current();
            }
        }
        if specialized_kind.is_some() {
            self.builder.finish_node();
        }
        self.builder.finish_node();
    }

    fn parse_parenthesized_expression(&mut self) {
        self.builder.start_node(SyntaxKind::ParenthesizedExpression);
        self.token_current();
        self.parse_value_until(&[SyntaxKind::RightParen]);
        if self.current_kind() == Some(SyntaxKind::RightParen) {
            self.token_current();
        }
        self.builder.finish_node();
    }

    fn parse_at_rule(&mut self) {
        let spec = self.current_text().and_then(at_rule_spec);
        self.builder.start_node(SyntaxKind::AtRule);
        if let Some(spec) = spec {
            self.builder.start_node(spec.node_kind);
        }

        if self.current_kind() == Some(SyntaxKind::AtKeyword) {
            self.token_current();
        }

        while !self.at_end() {
            match self.current_kind() {
                Some(SyntaxKind::Semicolon) => {
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
                Some(_) => self.token_current(),
                None => break,
            }
        }

        if spec.is_some() {
            self.builder.finish_node();
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
                Some(SyntaxKind::RightBrace) | None => break,
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
        self.builder.start_node(SyntaxKind::DeclarationList);
        self.parse_declaration_list();
        self.builder.finish_node();
        if self.current_kind() == Some(SyntaxKind::RightBrace) {
            self.token_current();
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
        self.builder.start_node(SyntaxKind::KeyframeBlock);
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
        while !self.at_end() {
            match self.current_kind() {
                Some(kind) if recovery.contains(&kind) => break,
                Some(_) => self.token_current(),
                None => break,
            }
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

    fn find_before_stop(&self, target: SyntaxKind, stop: &[SyntaxKind]) -> bool {
        let mut index = self.position;
        while let Some(token) = self.tokens.get(index) {
            if token.kind == target {
                return true;
            }
            if stop.contains(&token.kind) {
                return false;
            }
            index += 1;
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
        ) && self.find_before_recovery(
            SyntaxKind::LeftBrace,
            &[
                SyntaxKind::Colon,
                SyntaxKind::Semicolon,
                SyntaxKind::RightBrace,
            ],
        )
    }

    fn current_starts_less_mixin_declaration(&self) -> bool {
        self.dialect == StyleDialect::Less
            && matches!(
                self.current_kind(),
                Some(SyntaxKind::Dot | SyntaxKind::Hash)
            )
            && self.find_before_stop(
                SyntaxKind::LeftParen,
                &[
                    SyntaxKind::LeftBrace,
                    SyntaxKind::Semicolon,
                    SyntaxKind::RightBrace,
                ],
            )
            && self.find_before_recovery(
                SyntaxKind::LeftBrace,
                &[SyntaxKind::Semicolon, SyntaxKind::RightBrace],
            )
    }

    fn current_starts_less_mixin_call(&self) -> bool {
        self.dialect == StyleDialect::Less
            && matches!(
                self.current_kind(),
                Some(SyntaxKind::Dot | SyntaxKind::Hash)
            )
            && self.find_before_stop(
                SyntaxKind::LeftParen,
                &[
                    SyntaxKind::Semicolon,
                    SyntaxKind::RightBrace,
                    SyntaxKind::LeftBrace,
                ],
            )
            && !self.find_before_recovery(
                SyntaxKind::LeftBrace,
                &[SyntaxKind::Semicolon, SyntaxKind::RightBrace],
            )
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
            tokens: Vec::new(),
            errors: Vec::new(),
        }
    }

    fn tokenize(&mut self) {
        while let Some(current) = self.current_char() {
            let start = self.offset;
            match current {
                char if char.is_whitespace() => {
                    self.consume_while(SyntaxKind::Whitespace, |c| c.is_whitespace())
                }
                '/' if self.starts_with("/*") => self.consume_block_comment(),
                '/' if self.starts_with("//") && self.extension.dialect() != StyleDialect::Css => {
                    self.consume_line_comment()
                }
                '!' if self.starts_with_ascii_keyword("!important") => {
                    self.consume_static(SyntaxKind::Important, start, "!important".len())
                }
                '"' | '\'' => self.consume_string(current),
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
                '%' => self.consume_static(SyntaxKind::Percent, start, 1),
                '=' if self.starts_with("=>") => self.consume_static(SyntaxKind::Arrow, start, 2),
                '=' => self.consume_static(SyntaxKind::Equals, start, 1),
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
            self.bump_char(char);
            if char == '\n' {
                break;
            }
        }
        self.push(SyntaxKind::LineComment, start, self.offset);
    }

    fn consume_string(&mut self, quote: char) {
        let start = self.offset;
        self.bump_char(quote);
        while let Some(char) = self.current_char() {
            self.bump_char(char);
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
        self.push(SyntaxKind::Ident, start, self.offset);
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

    fn current_char(&self) -> Option<char> {
        self.text[self.offset..].chars().next()
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
    )
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
                if style_wrapper_at_rule(tokens[index].text) {
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
                if next_non_trivia_token(tokens, index + 1)
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

fn skip_trivia_tokens(tokens: &[Token<'_>], mut index: usize, end: usize) -> usize {
    while index < end && tokens[index].kind.is_trivia() {
        index += 1;
    }
    index
}

fn skip_statement(tokens: &[Token<'_>], mut index: usize, end: usize) -> usize {
    while index < end {
        match tokens[index].kind {
            SyntaxKind::Semicolon => return index + 1,
            SyntaxKind::RightBrace => return index,
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
            SyntaxKind::Semicolon | SyntaxKind::RightBrace => return None,
            SyntaxKind::LeftBrace => {
                let close = matching_right_brace(tokens, index, end)?;
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

fn style_wrapper_at_rule(name: &str) -> bool {
    matches!(
        name,
        "@media" | "@supports" | "@layer" | "@scope" | "@container" | "@starting-style"
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
        "@container" => (SyntaxKind::ContainerRule, AtRuleBlockKind::GroupRuleList),
        "@layer" => (SyntaxKind::LayerRule, AtRuleBlockKind::GroupRuleList),
        "@scope" => (SyntaxKind::ScopeRule, AtRuleBlockKind::GroupRuleList),
        "@starting-style" => (
            SyntaxKind::StartingStyleRule,
            AtRuleBlockKind::GroupRuleList,
        ),
        "@keyframes" => (SyntaxKind::KeyframesRule, AtRuleBlockKind::Keyframes),
        "@font-face" => (SyntaxKind::FontFaceRule, AtRuleBlockKind::DeclarationList),
        "@page" => (SyntaxKind::PageRule, AtRuleBlockKind::DeclarationList),
        "@property" => (SyntaxKind::PropertyRule, AtRuleBlockKind::DeclarationList),
        "@charset" => (SyntaxKind::CharsetRule, AtRuleBlockKind::Raw),
        "@import" => (SyntaxKind::ImportRule, AtRuleBlockKind::Raw),
        "@namespace" => (SyntaxKind::NamespaceRule, AtRuleBlockKind::Raw),
        _ => return None,
    };
    Some(AtRuleSpec {
        node_kind,
        block_kind,
    })
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
        SyntaxKind::Comma | SyntaxKind::LeftBrace | SyntaxKind::RightBrace | SyntaxKind::Semicolon
    )
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
    fn parses_registered_keyframes_and_declaration_at_rules() {
        let keyframes = parse(
            "@keyframes fade { from { opacity: 0; } to { opacity: 1; } }",
            StyleDialect::Css,
        );
        let font_face = parse(
            "@font-face { font-family: \"Demo\"; src: url(demo.woff2); }",
            StyleDialect::Css,
        );
        let keyframe_kinds = node_kinds(&keyframes.syntax());
        let font_face_kinds = node_kinds(&font_face.syntax());

        assert!(keyframes.errors().is_empty());
        assert!(font_face.errors().is_empty());
        assert!(keyframe_kinds.contains(&SyntaxKind::KeyframesRule));
        assert!(keyframe_kinds.contains(&SyntaxKind::KeyframeBlock));
        assert!(font_face_kinds.contains(&SyntaxKind::FontFaceRule));
        assert!(font_face_kinds.contains(&SyntaxKind::DeclarationList));
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
    fn keeps_important_annotation_in_declaration_values() {
        let result = parse(".a { color: red !important; }", StyleDialect::Css);
        let kinds = node_kinds(&result.syntax());

        assert!(result.errors().is_empty());
        assert!(kinds.contains(&SyntaxKind::Declaration));
        assert!(kinds.contains(&SyntaxKind::Value));
        assert!(token_kinds(&result.syntax()).contains(&SyntaxKind::Important));
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
    fn extracts_initial_style_facts_from_parser_surface() {
        let facts = collect_style_facts(
            "@use \"tokens\"; $gap: 1rem; .card#main { --space: $gap; }",
            StyleDialect::Scss,
        );

        assert_eq!(facts.product, "omena-parser.style-facts");
        assert_eq!(facts.dialect, StyleDialect::Scss);
        assert_eq!(facts.selector_count, 2);
        assert_eq!(facts.variable_count, 3);
        assert_eq!(facts.at_rule_count, 1);
        assert!(facts.selectors.iter().any(|selector| {
            selector.kind == ParsedSelectorFactKind::Class && selector.name == "card"
        }));
        assert!(facts.selectors.iter().any(|selector| {
            selector.kind == ParsedSelectorFactKind::Id && selector.name == "main"
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
        assert!(
            summary
                .ready_surfaces
                .contains(&"lessMixinDeclarationCstNodes")
        );
        assert!(summary.ready_surfaces.contains(&"lessMixinCallCstNodes"));
        assert!(summary.ready_surfaces.contains(&"lessMixinGuardCstNodes"));
        assert!(
            summary
                .ready_surfaces
                .contains(&"importantAnnotationTokenization")
        );
        assert!(
            summary
                .ready_surfaces
                .contains(&"initialDialectStatementNodes")
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

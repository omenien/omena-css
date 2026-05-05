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
use omena_syntax::{StyleDialect, SyntaxKind};

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
            "initialDialectStatementNodes",
            "recoveryBogusSkeleton",
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
        let kind = if self.find_before_recovery(
            SyntaxKind::LeftBrace,
            &[SyntaxKind::Semicolon, SyntaxKind::RightBrace],
        ) {
            SyntaxKind::Rule
        } else {
            SyntaxKind::BogusRule
        };

        self.builder.start_node(kind);
        self.parse_selector_list();
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

    fn parse_pseudo_selector(&mut self, kind: SyntaxKind) {
        self.builder.start_node(kind);
        self.token_current();
        if self.current_kind() == Some(SyntaxKind::Ident) {
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
                Some(SyntaxKind::AtKeyword) => self.parse_at_rule(),
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
        self.builder.start_node(SyntaxKind::FunctionCall);
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
                '+' => self.consume_static(SyntaxKind::Plus, start, 1),
                '-' => self.consume_static(SyntaxKind::Minus, start, 1),
                '*' => self.consume_static(SyntaxKind::Star, start, 1),
                '/' => self.consume_static(SyntaxKind::Slash, start, 1),
                '%' => self.consume_static(SyntaxKind::Percent, start, 1),
                '=' => self.consume_static(SyntaxKind::Equals, start, 1),
                '~' => self.consume_static(SyntaxKind::Tilde, start, 1),
                '|' if self.starts_with("||") => {
                    self.consume_static(SyntaxKind::ColumnCombinator, start, 2)
                }
                '|' => self.consume_static(SyntaxKind::Pipe, start, 1),
                '^' => self.consume_static(SyntaxKind::Caret, start, 1),
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
        "@mixin" => (SyntaxKind::ScssMixinDeclaration, AtRuleBlockKind::Raw),
        "@include" => (SyntaxKind::ScssIncludeRule, AtRuleBlockKind::Raw),
        "@function" => (SyntaxKind::ScssFunctionDeclaration, AtRuleBlockKind::Raw),
        "@return" => (SyntaxKind::ScssReturnRule, AtRuleBlockKind::Raw),
        "@extend" => (SyntaxKind::ScssExtendRule, AtRuleBlockKind::Raw),
        "@if" => (SyntaxKind::ScssControlIf, AtRuleBlockKind::Raw),
        "@else" => (SyntaxKind::ScssControlElse, AtRuleBlockKind::Raw),
        "@each" => (SyntaxKind::ScssControlEach, AtRuleBlockKind::Raw),
        "@for" => (SyntaxKind::ScssControlFor, AtRuleBlockKind::Raw),
        "@while" => (SyntaxKind::ScssControlWhile, AtRuleBlockKind::Raw),
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
    fn structures_css_value_function_calls() {
        let result = parse(".a { width: calc(var(--gap) + 1rem); }", StyleDialect::Css);
        let kinds = node_kinds(&result.syntax());

        assert!(result.errors().is_empty());
        assert!(kinds.contains(&SyntaxKind::Value));
        assert!(kinds.contains(&SyntaxKind::FunctionCall));
        assert!(kinds.contains(&SyntaxKind::FunctionArguments));
        assert!(kinds.contains(&SyntaxKind::BinaryExpression));
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
                .contains(&"initialDialectStatementNodes")
        );
        assert!(summary.ready_surfaces.contains(&"recoveryBogusSkeleton"));
        assert!(summary.not_ready_surfaces.contains(&"productCutover"));
    }

    fn node_kinds(node: &SyntaxNode<SyntaxKind>) -> Vec<SyntaxKind> {
        let mut kinds = vec![node.kind()];
        for child in node.children() {
            kinds.extend(node_kinds(child));
        }
        kinds
    }
}

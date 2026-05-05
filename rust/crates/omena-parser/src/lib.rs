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
    let mut parser = Parser::new(tokens, errors);
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
        ],
        not_ready_surfaces: vec![
            "fullRecursiveDescentGrammar",
            "prattValueParser",
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
    builder: GreenNodeBuilder<'static, 'static, SyntaxKind>,
    errors: Vec<ParseError>,
}

impl<'text> Parser<'text> {
    fn new(tokens: Vec<Token<'text>>, errors: Vec<ParseError>) -> Self {
        Self {
            tokens,
            position: 0,
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
                Some(SyntaxKind::AtKeyword) => self.parse_at_rule(),
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
                Some(_) => self.token_current(),
                None => break,
            }
        }
        self.builder.finish_node();
    }

    fn parse_declaration_list(&mut self) {
        while !self.at_end() {
            self.eat_trivia();
            match self.current_kind() {
                Some(SyntaxKind::RightBrace) | None => break,
                Some(SyntaxKind::AtKeyword) => self.parse_at_rule(),
                Some(SyntaxKind::LeftBrace) => {
                    self.builder.start_node(SyntaxKind::BogusDeclaration);
                    self.token_current();
                    self.builder.finish_node();
                }
                Some(_) => self.parse_declaration(),
            }
        }
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

    fn parse_value_until(&mut self, recovery: &[SyntaxKind]) {
        while !self.at_end() {
            match self.current_kind() {
                Some(kind) if recovery.contains(&kind) => break,
                Some(SyntaxKind::Ident) if self.next_kind() == Some(SyntaxKind::LeftParen) => {
                    self.parse_function_call()
                }
                Some(SyntaxKind::LeftParen) => self.parse_parenthesized_expression(),
                Some(_) => self.token_current(),
                None => break,
            }
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
        self.builder.start_node(SyntaxKind::AtRule);
        while !self.at_end() {
            match self.current_kind() {
                Some(SyntaxKind::Semicolon) => {
                    self.token_current();
                    break;
                }
                Some(SyntaxKind::LeftBrace) => {
                    self.consume_balanced_block();
                    break;
                }
                Some(_) => self.token_current(),
                None => break,
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

    fn current_kind(&self) -> Option<SyntaxKind> {
        self.tokens.get(self.position).map(|token| token.kind)
    }

    fn next_kind(&self) -> Option<SyntaxKind> {
        self.tokens.get(self.position + 1).map(|token| token.kind)
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

        assert_eq!(scss.syntax().kind(), SyntaxKind::Root);
        assert_eq!(less.syntax().kind(), SyntaxKind::Root);
        assert_eq!(less_at_rule.syntax().kind(), SyntaxKind::Root);
        assert!(scss.errors().is_empty());
        assert!(less.errors().is_empty());
        assert!(less_at_rule.errors().is_empty());
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
    fn structures_css_value_function_calls() {
        let result = parse(".a { width: calc(var(--gap) + 1rem); }", StyleDialect::Css);
        let kinds = node_kinds(&result.syntax());

        assert!(result.errors().is_empty());
        assert!(kinds.contains(&SyntaxKind::Value));
        assert!(kinds.contains(&SyntaxKind::FunctionCall));
        assert!(kinds.contains(&SyntaxKind::FunctionArguments));
    }

    #[test]
    fn summarizes_green_field_parser_boundary() {
        let summary = summarize_parser_boundary();

        assert_eq!(summary.product, "omena-parser.boundary");
        assert_eq!(summary.dialect_count, 4);
        assert_eq!(summary.shared_name_kind_count, 8);
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

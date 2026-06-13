use cstree::text::{TextRange, TextSize};
use omena_syntax::{StyleDialect, SyntaxKind};

use crate::{DialectExtension, ParseError, ParseErrorCode, matches_ignore_ascii_case};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct Token<'text> {
    pub(crate) kind: SyntaxKind,
    pub(crate) text: &'text str,
    pub(crate) range: TextRange,
}

pub(crate) struct Tokenizer<'text, 'extension, E> {
    pub(crate) text: &'text str,
    pub(crate) extension: &'extension E,
    pub(crate) offset: usize,
    pub(crate) scss_interpolation_depth: usize,
    pub(crate) less_interpolation_depth: usize,
    pub(crate) sass_indent_stack: Vec<usize>,
    pub(crate) tokens: Vec<Token<'text>>,
    pub(crate) errors: Vec<ParseError>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LexResult {
    tokens: Vec<LexedToken>,
    errors: Vec<ParseError>,
    dialect: StyleDialect,
}

impl LexResult {
    pub(crate) fn new(
        tokens: Vec<LexedToken>,
        errors: Vec<ParseError>,
        dialect: StyleDialect,
    ) -> Self {
        Self {
            tokens,
            errors,
            dialect,
        }
    }

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

pub(crate) fn public_token_text(text: &str) -> String {
    text.chars()
        .map(css_syntax_preprocessed_char)
        .collect::<String>()
}

pub(crate) fn is_name_start(char: char) -> bool {
    let char = css_syntax_preprocessed_char(char);
    char == '_' || char == '-' || char.is_alphabetic() || !char.is_ascii()
}

pub(crate) fn is_name_continue(char: char) -> bool {
    is_name_start(char) || char.is_ascii_digit()
}

pub(crate) fn is_non_printable_code_point(char: char) -> bool {
    let char = css_syntax_preprocessed_char(char);
    matches!(char, '\u{0000}'..='\u{0008}' | '\u{000b}' | '\u{000e}'..='\u{001f}' | '\u{007f}')
}

pub(crate) fn is_custom_property_name_text(text: &str) -> bool {
    let Some(rest) = text.strip_prefix("--") else {
        return false;
    };
    let Some(first) = rest.chars().next() else {
        return false;
    };
    first == '-' || is_name_start(first) || starts_valid_escape_text(rest)
}

pub(crate) fn is_css_at_rule_name(text: &str) -> bool {
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

pub(crate) fn sass_token_can_end_statement(kind: SyntaxKind) -> bool {
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

pub(crate) fn text_range(start: usize, end: usize) -> TextRange {
    TextRange::new(TextSize::from(start as u32), TextSize::from(end as u32))
}

impl<'text, 'extension, E> Tokenizer<'text, 'extension, E>
where
    E: DialectExtension,
{
    pub(crate) fn new(text: &'text str, extension: &'extension E) -> Self {
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

    pub(crate) fn tokenize(&mut self) {
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
}

impl<'text, 'extension, E> Tokenizer<'text, 'extension, E>
where
    E: DialectExtension,
{
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
}

impl<'text, 'extension, E> Tokenizer<'text, 'extension, E>
where
    E: DialectExtension,
{
    pub(crate) fn starts_with(&self, pattern: &str) -> bool {
        self.text[self.offset..].starts_with(pattern)
    }

    pub(crate) fn current_starts_valid_escape(&self) -> bool {
        self.escape_starts_at(self.offset)
    }

    pub(crate) fn current_starts_number(&self) -> bool {
        self.starts_number_at(self.offset)
    }

    pub(crate) fn current_starts_number_exponent(&self) -> bool {
        let Some('e' | 'E') = self.current_char() else {
            return false;
        };
        let exponent_offset = self.offset + 'e'.len_utf8();
        self.char_at(exponent_offset)
            .is_some_and(|char| char.is_ascii_digit())
            || (matches!(self.char_at(exponent_offset), Some('+' | '-'))
                && self.char_after_offset_is_ascii_digit(exponent_offset))
    }

    pub(crate) fn starts_number_at(&self, offset: usize) -> bool {
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

    pub(crate) fn current_starts_ident_sequence(&self) -> bool {
        self.starts_ident_sequence_at(self.offset)
    }

    pub(crate) fn starts_ident_sequence_at(&self, offset: usize) -> bool {
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

    pub(crate) fn escape_starts_at(&self, offset: usize) -> bool {
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

    pub(crate) fn char_at(&self, offset: usize) -> Option<char> {
        self.text.get(offset..)?.chars().next()
    }

    pub(crate) fn char_after_current_is_ascii_digit(&self) -> bool {
        self.char_after_offset_is_ascii_digit(self.offset)
    }

    pub(crate) fn char_after_offset_is_ascii_digit(&self, offset: usize) -> bool {
        let Some(char) = self.char_at(offset) else {
            return false;
        };
        self.char_at(offset + char.len_utf8())
            .is_some_and(|char| char.is_ascii_digit())
    }

    pub(crate) fn starts_with_ascii_keyword(&self, keyword: &str) -> bool {
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

    pub(crate) fn supports_scss_interpolation(&self) -> bool {
        matches!(
            self.extension.dialect(),
            StyleDialect::Scss | StyleDialect::Sass
        )
    }

    pub(crate) fn supports_less_interpolation(&self) -> bool {
        self.extension.dialect() == StyleDialect::Less
    }

    pub(crate) fn starts_less_escaped_string(&self) -> bool {
        self.extension.dialect() == StyleDialect::Less
            && (self.starts_with("~\"") || self.starts_with("~'"))
    }

    pub(crate) fn starts_unicode_range(&self) -> bool {
        let mut chars = self.text[self.offset..].chars();
        matches!(chars.next(), Some('u' | 'U'))
            && chars.next() == Some('+')
            && chars
                .next()
                .is_some_and(|char| char.is_ascii_hexdigit() || char == '?')
    }

    pub(crate) fn current_char(&self) -> Option<char> {
        self.text[self.offset..].chars().next()
    }

    pub(crate) fn next_char_is_hex_digit(&self) -> bool {
        let offset = self.offset + '-'.len_utf8();
        self.text
            .get(offset..)
            .and_then(|tail| tail.chars().next())
            .is_some_and(|char| char.is_ascii_hexdigit())
    }

    pub(crate) fn bump_current(&mut self) {
        if let Some(char) = self.current_char() {
            self.bump_char(char);
        }
    }

    pub(crate) fn bump_char(&mut self, char: char) {
        self.offset += char.len_utf8();
    }
}

fn css_syntax_preprocessed_char(char: char) -> char {
    if char == '\0' { '\u{fffd}' } else { char }
}

fn starts_valid_escape_text(text: &str) -> bool {
    text.starts_with('\\')
        && text['\\'.len_utf8()..]
            .chars()
            .next()
            .is_some_and(|char| !matches!(char, '\n' | '\r' | '\u{000c}'))
}

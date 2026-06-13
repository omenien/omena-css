use cstree::text::TextRange;
use omena_syntax::{StyleDialect, SyntaxKind};

use crate::{ParseError, matches_ignore_ascii_case};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct Token<'text> {
    pub(crate) kind: SyntaxKind,
    pub(crate) text: &'text str,
    pub(crate) range: TextRange,
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

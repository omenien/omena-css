use cstree::text::TextRange;
use omena_syntax::{StyleDialect, SyntaxKind};

use crate::ParseError;

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

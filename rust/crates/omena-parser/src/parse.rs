use cstree::{green::GreenNode, interning::TokenInterner, syntax::SyntaxNode, text::TextRange};
use omena_syntax::{StyleDialect, SyntaxKind};
use std::sync::Arc;

use crate::ParsedCst;

#[derive(Debug, Clone)]
pub struct ParseResult {
    green: GreenNode,
    interner: Option<Arc<TokenInterner>>,
    errors: Vec<ParseError>,
    token_count: usize,
    dialect: StyleDialect,
}

impl ParseResult {
    pub(crate) fn new(
        green: GreenNode,
        interner: Option<Arc<TokenInterner>>,
        errors: Vec<ParseError>,
        token_count: usize,
        dialect: StyleDialect,
    ) -> Self {
        Self {
            green,
            interner,
            errors,
            token_count,
            dialect,
        }
    }
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

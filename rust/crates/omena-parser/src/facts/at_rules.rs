//! Parser facts for at-rule headers.
//!
//! The fact layer records the at-rule kind and source span used by summaries,
//! diagnostics, and transform planning.

use cstree::text::TextRange;
use omena_syntax::{StyleDialect, SyntaxKind};

#[cfg(feature = "internal-oracle")]
use crate::Token;
use crate::{ParseResult, at_rule_spec, scss_at_rule_spec};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParsedAtRuleFact {
    pub name: String,
    pub node_kind: Option<SyntaxKind>,
    pub range: TextRange,
}

#[cfg(feature = "internal-oracle")]
pub(crate) fn collect_at_rule_facts_from_tokens(
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

pub(crate) fn collect_at_rule_facts_from_cst(
    text: &str,
    parsed: &ParseResult,
) -> Vec<ParsedAtRuleFact> {
    parsed
        .syntax()
        .descendants_with_tokens()
        .filter_map(|element| element.into_token())
        .filter(|token| token.kind() == SyntaxKind::AtKeyword)
        .map(|token| at_rule_fact_from_cst_token(text, token.text_range(), parsed.dialect()))
        .collect()
}

fn at_rule_fact_from_cst_token(
    text: &str,
    range: TextRange,
    dialect: StyleDialect,
) -> ParsedAtRuleFact {
    let start = u32::from(range.start()) as usize;
    let end = u32::from(range.end()) as usize;
    let source_text = text.get(start..end).unwrap_or_default();
    let css_spec = at_rule_spec(source_text);
    let node_kind = css_spec
        .or_else(|| match dialect {
            StyleDialect::Scss | StyleDialect::Sass => scss_at_rule_spec(source_text),
            StyleDialect::Css | StyleDialect::Less => None,
        })
        .map(|spec| spec.node_kind);
    let name = if css_spec.is_some() {
        source_text.to_ascii_lowercase()
    } else {
        source_text.to_string()
    };
    ParsedAtRuleFact {
        name,
        node_kind,
        range,
    }
}

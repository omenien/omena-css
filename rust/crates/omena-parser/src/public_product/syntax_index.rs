use cstree::syntax::SyntaxNode;
use omena_syntax::SyntaxKind;

use crate::{ParseResult, ParserByteSpanV0};

#[derive(Debug, Clone, PartialEq, Eq)]
struct DeclarationSyntaxV0 {
    span: ParserByteSpanV0,
    property_name: String,
    value_span: ParserByteSpanV0,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct CssModuleValueSyntaxV0 {
    span: ParserByteSpanV0,
    value_span: Option<ParserByteSpanV0>,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub(super) struct ProductSyntaxIndexV0 {
    css_module_values: Vec<CssModuleValueSyntaxV0>,
    scss_forward_rules: Vec<ParserByteSpanV0>,
    keyframes_rules: Vec<ParserByteSpanV0>,
    declarations: Vec<DeclarationSyntaxV0>,
    sass_parameter_lists: Vec<ParserByteSpanV0>,
}

impl ProductSyntaxIndexV0 {
    pub(super) fn new(source: &str, parsed: &ParseResult) -> Self {
        let mut index = Self::default();
        for node in parsed.syntax().descendants() {
            match node.kind() {
                SyntaxKind::CssModuleExportBlock | SyntaxKind::CssModuleImportBlock => {
                    index.css_module_values.push(CssModuleValueSyntaxV0 {
                        span: node_span(node),
                        value_span: value_span_after_colon(node),
                    });
                }
                SyntaxKind::ScssForwardRule => {
                    index.scss_forward_rules.push(node_span(node));
                }
                SyntaxKind::KeyframesRule => {
                    index.keyframes_rules.push(node_span(node));
                }
                SyntaxKind::Declaration | SyntaxKind::CustomPropertyDeclaration => {
                    if let Some(declaration) = declaration_syntax(source, node) {
                        index.declarations.push(declaration);
                    }
                }
                SyntaxKind::ScssMixinDeclaration | SyntaxKind::ScssFunctionDeclaration => {
                    if let Some(span) = parameter_list_span(node) {
                        index.sass_parameter_lists.push(span);
                    }
                }
                _ => {}
            }
        }
        index
    }

    pub(super) fn css_module_value_span_for_offset(
        &self,
        offset: usize,
    ) -> Option<ParserByteSpanV0> {
        containing_span(
            self.css_module_values
                .iter()
                .map(|definition| definition.span),
            offset,
        )
    }

    pub(super) fn css_module_value_text(&self, source: &str, offset: usize) -> Option<String> {
        self.css_module_values
            .iter()
            .filter(|definition| span_contains_offset(definition.span, offset))
            .min_by_key(|definition| span_len(definition.span))
            .and_then(|definition| definition.value_span)
            .and_then(|span| source.get(span.start..span.end))
            .map(str::trim)
            .map(ToString::to_string)
    }

    pub(super) fn scss_forward_span_for_offset(&self, offset: usize) -> Option<ParserByteSpanV0> {
        containing_span(self.scss_forward_rules.iter().copied(), offset)
    }

    pub(super) fn keyframes_span_for_offset(&self, offset: usize) -> Option<ParserByteSpanV0> {
        containing_span(self.keyframes_rules.iter().copied(), offset)
    }

    pub(super) fn declaration_span_for_offset(&self, offset: usize) -> Option<ParserByteSpanV0> {
        containing_span(
            self.declarations.iter().map(|declaration| declaration.span),
            offset,
        )
    }

    pub(super) fn declaration_property_name_for_offset(&self, offset: usize) -> Option<&str> {
        self.declaration_for_offset(offset)
            .map(|declaration| declaration.property_name.as_str())
    }

    pub(super) fn declaration_value_text(&self, source: &str, offset: usize) -> Option<String> {
        let declaration = self.declaration_for_offset(offset)?;
        source
            .get(declaration.value_span.start..declaration.value_span.end)
            .map(str::trim)
            .map(ToString::to_string)
    }

    pub(super) fn sass_parameter_list_contains(&self, offset: usize) -> bool {
        self.sass_parameter_lists
            .iter()
            .any(|span| span_contains_offset(*span, offset))
    }

    fn declaration_for_offset(&self, offset: usize) -> Option<&DeclarationSyntaxV0> {
        self.declarations
            .iter()
            .filter(|declaration| span_contains_offset(declaration.span, offset))
            .min_by_key(|declaration| span_len(declaration.span))
    }
}

fn parameter_list_span(node: &SyntaxNode<SyntaxKind>) -> Option<ParserByteSpanV0> {
    let mut depth = 0usize;
    let mut start = None;
    for token in node
        .descendants_with_tokens()
        .filter_map(|element| element.into_token())
    {
        let span = byte_span(token.text_range());
        match token.kind() {
            SyntaxKind::LeftParen => {
                depth = depth.saturating_add(1);
                if depth == 1 {
                    start = Some(span.end);
                }
            }
            SyntaxKind::RightParen if depth == 1 => {
                return start.map(|start| ParserByteSpanV0 {
                    start,
                    end: span.start,
                });
            }
            SyntaxKind::RightParen => depth = depth.saturating_sub(1),
            SyntaxKind::LeftBrace if depth == 0 => return None,
            _ => {}
        }
    }
    None
}

fn declaration_syntax(source: &str, node: &SyntaxNode<SyntaxKind>) -> Option<DeclarationSyntaxV0> {
    let span = node_span(node);
    let colon = node
        .descendants_with_tokens()
        .filter_map(|element| element.into_token())
        .find(|token| token.kind() == SyntaxKind::Colon)
        .map(|token| byte_span(token.text_range()))?;
    let value_span = value_span_after_colon(node)?;
    let property_name = source
        .get(span.start..colon.start)?
        .trim()
        .to_ascii_lowercase();
    (!property_name.is_empty()).then_some(DeclarationSyntaxV0 {
        span,
        property_name,
        value_span,
    })
}

fn value_span_after_colon(node: &SyntaxNode<SyntaxKind>) -> Option<ParserByteSpanV0> {
    let mut colon_end = None;
    let mut value_end = None;
    for token in node
        .descendants_with_tokens()
        .filter_map(|element| element.into_token())
    {
        let span = byte_span(token.text_range());
        if colon_end.is_none() && token.kind() == SyntaxKind::Colon {
            colon_end = Some(span.end);
            continue;
        }
        if colon_end.is_some()
            && matches!(
                token.kind(),
                SyntaxKind::Semicolon | SyntaxKind::SassOptionalSemicolon
            )
        {
            value_end = Some(span.start);
            break;
        }
    }
    let start = colon_end?;
    let end = value_end.unwrap_or_else(|| node_span(node).end);
    (start <= end).then_some(ParserByteSpanV0 { start, end })
}

fn containing_span(
    spans: impl Iterator<Item = ParserByteSpanV0>,
    offset: usize,
) -> Option<ParserByteSpanV0> {
    spans
        .filter(|span| span_contains_offset(*span, offset))
        .min_by_key(|span| span_len(*span))
}

fn span_contains_offset(span: ParserByteSpanV0, offset: usize) -> bool {
    span.start <= offset && offset < span.end
}

fn span_len(span: ParserByteSpanV0) -> usize {
    span.end.saturating_sub(span.start)
}

fn node_span(node: &SyntaxNode<SyntaxKind>) -> ParserByteSpanV0 {
    byte_span(node.text_range())
}

fn byte_span(range: cstree::text::TextRange) -> ParserByteSpanV0 {
    ParserByteSpanV0 {
        start: u32::from(range.start()) as usize,
        end: u32::from(range.end()) as usize,
    }
}

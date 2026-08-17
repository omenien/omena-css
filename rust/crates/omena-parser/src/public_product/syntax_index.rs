use cstree::syntax::SyntaxNode;
use omena_syntax::{SyntaxKind, css_keyword};

use crate::{ParseResult, ParserByteSpanV0, StyleDialect, is_at_rule_node_kind, parse};

/// One selector-bearing CST ancestor of a declaration.
///
/// `reset_to_root` is true only for the selector form of Sass `@at-root`.
/// The selector members come from CST-owned rule boundaries rather than a
/// declaration-string scan.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParserDeclarationSelectorContextV0 {
    pub reset_to_root: bool,
    pub selector_members: Vec<String>,
}

/// CST-owned syntax projection for one declaration.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParserDeclarationSyntaxFactV0 {
    pub byte_span: ParserByteSpanV0,
    pub property_name: String,
    pub value_span: ParserByteSpanV0,
    pub important: bool,
    pub selector_contexts: Vec<ParserDeclarationSelectorContextV0>,
    pub condition_contexts: Vec<String>,
    pub source_order: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct CssModuleValueSyntaxV0 {
    span: ParserByteSpanV0,
    value_span: Option<ParserByteSpanV0>,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ProductSyntaxIndexV0 {
    css_module_values: Vec<CssModuleValueSyntaxV0>,
    scss_forward_rules: Vec<ParserByteSpanV0>,
    keyframes_rules: Vec<ParserByteSpanV0>,
    declarations: Vec<ParserDeclarationSyntaxFactV0>,
    sass_parameter_lists: Vec<ParserByteSpanV0>,
}

impl ProductSyntaxIndexV0 {
    pub fn new(source: &str, parsed: &ParseResult) -> Self {
        let mut index = Self::default();
        let mut declaration_source_order = 0usize;
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
                    if let Some(declaration) =
                        declaration_syntax(source, node, declaration_source_order)
                    {
                        index.declarations.push(declaration);
                        declaration_source_order = declaration_source_order.saturating_add(1);
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

    pub fn declarations(&self) -> &[ParserDeclarationSyntaxFactV0] {
        self.declarations.as_slice()
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
            self.declarations
                .iter()
                .map(|declaration| declaration.byte_span),
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

    fn declaration_for_offset(&self, offset: usize) -> Option<&ParserDeclarationSyntaxFactV0> {
        self.declarations
            .iter()
            .filter(|declaration| span_contains_offset(declaration.byte_span, offset))
            .min_by_key(|declaration| span_len(declaration.byte_span))
    }
}

/// Parse a stylesheet and project its declarations through the product syntax
/// index. This is the additive consumer boundary for declaration-oriented
/// products; callers do not need to reconstruct declaration boundaries.
pub fn collect_parser_declaration_syntax_facts(
    source: &str,
    dialect: StyleDialect,
) -> Vec<ParserDeclarationSyntaxFactV0> {
    let parsed = parse(source, dialect);
    ProductSyntaxIndexV0::new(source, &parsed).declarations
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

fn declaration_syntax(
    source: &str,
    node: &SyntaxNode<SyntaxKind>,
    source_order: usize,
) -> Option<ParserDeclarationSyntaxFactV0> {
    let declaration_span = node_span(node);
    let colon = node
        .descendants_with_tokens()
        .filter_map(|element| element.into_token())
        .find(|token| token.kind() == SyntaxKind::Colon)
        .map(|token| byte_span(token.text_range()))?;
    let mut value_span = value_span_after_colon(node)?;
    let important_span = node
        .descendants()
        .find(|child| child.kind() == SyntaxKind::ImportantAnnotation)
        .map(node_span);
    if let Some(important_span) = important_span {
        value_span.end = value_span.end.min(important_span.start);
    }
    let property_name = source
        .get(declaration_span.start..colon.start)?
        .trim()
        .to_ascii_lowercase();
    (!property_name.is_empty()).then_some(ParserDeclarationSyntaxFactV0 {
        byte_span: declaration_span,
        property_name,
        value_span,
        important: important_span.is_some(),
        selector_contexts: declaration_selector_contexts(source, node),
        condition_contexts: declaration_condition_contexts(source, node),
        source_order,
    })
}

fn declaration_selector_contexts(
    source: &str,
    node: &SyntaxNode<SyntaxKind>,
) -> Vec<ParserDeclarationSelectorContextV0> {
    let mut ancestors = node.ancestors().skip(1).collect::<Vec<_>>();
    ancestors.reverse();
    ancestors
        .into_iter()
        .filter_map(|ancestor| match ancestor.kind() {
            SyntaxKind::Rule | SyntaxKind::NestRule => {
                let selector_members = selector_members_for_rule(source, ancestor);
                (!selector_members.is_empty()).then_some(ParserDeclarationSelectorContextV0 {
                    reset_to_root: false,
                    selector_members,
                })
            }
            SyntaxKind::ScssAtRootRule => {
                let selector_members = at_root_selector_members(source, ancestor);
                (!selector_members.is_empty()).then_some(ParserDeclarationSelectorContextV0 {
                    reset_to_root: true,
                    selector_members,
                })
            }
            _ => None,
        })
        .collect()
}

fn selector_members_for_rule(source: &str, node: &SyntaxNode<SyntaxKind>) -> Vec<String> {
    let Some(selector_list) = node.children().find(|child| {
        matches!(
            child.kind(),
            SyntaxKind::SelectorList
                | SyntaxKind::RelativeSelectorList
                | SyntaxKind::BogusSelectorList
        )
    }) else {
        return Vec::new();
    };
    selector_list
        .children()
        .filter(|child| {
            matches!(
                child.kind(),
                SyntaxKind::Selector | SyntaxKind::RelativeSelector | SyntaxKind::BogusSelector
            )
        })
        .filter_map(|selector| source_text_for_node(source, selector))
        .map(|selector| selector.trim().to_string())
        .filter(|selector| !selector.is_empty())
        .collect()
}

fn at_root_selector_members(source: &str, node: &SyntaxNode<SyntaxKind>) -> Vec<String> {
    let Some(header) = block_header_text(source, node) else {
        return Vec::new();
    };
    let Some(rest) = css_keyword(header.trim_start()).strip_prefix("@at-root") else {
        return Vec::new();
    };
    if let Some(next) = rest.chars().next()
        && !next.is_ascii_whitespace()
    {
        return Vec::new();
    }
    let selector = rest.trim();
    if selector.is_empty() || selector.starts_with('(') {
        Vec::new()
    } else {
        vec![selector.to_string()]
    }
}

fn declaration_condition_contexts(source: &str, node: &SyntaxNode<SyntaxKind>) -> Vec<String> {
    let mut ancestors = node.ancestors().skip(1).collect::<Vec<_>>();
    ancestors.reverse();
    let mut contexts = Vec::new();
    for context in ancestors
        .into_iter()
        .filter(|ancestor| {
            is_at_rule_node_kind(ancestor.kind())
                && !matches!(
                    ancestor.kind(),
                    SyntaxKind::LayerRule | SyntaxKind::ScssAtRootRule | SyntaxKind::NestRule
                )
        })
        .filter_map(|ancestor| block_header_text(source, ancestor))
        .map(|header| header.split_whitespace().collect::<Vec<_>>().join(" "))
        .filter(|header| !header.is_empty())
    {
        if contexts.last() != Some(&context) {
            contexts.push(context);
        }
    }
    contexts
}

fn block_header_text<'a>(source: &'a str, node: &SyntaxNode<SyntaxKind>) -> Option<&'a str> {
    let open = node
        .descendants_with_tokens()
        .filter_map(|element| element.into_token())
        .find(|token| token.kind() == SyntaxKind::LeftBrace)
        .map(|token| byte_span(token.text_range()))?;
    source.get(node_span(node).start..open.start)
}

fn source_text_for_node<'a>(source: &'a str, node: &SyntaxNode<SyntaxKind>) -> Option<&'a str> {
    let span = node_span(node);
    source.get(span.start..span.end)
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

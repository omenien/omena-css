use std::collections::HashMap;

use cstree::syntax::SyntaxNode;
use omena_syntax::ident::{
    AuthoredPropertyTextV0, CanonicalPropertyKeyV0, PropertyNameKindV0, PropertyNameV0,
};
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
#[derive(Debug, Clone)]
pub struct ParserDeclarationSyntaxFactV0 {
    pub byte_span: ParserByteSpanV0,
    /// Authored spelling without surrounding declaration trivia.
    pub property_name: AuthoredPropertyTextV0,
    /// The sole identity carrier used by downstream declaration consumers.
    pub property_key: CanonicalPropertyKeyV0,
    pub value_span: ParserByteSpanV0,
    pub value_text: String,
    pub important: bool,
    pub selector_contexts: Vec<ParserDeclarationSelectorContextV0>,
    pub condition_contexts: Vec<String>,
    pub source_order: usize,
}

impl PartialEq for ParserDeclarationSyntaxFactV0 {
    fn eq(&self, other: &Self) -> bool {
        self.byte_span == other.byte_span
            && self.property_key == other.property_key
            && self.value_span == other.value_span
            && self.value_text == other.value_text
            && self.important == other.important
            && self.selector_contexts == other.selector_contexts
            && self.condition_contexts == other.condition_contexts
            && self.source_order == other.source_order
    }
}

impl Eq for ParserDeclarationSyntaxFactV0 {}

#[derive(Default)]
struct DeclarationContextCache {
    selector_contexts: HashMap<(usize, usize), Option<ParserDeclarationSelectorContextV0>>,
    condition_contexts: HashMap<(usize, usize), Option<String>>,
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
        let mut declaration_context_cache = DeclarationContextCache::default();
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
                    if let Some(declaration) = declaration_syntax(
                        source,
                        node,
                        declaration_source_order,
                        &mut declaration_context_cache,
                    ) {
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

    /// Project declarations from an existing parse without building the
    /// unrelated product-index families.
    pub fn declarations_from_parse(
        source: &str,
        parsed: &ParseResult,
    ) -> Vec<ParserDeclarationSyntaxFactV0> {
        let mut declarations = Vec::new();
        let root = parsed.syntax();
        collect_declarations_from_subtree(
            source,
            &root,
            &mut Vec::new(),
            &mut Vec::new(),
            &mut declarations,
        );
        declarations
    }

    /// Consume the index and return its declaration projection without
    /// cloning declaration strings.
    pub fn into_declarations(self) -> Vec<ParserDeclarationSyntaxFactV0> {
        self.declarations
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

    pub(super) fn declaration_property_key_for_offset(
        &self,
        offset: usize,
    ) -> Option<CanonicalPropertyKeyV0> {
        self.declaration_for_offset(offset)
            .map(|declaration| declaration.property_key.clone())
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
    ProductSyntaxIndexV0::declarations_from_parse(source, &parsed)
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
    context_cache: &mut DeclarationContextCache,
) -> Option<ParserDeclarationSyntaxFactV0> {
    let (selector_contexts, condition_contexts) = declaration_contexts(source, node, context_cache);
    declaration_syntax_with_context(
        source,
        node,
        source_order,
        selector_contexts,
        condition_contexts,
    )
}

fn declaration_syntax_with_context(
    source: &str,
    node: &SyntaxNode<SyntaxKind>,
    source_order: usize,
    selector_contexts: Vec<ParserDeclarationSelectorContextV0>,
    condition_contexts: Vec<String>,
) -> Option<ParserDeclarationSyntaxFactV0> {
    let mut declaration_span = node_span(node);
    let declaration_end = declaration_span.end;
    let mut colon = None;
    let mut value_end = None;
    let mut important_start = None;
    let mut property_name = String::new();
    let mut property_is_custom = node.kind() == SyntaxKind::CustomPropertyDeclaration;
    let mut value_text = String::new();
    for token in node
        .descendants_with_tokens()
        .filter_map(|element| element.into_token())
    {
        let token_span = byte_span(token.text_range());
        if token.parent().kind() == SyntaxKind::ImportantAnnotation {
            important_start.get_or_insert(token_span.start);
        }
        if colon.is_none() && token.kind() == SyntaxKind::Colon {
            colon = Some(token_span);
            continue;
        }
        if colon.is_some()
            && value_end.is_none()
            && matches!(
                token.kind(),
                SyntaxKind::Semicolon | SyntaxKind::SassOptionalSemicolon
            )
        {
            value_end = Some(token_span.start);
        }
        if token.kind() == SyntaxKind::Semicolon {
            declaration_span.end = declaration_span.end.min(token_span.start);
        }

        let target = if colon.is_none() {
            property_is_custom |= token.kind() == SyntaxKind::CustomPropertyName;
            Some(&mut property_name)
        } else if value_end.is_none() && important_start.is_none() {
            Some(&mut value_text)
        } else {
            None
        };
        if let Some(target) = target {
            append_normalized_declaration_token(source, token.kind(), token_span, target);
        }
    }
    let colon = colon?;
    let value_span = ParserByteSpanV0 {
        start: colon.end,
        end: value_end
            .unwrap_or(declaration_end)
            .min(important_start.unwrap_or(usize::MAX)),
    };
    let property = PropertyNameV0::new(
        property_name,
        if property_is_custom {
            PropertyNameKindV0::Custom
        } else {
            PropertyNameKindV0::Standard
        },
    );
    let property_name = property.authored_text();
    let property_key = property.canonical_key();
    let value_text = value_text.trim().to_string();
    (!property_name.is_empty()).then_some(ParserDeclarationSyntaxFactV0 {
        byte_span: declaration_span,
        property_name,
        property_key,
        value_span,
        value_text,
        important: important_start.is_some(),
        selector_contexts,
        condition_contexts,
        source_order,
    })
}

fn collect_declarations_from_subtree(
    source: &str,
    node: &SyntaxNode<SyntaxKind>,
    selector_contexts: &mut Vec<ParserDeclarationSelectorContextV0>,
    condition_contexts: &mut Vec<String>,
    declarations: &mut Vec<ParserDeclarationSyntaxFactV0>,
) {
    if matches!(
        node.kind(),
        SyntaxKind::Declaration | SyntaxKind::CustomPropertyDeclaration
    ) {
        if let Some(declaration) = declaration_syntax_with_context(
            source,
            node,
            declarations.len(),
            selector_contexts.clone(),
            condition_contexts.clone(),
        ) {
            declarations.push(declaration);
        }
        return;
    }

    let selector_pushed = selector_context_for_node(source, node).is_some_and(|context| {
        selector_contexts.push(context);
        true
    });
    let condition_context = condition_context_for_node(source, node);
    let condition_pushed = condition_context
        .filter(|context| condition_contexts.last() != Some(context))
        .is_some_and(|context| {
            condition_contexts.push(context);
            true
        });

    for child in node.children() {
        collect_declarations_from_subtree(
            source,
            child,
            selector_contexts,
            condition_contexts,
            declarations,
        );
    }

    if condition_pushed {
        condition_contexts.pop();
    }
    if selector_pushed {
        selector_contexts.pop();
    }
}

fn selector_context_for_node(
    source: &str,
    node: &SyntaxNode<SyntaxKind>,
) -> Option<ParserDeclarationSelectorContextV0> {
    let (reset_to_root, selector_members) = match node.kind() {
        SyntaxKind::Rule | SyntaxKind::NestRule => (false, selector_members_for_rule(source, node)),
        SyntaxKind::ScssAtRootRule => (true, at_root_selector_members(source, node)),
        _ => return None,
    };
    (!selector_members.is_empty()).then_some(ParserDeclarationSelectorContextV0 {
        reset_to_root,
        selector_members,
    })
}

fn condition_context_for_node(source: &str, node: &SyntaxNode<SyntaxKind>) -> Option<String> {
    if !is_at_rule_node_kind(node.kind())
        || matches!(
            node.kind(),
            SyntaxKind::LayerRule | SyntaxKind::ScssAtRootRule | SyntaxKind::NestRule
        )
    {
        return None;
    }
    block_header_text(source, node)
        .map(|header| header.split_whitespace().collect::<Vec<_>>().join(" "))
        .filter(|header| !is_non_condition_wrapper_header(header))
        .filter(|header| !header.is_empty())
}

fn declaration_contexts(
    source: &str,
    node: &SyntaxNode<SyntaxKind>,
    cache: &mut DeclarationContextCache,
) -> (Vec<ParserDeclarationSelectorContextV0>, Vec<String>) {
    let mut ancestors = node.ancestors().skip(1).collect::<Vec<_>>();
    ancestors.reverse();
    let mut selector_contexts = Vec::new();
    let mut condition_contexts = Vec::new();
    for ancestor in ancestors {
        let span = node_span(ancestor);
        let key = (span.start, span.end);
        if matches!(
            ancestor.kind(),
            SyntaxKind::Rule | SyntaxKind::NestRule | SyntaxKind::ScssAtRootRule
        ) {
            let context = cache
                .selector_contexts
                .entry(key)
                .or_insert_with(|| match ancestor.kind() {
                    SyntaxKind::Rule | SyntaxKind::NestRule => {
                        let selector_members = selector_members_for_rule(source, ancestor);
                        (!selector_members.is_empty()).then_some(
                            ParserDeclarationSelectorContextV0 {
                                reset_to_root: false,
                                selector_members,
                            },
                        )
                    }
                    SyntaxKind::ScssAtRootRule => {
                        let selector_members = at_root_selector_members(source, ancestor);
                        (!selector_members.is_empty()).then_some(
                            ParserDeclarationSelectorContextV0 {
                                reset_to_root: true,
                                selector_members,
                            },
                        )
                    }
                    _ => None,
                })
                .clone();
            if let Some(context) = context {
                selector_contexts.push(context);
            }
        }
        if is_at_rule_node_kind(ancestor.kind())
            && !matches!(
                ancestor.kind(),
                SyntaxKind::LayerRule | SyntaxKind::ScssAtRootRule | SyntaxKind::NestRule
            )
        {
            let context = cache
                .condition_contexts
                .entry(key)
                .or_insert_with(|| {
                    block_header_text(source, ancestor)
                        .map(|header| header.split_whitespace().collect::<Vec<_>>().join(" "))
                        .filter(|header| !is_non_condition_wrapper_header(header))
                        .filter(|header| !header.is_empty())
                })
                .clone();
            if let Some(context) = context
                && condition_contexts.last() != Some(&context)
            {
                condition_contexts.push(context);
            }
        }
    }
    (selector_contexts, condition_contexts)
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

fn is_non_condition_wrapper_header(header: &str) -> bool {
    ["@layer", "@at-root", "@nest"]
        .into_iter()
        .any(|keyword| at_rule_header_has_keyword(header, keyword))
}

fn at_rule_header_has_keyword(header: &str, keyword: &str) -> bool {
    let Some(rest) = css_keyword(header.trim_start()).strip_prefix(keyword) else {
        return false;
    };
    rest.is_empty() || rest.chars().next().is_some_and(char::is_whitespace)
}

fn append_normalized_declaration_token(
    source: &str,
    kind: SyntaxKind,
    span: ParserByteSpanV0,
    target: &mut String,
) {
    if matches!(
        kind,
        SyntaxKind::BlockComment | SyntaxKind::LineComment | SyntaxKind::ScssSilentComment
    ) {
        if !target.ends_with(char::is_whitespace) {
            target.push(' ');
        }
    } else if let Some(token_text) = source.get(span.start..span.end) {
        target.push_str(token_text);
    }
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

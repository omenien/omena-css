//! Recursive-descent parser entry points and result types.
//!
//! This module owns the concrete parser loop while exporting stable parse,
//! lex, and fact-collection functions for the crate's public API.

use cstree::{
    build::{GreenNodeBuilder, NodeCache},
    green::GreenNode,
    interning::TokenInterner,
    syntax::SyntaxNode,
    text::{TextRange, TextSize},
};
use omena_syntax::{StyleDialect, SyntaxKind};
use std::sync::{Arc, OnceLock};

use crate::extension::{AtRuleBlockKind, AtRuleSpec, at_rule_spec, scss_at_rule_spec};
use crate::facts::collect_style_facts_with_extension;
use crate::{
    BuiltinDialectExtension, DialectExtension, LexResult, LexedToken, ParsedCst, ParsedStyleFacts,
    Token, Tokenizer, UNARY_PREFIX_RIGHT_BINDING_POWER, at_rule_prelude_head_is_custom_ident,
    at_rule_prelude_head_is_custom_property_name, attribute_name_token_can_continue,
    attribute_name_token_can_start, attribute_value_token_can_start, bracketed_value_recovery,
    comma_separated_component_value_list_item_recovery, css_module_block_scope_marker_in_header,
    css_module_header_is_global_only, css_module_scope_function_kind,
    function_argument_count_is_valid, function_argument_recovery,
    function_requires_filled_top_level_arguments, infix_binding_power, interpolation_end_kind,
    is_at_rule_prelude_boundary, is_attribute_matcher, is_combinator,
    is_component_value_atom_start, is_css_module_from_source_token,
    is_dynamic_function_argument_head, is_interpolation_start, is_nth_pseudo_class,
    is_scss_control_rule_kind, is_scss_module_namespace_token, is_scss_module_source_token,
    is_scss_module_visibility_name_token, is_selector_boundary, is_selector_boundary_until,
    is_selector_list_pseudo_class, is_statement_end, keyframe_selector_token_is_valid,
    language_tag_token_can_start, matches_ignore_ascii_case, matching_simple_block_close,
    namespace_selector_target_can_start, public_token_text, selector_component_can_start,
    selector_item_token_is_recoverable, simple_block_recovery, specialized_function_kind,
    value_list_item_recovery, variable_declaration_node_kind,
};

#[derive(Debug)]
pub struct ParseResult {
    green: GreenNode,
    interner: Option<Arc<TokenInterner>>,
    errors: Vec<ParseError>,
    token_count: usize,
    dialect: StyleDialect,
    syntax_root: OnceLock<SyntaxNode<SyntaxKind>>,
    syntax_tokens: OnceLock<Vec<SyntaxTokenView>>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct SyntaxTokenView {
    pub(crate) kind: SyntaxKind,
    pub(crate) range: TextRange,
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
            syntax_root: OnceLock::new(),
            syntax_tokens: OnceLock::new(),
        }
    }

    fn materialize_syntax_root(&self) -> SyntaxNode<SyntaxKind> {
        crate::record_omena_parser_syntax_root_materialization();
        if let Some(interner) = &self.interner {
            return SyntaxNode::new_root_with_resolver(self.green.clone(), Arc::clone(interner))
                .syntax()
                .clone();
        }
        SyntaxNode::new_root(self.green.clone())
    }
}

impl Clone for ParseResult {
    fn clone(&self) -> Self {
        let syntax_root = OnceLock::new();
        if let Some(root) = self.syntax_root.get() {
            let _ = syntax_root.set(root.clone());
        }
        let syntax_tokens = OnceLock::new();
        if let Some(tokens) = self.syntax_tokens.get() {
            let _ = syntax_tokens.set(tokens.clone());
        }
        Self {
            green: self.green.clone(),
            interner: self.interner.clone(),
            errors: self.errors.clone(),
            token_count: self.token_count,
            dialect: self.dialect,
            syntax_root,
            syntax_tokens,
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
        self.syntax_root
            .get_or_init(|| self.materialize_syntax_root())
            .clone()
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

    pub(crate) fn syntax_token_views(&self) -> &[SyntaxTokenView] {
        self.syntax_tokens
            .get_or_init(|| {
                self.syntax()
                    .descendants_with_tokens()
                    .filter_map(|element| element.into_token())
                    .map(|token| SyntaxTokenView {
                        kind: token.kind(),
                        range: token.text_range(),
                    })
                    .collect()
            })
            .as_slice()
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

#[derive(Debug, Default)]
pub struct ParseReuseCache {
    node_cache: NodeCache<'static>,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct SyntaxNodeId {
    value: String,
}

impl SyntaxNodeId {
    pub fn as_str(&self) -> &str {
        self.value.as_str()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct HirId {
    value: String,
}

impl HirId {
    pub fn as_str(&self) -> &str {
        self.value.as_str()
    }
}

pub fn syntax_node_id(node: &SyntaxNode<SyntaxKind>) -> SyntaxNodeId {
    let path = syntax_node_child_path(node)
        .into_iter()
        .map(|index| index.to_string())
        .collect::<Vec<_>>()
        .join(".");
    let text = node
        .try_resolved()
        .map(|resolved| resolved.text().to_string())
        .unwrap_or_default();
    let text_hash = stable_parser_identity_hash(text.as_bytes());
    SyntaxNodeId {
        value: format!(
            "syntax:v0:kind={}:path={}:len={}:text={text_hash:016x}",
            node.kind().as_u32(),
            path,
            u32::from(node.text_range().len())
        ),
    }
}

pub fn hir_id_for_syntax_node(node: &SyntaxNode<SyntaxKind>) -> HirId {
    let syntax_id = syntax_node_id(node);
    HirId {
        value: format!("hir:v0:{}", syntax_id.as_str()),
    }
}

pub fn parse(text: &str, dialect: StyleDialect) -> ParseResult {
    parse_entry_point(text, dialect, ParseEntryPoint::Stylesheet)
}

/// Parses a stylesheet without collecting parser facts.
pub fn parse_only(text: &str, dialect: StyleDialect) -> ParseResult {
    parse(text, dialect)
}

pub fn parse_entry_point(
    text: &str,
    dialect: StyleDialect,
    entry_point: ParseEntryPoint,
) -> ParseResult {
    let extension = BuiltinDialectExtension::new(dialect);
    parse_entry_point_with_extension(text, &extension, entry_point)
}

pub fn lex(text: &str, dialect: StyleDialect) -> LexResult {
    let extension = BuiltinDialectExtension::new(dialect);
    lex_with_extension(text, &extension)
}

pub fn lex_with_extension(text: &str, extension: &impl DialectExtension) -> LexResult {
    let (tokens, errors) = tokenize(text, extension);
    let token_count = tokens.len();
    crate::record_omena_parser_lex_materialization(token_count);
    LexResult::new(
        tokens
            .into_iter()
            .map(|token| LexedToken {
                kind: token.kind,
                range: token.range,
                text: public_token_text(token.text),
            })
            .collect(),
        errors,
        extension.dialect(),
    )
}

pub fn parse_with_extension(text: &str, extension: &impl DialectExtension) -> ParseResult {
    parse_entry_point_with_extension(text, extension, ParseEntryPoint::Stylesheet)
}

pub fn parse_entry_point_with_extension(
    text: &str,
    extension: &impl DialectExtension,
    entry_point: ParseEntryPoint,
) -> ParseResult {
    let (tokens, errors) = tokenize(text, extension);
    let token_count = tokens.len();
    let mut parser = Parser::new(tokens, errors, extension.dialect());
    crate::record_omena_parser_parse_materialization(token_count);
    let (green, interner) = parser.parse_entry_point(entry_point);

    ParseResult::new(
        green,
        interner,
        parser.into_errors(),
        token_count,
        extension.dialect(),
    )
}

pub fn parse_with_reuse_cache(
    text: &str,
    dialect: StyleDialect,
    cache: &mut ParseReuseCache,
) -> ParseResult {
    parse_entry_point_with_reuse_cache(text, dialect, ParseEntryPoint::Stylesheet, cache)
}

pub fn parse_entry_point_with_reuse_cache(
    text: &str,
    dialect: StyleDialect,
    entry_point: ParseEntryPoint,
    cache: &mut ParseReuseCache,
) -> ParseResult {
    let extension = BuiltinDialectExtension::new(dialect);
    parse_entry_point_with_extension_and_reuse_cache(text, &extension, entry_point, cache)
}

pub fn parse_entry_point_with_extension_and_reuse_cache(
    text: &str,
    extension: &impl DialectExtension,
    entry_point: ParseEntryPoint,
    cache: &mut ParseReuseCache,
) -> ParseResult {
    let (tokens, errors) = tokenize(text, extension);
    let token_count = tokens.len();
    let node_cache = std::mem::take(&mut cache.node_cache);
    let mut parser = Parser::new_with_node_cache(tokens, errors, extension.dialect(), node_cache);
    crate::record_omena_parser_parse_materialization(token_count);
    let (green, node_cache) = parser.parse_entry_point_reusing_cache(entry_point);
    cache.node_cache = node_cache.unwrap_or_default();

    ParseResult::new(
        green,
        None,
        parser.into_errors(),
        token_count,
        extension.dialect(),
    )
}

pub fn collect_style_facts(text: &str, dialect: StyleDialect) -> ParsedStyleFacts {
    let extension = BuiltinDialectExtension::new(dialect);
    collect_style_facts_with_extension(text, &extension)
}

pub(crate) fn tokenize<'text>(
    text: &'text str,
    extension: &impl DialectExtension,
) -> (Vec<Token<'text>>, Vec<ParseError>) {
    let mut tokenizer = Tokenizer::new(text, extension);
    tokenizer.tokenize();
    (tokenizer.tokens, tokenizer.errors)
}

fn syntax_node_child_path(node: &SyntaxNode<SyntaxKind>) -> Vec<usize> {
    let mut ancestors = node.ancestors().collect::<Vec<_>>();
    ancestors.reverse();
    ancestors
        .windows(2)
        .map(|pair| {
            let parent = pair[0];
            let child = pair[1];
            parent
                .children()
                .position(|candidate| candidate == child)
                .unwrap_or(0)
        })
        .collect()
}

fn stable_parser_identity_hash(bytes: &[u8]) -> u64 {
    const FNV_OFFSET: u64 = 0xcbf29ce484222325;
    const FNV_PRIME: u64 = 0x00000100000001b3;

    bytes.iter().fold(FNV_OFFSET, |hash, byte| {
        (hash ^ u64::from(*byte)).wrapping_mul(FNV_PRIME)
    })
}

pub(crate) struct Parser<'text> {
    tokens: Vec<Token<'text>>,
    position: usize,
    dialect: StyleDialect,
    builder: GreenNodeBuilder<'static, 'static, SyntaxKind>,
    errors: Vec<ParseError>,
}

impl<'text> Parser<'text> {
    pub(crate) fn new(
        tokens: Vec<Token<'text>>,
        errors: Vec<ParseError>,
        dialect: StyleDialect,
    ) -> Self {
        Self::new_with_node_cache(tokens, errors, dialect, NodeCache::new())
    }

    pub(crate) fn new_with_node_cache(
        tokens: Vec<Token<'text>>,
        errors: Vec<ParseError>,
        dialect: StyleDialect,
        node_cache: NodeCache<'static>,
    ) -> Self {
        Self {
            tokens,
            position: 0,
            dialect,
            builder: GreenNodeBuilder::from_cache(node_cache),
            errors,
        }
    }

    pub(crate) fn parse(&mut self) -> (GreenNode, Option<Arc<TokenInterner>>) {
        self.parse_entry_point(ParseEntryPoint::Stylesheet)
    }

    fn parse_entry_point(
        &mut self,
        entry_point: ParseEntryPoint,
    ) -> (GreenNode, Option<Arc<TokenInterner>>) {
        let (green, cache) = self.parse_entry_point_reusing_cache(entry_point);
        let interner = cache.and_then(|cache| cache.into_interner()).map(Arc::new);
        (green, interner)
    }

    fn parse_entry_point_reusing_cache(
        &mut self,
        entry_point: ParseEntryPoint,
    ) -> (GreenNode, Option<NodeCache<'static>>) {
        self.builder.start_node(SyntaxKind::Root);
        match entry_point {
            ParseEntryPoint::Stylesheet => {
                self.builder.start_node(SyntaxKind::Stylesheet);
                self.parse_stylesheet_items();
                self.builder.finish_node();
            }
            ParseEntryPoint::RuleList => {
                self.builder.start_node(SyntaxKind::RuleList);
                self.parse_rule_list_items();
                self.builder.finish_node();
            }
            ParseEntryPoint::Rule => self.parse_rule(),
            ParseEntryPoint::DeclarationList => {
                self.builder.start_node(SyntaxKind::DeclarationList);
                self.parse_declaration_list();
                self.builder.finish_node();
            }
            ParseEntryPoint::Declaration => self.parse_declaration(),
            ParseEntryPoint::Value => {
                self.builder.start_node(SyntaxKind::Value);
                self.parse_value_or_value_list_until(&[]);
                self.builder.finish_node();
            }
            ParseEntryPoint::ComponentValue => self.parse_component_value(&[]),
            ParseEntryPoint::ComponentValueList => self.parse_component_value_list_until(&[]),
            ParseEntryPoint::CommaSeparatedComponentValueList => {
                self.parse_comma_separated_component_value_list_until(&[])
            }
            ParseEntryPoint::SimpleBlock => self.parse_simple_block_entry_point(&[]),
        }
        self.parse_sass_indentation_bogus();
        self.parse_entry_point_trailing_bogus();
        self.builder.finish_node();

        let builder = std::mem::take(&mut self.builder);
        builder.finish()
    }

    fn parse_sass_indentation_bogus(&mut self) {
        if self.dialect != StyleDialect::Sass
            || !self
                .errors
                .iter()
                .any(|error| error.message == "inconsistent Sass indentation")
        {
            return;
        }
        self.builder.start_node(SyntaxKind::BogusSassIndentation);
        self.builder.finish_node();
    }

    fn parse_entry_point_trailing_bogus(&mut self) {
        self.eat_trivia();
        if self.at_end() {
            return;
        }
        self.builder.start_node(SyntaxKind::BogusRecovery);
        while !self.at_end() {
            self.token_current();
        }
        self.builder.finish_node();
    }

    pub(crate) fn into_errors(self) -> Vec<ParseError> {
        self.errors
    }

    fn parse_stylesheet_items(&mut self) {
        while !self.at_end() {
            self.eat_trivia();
            if self.at_end() {
                break;
            }
            match self.current_kind() {
                Some(SyntaxKind::AtKeyword) if self.current_is_css_module_value_rule() => {
                    self.parse_css_module_value_rule()
                }
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
                Some(SyntaxKind::Cdo | SyntaxKind::Cdc) => self.token_current(),
                Some(SyntaxKind::RightBrace | SyntaxKind::SassDedent) => self.token_current(),
                Some(SyntaxKind::Semicolon | SyntaxKind::SassOptionalSemicolon) => {
                    self.token_current()
                }
                Some(_) => self.parse_rule(),
                None => break,
            }
        }
    }

    fn parse_rule(&mut self) {
        let starts_less_mixin =
            self.dialect == StyleDialect::Less && self.current_starts_less_callable_signature();
        let has_rule_block = self.find_rule_block_open_before_recovery(&[
            SyntaxKind::Semicolon,
            SyntaxKind::SassOptionalSemicolon,
            SyntaxKind::RightBrace,
            SyntaxKind::SassDedent,
        ]);
        let kind = if let Some(kind) = self
            .current_icss_module_rule_kind()
            .filter(|_| has_rule_block)
        {
            kind
        } else if self.current_starts_less_mixin_declaration() {
            SyntaxKind::LessMixinDeclaration
        } else if starts_less_mixin {
            SyntaxKind::BogusLessMixin
        } else if has_rule_block {
            SyntaxKind::Rule
        } else {
            SyntaxKind::BogusRule
        };

        self.builder.start_node(kind);
        if kind == SyntaxKind::CssModuleImportBlock && !self.current_icss_import_has_source() {
            self.error_at_current(ParseErrorCode::ExpectedValue, "expected ICSS import source");
        }
        if kind == SyntaxKind::LessMixinDeclaration {
            self.parse_less_mixin_header();
        } else if kind == SyntaxKind::BogusLessMixin {
            self.parse_until_recovery_with_optional_less_guard(&[
                SyntaxKind::Semicolon,
                SyntaxKind::RightBrace,
                SyntaxKind::SassDedent,
            ]);
            self.error_at_current(
                ParseErrorCode::UnexpectedCharacter,
                "expected Less mixin block",
            );
        } else {
            self.parse_selector_list();
        }
        if self.current_kind() == Some(SyntaxKind::LeftBrace) {
            self.token_current();
            self.builder
                .start_node(if self.previous_left_brace_has_match() {
                    SyntaxKind::DeclarationList
                } else {
                    SyntaxKind::BogusDeclarationList
                });
            self.parse_declaration_list();
            self.builder.finish_node();
            if self.current_kind() == Some(SyntaxKind::RightBrace) {
                self.token_current();
            } else {
                self.missing_token_bogus_trivia(
                    ParseErrorCode::UnexpectedCharacter,
                    "unterminated declaration block",
                );
            }
        } else if self.current_kind() == Some(SyntaxKind::SassIndent) {
            self.builder.start_node(SyntaxKind::SassIndentedBlock);
            self.token_current();
            self.builder.start_node(SyntaxKind::DeclarationList);
            self.parse_declaration_list();
            self.builder.finish_node();
            if self.current_kind() == Some(SyntaxKind::SassDedent) {
                self.token_current();
            } else {
                self.missing_token_bogus_trivia(
                    ParseErrorCode::UnexpectedCharacter,
                    "unterminated Sass indented declaration block",
                );
            }
            self.builder.finish_node();
        } else {
            self.consume_until_recovery(&[
                SyntaxKind::Semicolon,
                SyntaxKind::SassOptionalSemicolon,
                SyntaxKind::RightBrace,
                SyntaxKind::SassDedent,
            ]);
            if self.current_kind().is_some_and(is_statement_end) {
                self.token_current();
            }
        }
        self.builder.finish_node();
    }

    fn current_icss_module_rule_kind(&self) -> Option<SyntaxKind> {
        if self.current_kind() != Some(SyntaxKind::Colon) {
            return None;
        }
        let (name_index, name_kind) = self.non_trivia_token_from(self.position + 1)?;
        if name_kind != SyntaxKind::Ident {
            return None;
        }
        match self.tokens.get(name_index)?.text {
            "export" => Some(SyntaxKind::CssModuleExportBlock),
            "import" => Some(SyntaxKind::CssModuleImportBlock),
            _ => None,
        }
    }

    fn current_icss_import_has_source(&self) -> bool {
        let Some((name_index, SyntaxKind::Ident)) = self.non_trivia_token_from(self.position + 1)
        else {
            return false;
        };
        if self
            .tokens
            .get(name_index)
            .is_none_or(|token| token.text != "import")
        {
            return false;
        }
        let Some((open_index, SyntaxKind::LeftParen)) = self.non_trivia_token_from(name_index + 1)
        else {
            return false;
        };
        let Some((_, source_kind)) = self.non_trivia_token_from(open_index + 1) else {
            return false;
        };
        matches!(
            source_kind,
            SyntaxKind::String | SyntaxKind::Url | SyntaxKind::ScssInterpolationStart
        )
    }

    fn parse_selector_list(&mut self) {
        self.parse_selector_list_until(&[]);
    }

    fn parse_selector_list_until(&mut self, recovery: &[SyntaxKind]) {
        let kind = if self.current_kind() == Some(SyntaxKind::LeftBrace) {
            SyntaxKind::BogusSelectorList
        } else {
            SyntaxKind::SelectorList
        };
        self.builder.start_node(kind);
        while !self.at_end() {
            match self.current_kind() {
                Some(SyntaxKind::Comma) => self.token_current(),
                Some(kind) if is_selector_boundary_until(kind, recovery) => break,
                Some(SyntaxKind::SassIndentedNewline) => self.token_current(),
                Some(_)
                    if recovery.contains(&SyntaxKind::RightParen)
                        && self.current_selector_item_is_bogus(recovery) =>
                {
                    self.parse_bogus_selector_until(recovery)
                }
                Some(_) => self.parse_selector_until(recovery),
                None => break,
            }
        }
        self.builder.finish_node();
    }

    fn parse_strict_selector_list_until(&mut self, recovery: &[SyntaxKind]) {
        self.builder.start_node(
            if self.selector_list_contains_bogus_item_until(recovery)
                && self.current_kind() != Some(SyntaxKind::RightParen)
            {
                SyntaxKind::BogusSelectorList
            } else {
                SyntaxKind::SelectorList
            },
        );
        while !self.at_end() {
            match self.current_kind() {
                Some(SyntaxKind::Comma) => self.token_current(),
                Some(kind) if is_selector_boundary_until(kind, recovery) => break,
                Some(SyntaxKind::SassIndentedNewline) => self.token_current(),
                Some(_)
                    if self.current_selector_item_is_bogus(recovery)
                        && self.current_kind() != Some(SyntaxKind::RightParen) =>
                {
                    self.parse_bogus_selector_until(recovery)
                }
                Some(_) => self.parse_selector_until(recovery),
                None => break,
            }
        }
        self.builder.finish_node();
    }

    fn parse_relative_selector_list_until(&mut self, recovery: &[SyntaxKind]) {
        self.builder.start_node(
            if self.current_selector_item_is_bogus(recovery)
                && self.current_kind() != Some(SyntaxKind::RightParen)
            {
                SyntaxKind::BogusSelectorList
            } else {
                SyntaxKind::RelativeSelectorList
            },
        );
        while !self.at_end() {
            match self.current_kind() {
                Some(SyntaxKind::Comma) => self.token_current(),
                Some(kind) if is_selector_boundary_until(kind, recovery) => break,
                Some(SyntaxKind::SassIndentedNewline) => self.token_current(),
                Some(_)
                    if self.current_selector_item_is_bogus(recovery)
                        && self.current_kind() != Some(SyntaxKind::RightParen) =>
                {
                    self.parse_bogus_selector_until(recovery)
                }
                Some(_) => self.parse_relative_selector_until(recovery),
                None => break,
            }
        }
        self.builder.finish_node();
    }

    fn parse_relative_selector_until(&mut self, recovery: &[SyntaxKind]) {
        self.builder.start_node(SyntaxKind::RelativeSelector);
        self.builder.start_node(SyntaxKind::ComplexSelector);
        self.parse_complex_selector_until(recovery);
        self.builder.finish_node();
        self.builder.finish_node();
    }

    fn parse_bogus_selector_until(&mut self, recovery: &[SyntaxKind]) {
        self.builder.start_node(SyntaxKind::BogusSelector);
        self.error_at_current(
            ParseErrorCode::UnexpectedCharacter,
            "invalid selector in selector list",
        );
        let mut paren_depth = 0usize;
        let mut bracket_depth = 0usize;
        while !self.at_end() {
            let Some(kind) = self.current_kind() else {
                break;
            };
            if paren_depth == 0
                && bracket_depth == 0
                && (kind == SyntaxKind::Comma || is_selector_boundary_until(kind, recovery))
            {
                break;
            }
            match kind {
                SyntaxKind::LeftParen => paren_depth += 1,
                SyntaxKind::RightParen => paren_depth = paren_depth.saturating_sub(1),
                SyntaxKind::LeftBracket => bracket_depth += 1,
                SyntaxKind::RightBracket => bracket_depth = bracket_depth.saturating_sub(1),
                _ => {}
            }
            self.token_current();
        }
        self.builder.finish_node();
    }

    fn parse_selector_until(&mut self, recovery: &[SyntaxKind]) {
        self.builder.start_node(SyntaxKind::Selector);
        self.builder.start_node(SyntaxKind::ComplexSelector);
        self.parse_complex_selector_until(recovery);
        self.builder.finish_node();
        self.builder.finish_node();
    }

    fn parse_complex_selector_until(&mut self, recovery: &[SyntaxKind]) {
        let mut has_component = false;
        while !self.at_end() {
            match self.current_kind() {
                Some(kind) if is_selector_boundary_until(kind, recovery) => break,
                Some(SyntaxKind::Whitespace) => {
                    if has_component
                        && self.next_non_trivia_kind().is_some_and(|kind| {
                            !is_selector_boundary_until(kind, recovery) && !is_combinator(kind)
                        })
                    {
                        self.parse_whitespace_combinator();
                        has_component = false;
                    } else {
                        self.token_current();
                    }
                }
                Some(SyntaxKind::SassIndentedNewline) => self.token_current(),
                Some(kind) if is_combinator(kind) => {
                    self.parse_combinator();
                    has_component = false;
                }
                Some(_) => {
                    self.parse_compound_selector_until(recovery);
                    has_component = true;
                }
                None => break,
            }
        }
    }

    fn parse_compound_selector_until(&mut self, recovery: &[SyntaxKind]) {
        let starts_valid = self.current_kind().is_some_and(|kind| {
            selector_component_can_start(kind)
                || self.current_starts_namespace_qualified_selector(kind)
                || is_interpolation_start(kind)
        });
        self.builder.start_node(if starts_valid {
            SyntaxKind::CompoundSelector
        } else {
            SyntaxKind::BogusCompoundSelector
        });
        let start = self.position;
        while !self.at_end() {
            match self.current_kind() {
                Some(kind)
                    if is_selector_boundary_until(kind, recovery)
                        || kind == SyntaxKind::Whitespace
                        || kind == SyntaxKind::SassIndentedNewline
                        || is_combinator(kind) =>
                {
                    break;
                }
                Some(SyntaxKind::Dot) => self.parse_class_selector(),
                Some(SyntaxKind::Hash) => self.parse_id_selector(),
                Some(kind) if self.current_starts_namespace_qualified_selector(kind) => {
                    self.parse_namespace_qualified_selector()
                }
                Some(SyntaxKind::Ident) => self.parse_type_selector(),
                Some(SyntaxKind::Star) => self.parse_universal_selector(),
                Some(SyntaxKind::Ampersand) => self.parse_nesting_selector(),
                Some(SyntaxKind::ScssPlaceholder) => self.parse_scss_placeholder_selector(),
                Some(kind) if is_interpolation_start(kind) => self.parse_interpolation(
                    kind,
                    &[
                        SyntaxKind::Comma,
                        SyntaxKind::LeftBrace,
                        SyntaxKind::SassIndent,
                        SyntaxKind::RightBrace,
                        SyntaxKind::SassDedent,
                        SyntaxKind::RightParen,
                        SyntaxKind::Semicolon,
                        SyntaxKind::SassOptionalSemicolon,
                    ],
                ),
                Some(SyntaxKind::LeftBracket) => self.parse_attribute_selector(),
                Some(SyntaxKind::Colon) if self.current_starts_less_extend_rule() => {
                    self.parse_less_extend_rule()
                }
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
        if !starts_valid {
            self.error_at_current(
                ParseErrorCode::UnexpectedCharacter,
                "expected selector component",
            );
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

    fn parse_namespace_qualified_selector(&mut self) {
        let selector_kind =
            if self.namespace_qualified_selector_target_kind() == Some(SyntaxKind::Star) {
                SyntaxKind::UniversalSelector
            } else {
                SyntaxKind::TypeSelector
            };
        self.builder.start_node(selector_kind);
        self.builder.start_node(SyntaxKind::NamespacePrefix);
        if self.current_kind() != Some(SyntaxKind::Pipe) {
            self.token_current();
        }
        self.token_current();
        self.builder.finish_node();
        if matches!(
            self.current_kind(),
            Some(SyntaxKind::Ident | SyntaxKind::CustomPropertyName | SyntaxKind::Star)
        ) {
            self.token_current();
        } else {
            self.empty_bogus_node(
                SyntaxKind::BogusSelector,
                ParseErrorCode::ExpectedSelectorName,
                "expected namespace-qualified selector name",
            );
        }
        self.builder.finish_node();
    }

    fn parse_nesting_selector(&mut self) {
        self.builder.start_node(SyntaxKind::NestingSelectorNode);
        self.token_current();
        self.builder.finish_node();
    }

    fn parse_scss_placeholder_selector(&mut self) {
        self.builder.start_node(SyntaxKind::ScssPlaceholderSelector);
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
        let mut saw_matcher = false;
        let mut saw_value = false;
        let mut closed = false;
        while !self.at_end() {
            match self.current_kind() {
                Some(SyntaxKind::RightBracket) => {
                    self.token_current();
                    closed = true;
                    break;
                }
                Some(kind) if is_attribute_matcher(kind) => {
                    self.parse_attribute_matcher();
                    saw_matcher = true;
                }
                Some(kind) if is_selector_boundary(kind) => break,
                Some(kind) if !saw_matcher && attribute_name_token_can_start(kind) => {
                    self.parse_attribute_name()
                }
                Some(kind)
                    if saw_matcher && !saw_value && attribute_value_token_can_start(kind) =>
                {
                    self.parse_attribute_value();
                    saw_value = true;
                }
                Some(SyntaxKind::Ident | SyntaxKind::CustomPropertyName) if saw_value => {
                    self.parse_attribute_modifier()
                }
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

    fn parse_attribute_matcher(&mut self) {
        self.builder.start_node(SyntaxKind::AttributeMatcher);
        self.token_current();
        self.builder.finish_node();
    }

    fn parse_attribute_name(&mut self) {
        self.builder.start_node(SyntaxKind::AttributeName);
        while !self.at_end() {
            match self.current_kind() {
                Some(SyntaxKind::RightBracket) => break,
                Some(kind) if is_attribute_matcher(kind) || is_selector_boundary(kind) => break,
                Some(kind) if attribute_name_token_can_continue(kind) => self.token_current(),
                Some(_) => break,
                None => break,
            }
        }
        self.builder.finish_node();
    }

    fn parse_attribute_value(&mut self) {
        self.builder.start_node(SyntaxKind::AttributeValue);
        self.token_current();
        self.builder.finish_node();
    }

    fn parse_attribute_modifier(&mut self) {
        self.builder.start_node(SyntaxKind::AttributeModifier);
        self.token_current();
        self.builder.finish_node();
    }

    fn parse_pseudo_selector(&mut self, kind: SyntaxKind) {
        self.builder.start_node(kind);
        self.token_current();
        let pseudo_name = self.current_text().map(str::to_owned);
        let css_module_scope_kind = if kind == SyntaxKind::PseudoClassSelector {
            self.current_text().and_then(css_module_scope_function_kind)
        } else {
            None
        };
        if self.current_kind() == Some(SyntaxKind::Ident) {
            if let Some(kind) = css_module_scope_kind {
                self.builder.start_node(kind);
            }
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
            if kind == SyntaxKind::PseudoClassSelector
                && pseudo_name
                    .as_deref()
                    .is_some_and(is_selector_list_pseudo_class)
            {
                self.parse_selector_list_until(&[SyntaxKind::RightParen]);
            } else if kind == SyntaxKind::PseudoClassSelector
                && pseudo_name.as_deref() == Some("not")
            {
                self.parse_strict_selector_list_until(&[SyntaxKind::RightParen]);
            } else if kind == SyntaxKind::PseudoClassSelector
                && pseudo_name.as_deref() == Some("has")
            {
                self.parse_relative_selector_list_until(&[SyntaxKind::RightParen]);
            } else if kind == SyntaxKind::PseudoClassSelector
                && pseudo_name.as_deref().is_some_and(is_nth_pseudo_class)
            {
                self.parse_nth_selector_argument();
            } else if kind == SyntaxKind::PseudoClassSelector
                && pseudo_name.as_deref() == Some("lang")
            {
                self.parse_language_selector_argument();
            } else if kind == SyntaxKind::PseudoClassSelector
                && pseudo_name.as_deref() == Some("dir")
            {
                self.parse_directionality_selector_argument();
            } else {
                while !self.at_end() {
                    match self.current_kind() {
                        Some(SyntaxKind::RightParen) => break,
                        Some(kind) if is_selector_boundary(kind) => break,
                        Some(_) => self.token_current(),
                        None => break,
                    }
                }
            }
            self.builder.finish_node();
            if self.current_kind() == Some(SyntaxKind::RightParen) {
                self.token_current();
            }
        }
        if css_module_scope_kind.is_some() {
            self.builder.finish_node();
        }
        self.builder.finish_node();
    }

    fn parse_nth_selector_argument(&mut self) {
        self.builder.start_node(SyntaxKind::NthSelectorArgument);
        self.builder.start_node(SyntaxKind::NthSelectorFormula);
        while !self.at_end() {
            match self.current_kind() {
                Some(SyntaxKind::RightParen) => break,
                Some(kind) if is_selector_boundary(kind) => break,
                Some(SyntaxKind::Ident) if self.current_text() == Some("of") => break,
                Some(_) => self.token_current(),
                None => break,
            }
        }
        self.builder.finish_node();

        if self.current_kind() == Some(SyntaxKind::Ident) && self.current_text() == Some("of") {
            self.builder
                .start_node(SyntaxKind::NthSelectorOfSelectorList);
            self.token_current();
            self.parse_selector_list_until(&[SyntaxKind::RightParen]);
            self.builder.finish_node();
        }

        self.builder.finish_node();
    }

    fn parse_language_selector_argument(&mut self) {
        self.builder
            .start_node(SyntaxKind::LanguageSelectorArgument);
        while !self.at_end() {
            match self.current_kind() {
                Some(SyntaxKind::RightParen) => break,
                Some(SyntaxKind::Comma) => self.token_current(),
                Some(kind) if is_selector_boundary(kind) => break,
                Some(kind) if language_tag_token_can_start(kind) => self.parse_language_tag(),
                Some(_) => self.token_current(),
                None => break,
            }
        }
        self.builder.finish_node();
    }

    fn parse_language_tag(&mut self) {
        self.builder.start_node(SyntaxKind::LanguageTag);
        self.token_current();
        self.builder.finish_node();
    }

    fn parse_directionality_selector_argument(&mut self) {
        self.builder
            .start_node(SyntaxKind::DirectionalitySelectorArgument);
        if self
            .current_kind()
            .is_some_and(language_tag_token_can_start)
        {
            self.token_current();
        }
        while !self.at_end() {
            match self.current_kind() {
                Some(SyntaxKind::RightParen) => break,
                Some(kind) if is_selector_boundary(kind) => break,
                Some(_) => self.token_current(),
                None => break,
            }
        }
        self.builder.finish_node();
    }

    fn parse_less_extend_rule(&mut self) {
        self.builder.start_node(SyntaxKind::LessExtendRule);
        if self.current_kind() == Some(SyntaxKind::Colon) {
            self.token_current();
        }
        if self.current_text() == Some("extend") {
            self.token_current();
        } else {
            self.empty_bogus_node(
                SyntaxKind::BogusSelector,
                ParseErrorCode::ExpectedSelectorName,
                "expected Less extend selector",
            );
        }
        if self.current_kind() == Some(SyntaxKind::LeftParen) {
            self.token_current();
            self.builder.start_node(SyntaxKind::PseudoSelectorArgument);
            while !self.at_end() {
                match self.current_kind() {
                    Some(SyntaxKind::RightParen) => break,
                    Some(kind) if is_selector_boundary(kind) => break,
                    Some(kind) if is_interpolation_start(kind) => self.parse_interpolation(
                        kind,
                        &[
                            SyntaxKind::RightParen,
                            SyntaxKind::Comma,
                            SyntaxKind::LeftBrace,
                            SyntaxKind::SassIndent,
                            SyntaxKind::Semicolon,
                            SyntaxKind::SassOptionalSemicolon,
                        ],
                    ),
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
        let has_rhs = self
            .next_non_trivia_kind()
            .is_some_and(|kind| selector_component_can_start(kind) || is_interpolation_start(kind));
        self.builder.start_node(if has_rhs {
            SyntaxKind::Combinator
        } else {
            SyntaxKind::BogusCombinator
        });
        self.token_current();
        if !has_rhs {
            self.error_at_current(
                ParseErrorCode::UnexpectedCharacter,
                "expected selector after combinator",
            );
        }
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
                Some(SyntaxKind::RightBrace | SyntaxKind::SassDedent) | None => break,
                Some(SyntaxKind::Semicolon | SyntaxKind::SassOptionalSemicolon) => {
                    self.token_current()
                }
                Some(SyntaxKind::AtKeyword) if self.current_is_css_module_value_rule() => {
                    self.parse_css_module_value_rule()
                }
                Some(SyntaxKind::AtKeyword) if self.current_dialect_at_rule_spec().is_some() => {
                    self.parse_dialect_at_rule()
                }
                Some(SyntaxKind::AtKeyword) => self.parse_at_rule(),
                Some(_) if self.current_starts_less_namespace_access() => {
                    self.parse_less_namespace_access()
                }
                Some(_) if self.current_starts_less_mixin_call() => self.parse_less_mixin_call(),
                Some(_) if self.current_starts_scss_nested_property() => {
                    self.parse_scss_nested_property()
                }
                Some(_) if self.current_starts_nested_rule() => self.parse_rule(),
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

    fn parse_scss_nested_property(&mut self) {
        self.builder.start_node(SyntaxKind::ScssNestedProperty);
        self.builder.start_node(SyntaxKind::PropertyName);
        while !self.at_end() {
            match self.current_kind() {
                Some(SyntaxKind::Colon) => break,
                Some(
                    SyntaxKind::Semicolon
                    | SyntaxKind::SassOptionalSemicolon
                    | SyntaxKind::RightBrace
                    | SyntaxKind::SassDedent,
                ) => break,
                Some(kind) if is_interpolation_start(kind) => self.parse_interpolation(
                    kind,
                    &[
                        SyntaxKind::Colon,
                        SyntaxKind::Semicolon,
                        SyntaxKind::SassOptionalSemicolon,
                        SyntaxKind::RightBrace,
                        SyntaxKind::SassDedent,
                    ],
                ),
                Some(_) => self.token_current(),
                None => break,
            }
        }
        self.builder.finish_node();

        if self.current_kind() == Some(SyntaxKind::Colon) {
            self.token_current();
        }

        let block_recovery = [
            SyntaxKind::LeftBrace,
            SyntaxKind::SassIndent,
            SyntaxKind::Semicolon,
            SyntaxKind::SassOptionalSemicolon,
            SyntaxKind::RightBrace,
            SyntaxKind::SassDedent,
        ];
        if !matches!(
            self.current_kind(),
            Some(
                SyntaxKind::LeftBrace
                    | SyntaxKind::SassIndent
                    | SyntaxKind::Semicolon
                    | SyntaxKind::SassOptionalSemicolon
                    | SyntaxKind::RightBrace
                    | SyntaxKind::SassDedent
            )
        ) {
            self.builder.start_node(SyntaxKind::Value);
            self.parse_value_or_value_list_until(&block_recovery);
            self.builder.finish_node();
        }

        match self.current_kind() {
            Some(SyntaxKind::LeftBrace) => self.parse_declaration_block(),
            Some(SyntaxKind::SassIndent) => self.parse_sass_indented_nested_property_block(),
            Some(_) => self.consume_until_recovery(&[
                SyntaxKind::Semicolon,
                SyntaxKind::SassOptionalSemicolon,
                SyntaxKind::RightBrace,
                SyntaxKind::SassDedent,
            ]),
            None => {}
        }

        if self.current_kind().is_some_and(is_statement_end) {
            self.token_current();
        }
        self.builder.finish_node();
    }

    fn parse_sass_indented_nested_property_block(&mut self) {
        self.builder.start_node(SyntaxKind::SassIndentedBlock);
        if self.current_kind() == Some(SyntaxKind::SassIndent) {
            self.token_current();
        }
        self.builder.start_node(SyntaxKind::DeclarationList);
        self.parse_declaration_list();
        self.builder.finish_node();
        if self.current_kind() == Some(SyntaxKind::SassDedent) {
            self.token_current();
        } else {
            self.error_at_current(
                ParseErrorCode::UnexpectedCharacter,
                "unterminated Sass indented nested property block",
            );
        }
        self.builder.finish_node();
    }

    fn parse_variable_declaration(&mut self, kind: SyntaxKind) {
        let has_colon = self.find_before_recovery(
            SyntaxKind::Colon,
            &[
                SyntaxKind::Semicolon,
                SyntaxKind::SassOptionalSemicolon,
                SyntaxKind::RightBrace,
                SyntaxKind::SassDedent,
            ],
        );
        self.builder
            .start_node(variable_declaration_node_kind(kind, has_colon));
        self.token_current();
        if self.current_kind() == Some(SyntaxKind::Colon) {
            self.token_current();
            self.eat_value_trivia();
            let value_recovery = [
                SyntaxKind::Semicolon,
                SyntaxKind::SassOptionalSemicolon,
                SyntaxKind::RightBrace,
                SyntaxKind::SassDedent,
            ];
            if kind == SyntaxKind::LessVariableDeclaration
                && self.current_kind() == Some(SyntaxKind::LeftBrace)
            {
                self.parse_less_detached_ruleset();
            } else {
                let has_value = self
                    .non_trivia_token_from(self.position)
                    .is_some_and(|(_, kind)| !value_recovery.contains(&kind));
                self.builder.start_node(SyntaxKind::Value);
                if has_value {
                    self.parse_value_or_value_list_until(&value_recovery);
                } else {
                    self.empty_bogus_node(
                        SyntaxKind::BogusValue,
                        ParseErrorCode::ExpectedValue,
                        "expected variable value",
                    );
                }
                self.builder.finish_node();
            }
        } else {
            self.error_at_current(
                ParseErrorCode::UnexpectedCharacter,
                "expected variable declaration colon",
            );
            self.consume_until_recovery(&[
                SyntaxKind::Semicolon,
                SyntaxKind::SassOptionalSemicolon,
                SyntaxKind::RightBrace,
                SyntaxKind::SassDedent,
            ]);
        }
        if self.current_kind().is_some_and(is_statement_end) {
            self.token_current();
        }
        self.builder.finish_node();
    }

    fn parse_less_detached_ruleset(&mut self) {
        let closed = self.current_left_brace_has_match();
        self.builder.start_node(if closed {
            SyntaxKind::LessDetachedRulesetNode
        } else {
            SyntaxKind::BogusLessDetachedRuleset
        });
        if self.current_kind() == Some(SyntaxKind::LeftBrace) {
            self.token_current();
            self.builder.start_node(SyntaxKind::DeclarationList);
            self.parse_declaration_list();
            self.builder.finish_node();
        }
        if self.current_kind() == Some(SyntaxKind::RightBrace) {
            self.token_current();
        } else {
            self.error_at_current(
                ParseErrorCode::UnexpectedCharacter,
                "unterminated Less detached ruleset",
            );
        }
        self.builder.finish_node();
    }

    fn parse_declaration(&mut self) {
        let starts_composes = self.current_text() == Some("composes");
        let starts_custom_property = self.current_kind() == Some(SyntaxKind::CustomPropertyName);
        let has_colon = self.find_before_recovery(
            SyntaxKind::Colon,
            &[
                SyntaxKind::Semicolon,
                SyntaxKind::SassOptionalSemicolon,
                SyntaxKind::RightBrace,
                SyntaxKind::SassDedent,
                SyntaxKind::LeftBrace,
                SyntaxKind::SassIndent,
            ],
        );
        let kind = if starts_composes && has_colon {
            SyntaxKind::CssModuleComposesDeclaration
        } else if starts_composes {
            SyntaxKind::BogusComposesDeclaration
        } else if has_colon {
            SyntaxKind::Declaration
        } else {
            SyntaxKind::BogusDeclaration
        };
        self.builder.start_node(kind);
        if kind == SyntaxKind::CssModuleComposesDeclaration
            && self.current_css_module_scope_context() == Some("global")
        {
            self.error_at_current(
                ParseErrorCode::UnexpectedCharacter,
                "composes is not allowed inside :global scope",
            );
        }
        let property_kind = if matches!(
            self.current_kind(),
            Some(
                SyntaxKind::Colon
                    | SyntaxKind::Semicolon
                    | SyntaxKind::SassOptionalSemicolon
                    | SyntaxKind::LeftBrace
                    | SyntaxKind::SassIndent
                    | SyntaxKind::RightBrace
                    | SyntaxKind::SassDedent
            )
        ) {
            SyntaxKind::BogusPropertyName
        } else {
            SyntaxKind::PropertyName
        };
        self.builder.start_node(property_kind);
        while !self.at_end() {
            match self.current_kind() {
                Some(
                    SyntaxKind::Colon
                    | SyntaxKind::Semicolon
                    | SyntaxKind::SassOptionalSemicolon
                    | SyntaxKind::RightBrace
                    | SyntaxKind::SassDedent,
                ) => break,
                Some(kind) if is_interpolation_start(kind) => self.parse_interpolation(
                    kind,
                    &[
                        SyntaxKind::Colon,
                        SyntaxKind::Semicolon,
                        SyntaxKind::SassOptionalSemicolon,
                        SyntaxKind::RightBrace,
                        SyntaxKind::SassDedent,
                    ],
                ),
                Some(_) => self.token_current(),
                None => break,
            }
        }
        self.builder.finish_node();
        if property_kind == SyntaxKind::BogusPropertyName {
            self.error_at_current(
                ParseErrorCode::UnexpectedCharacter,
                "expected declaration property name",
            );
        }

        if self.current_kind() == Some(SyntaxKind::Colon) {
            self.token_current();
            let value_recovery = [
                SyntaxKind::Semicolon,
                SyntaxKind::SassOptionalSemicolon,
                SyntaxKind::RightBrace,
                SyntaxKind::SassDedent,
            ];
            let has_value = self
                .non_trivia_token_from(self.position)
                .is_some_and(|(_, kind)| !value_recovery.contains(&kind));
            self.builder.start_node(SyntaxKind::Value);
            if kind == SyntaxKind::CssModuleComposesDeclaration {
                self.parse_composes_value_until(&value_recovery);
            } else if starts_custom_property {
                self.builder.start_node(SyntaxKind::CustomPropertyValue);
                self.parse_component_value_list_until(&value_recovery);
                self.builder.finish_node();
            } else if !has_value {
                self.empty_bogus_node(
                    SyntaxKind::BogusValue,
                    ParseErrorCode::ExpectedValue,
                    "expected declaration value",
                );
            } else {
                self.parse_declaration_value_or_value_list_until(&value_recovery);
            }
            self.builder.finish_node();
        } else {
            self.consume_until_recovery(&[
                SyntaxKind::Semicolon,
                SyntaxKind::SassOptionalSemicolon,
                SyntaxKind::RightBrace,
                SyntaxKind::SassDedent,
            ]);
        }

        if self.current_kind().is_some_and(is_statement_end) {
            self.token_current();
        }
        self.builder.finish_node();
    }

    fn parse_composes_value_until(&mut self, recovery: &[SyntaxKind]) {
        let mut saw_target = false;
        if self.current_composes_value_has_multiple_from_clauses(recovery) {
            self.error_at_current(
                ParseErrorCode::UnexpectedCharacter,
                "multiple composes from clauses are not allowed",
            );
        }
        while !self.at_end() {
            self.eat_value_trivia();
            match self.current_kind() {
                Some(kind) if recovery.contains(&kind) => break,
                Some(SyntaxKind::Ident) if self.current_text() == Some("from") => {
                    if !saw_target {
                        self.empty_bogus_node(
                            SyntaxKind::BogusComposesTarget,
                            ParseErrorCode::UnexpectedCharacter,
                            "expected composes target before from clause",
                        );
                        saw_target = true;
                    }
                    self.parse_css_module_from_clause(recovery);
                }
                Some(SyntaxKind::Ident | SyntaxKind::CustomPropertyName) => {
                    self.builder.start_node(SyntaxKind::CssModuleComposesTarget);
                    self.token_current();
                    self.builder.finish_node();
                    saw_target = true;
                }
                Some(kind) if is_interpolation_start(kind) => {
                    self.parse_interpolation(kind, recovery)
                }
                Some(_) => self.token_current(),
                None => break,
            }
        }
        if !saw_target {
            self.empty_bogus_node(
                SyntaxKind::BogusComposesTarget,
                ParseErrorCode::UnexpectedCharacter,
                "expected composes target",
            );
        }
    }

    fn current_composes_value_has_multiple_from_clauses(&self, recovery: &[SyntaxKind]) -> bool {
        let mut index = self.position;
        let mut paren_depth = 0usize;
        let mut bracket_depth = 0usize;
        let mut brace_depth = 0usize;
        let mut from_count = 0usize;
        while let Some(token) = self.tokens.get(index) {
            if paren_depth == 0
                && bracket_depth == 0
                && brace_depth == 0
                && recovery.contains(&token.kind)
            {
                break;
            }
            match token.kind {
                SyntaxKind::LeftParen => paren_depth += 1,
                SyntaxKind::RightParen => paren_depth = paren_depth.saturating_sub(1),
                SyntaxKind::LeftBracket => bracket_depth += 1,
                SyntaxKind::RightBracket => bracket_depth = bracket_depth.saturating_sub(1),
                SyntaxKind::LeftBrace => brace_depth += 1,
                SyntaxKind::RightBrace => brace_depth = brace_depth.saturating_sub(1),
                SyntaxKind::Ident
                    if paren_depth == 0
                        && bracket_depth == 0
                        && brace_depth == 0
                        && token.text == "from" =>
                {
                    from_count += 1;
                    if from_count > 1 {
                        return true;
                    }
                }
                _ => {}
            }
            index += 1;
        }
        false
    }

    fn parse_css_module_from_clause(&mut self, recovery: &[SyntaxKind]) {
        let source = self.non_trivia_token_from(self.position + 1);
        let has_source = source.is_some_and(|(_, kind)| !recovery.contains(&kind));
        let has_valid_source = source.is_some_and(|(index, kind)| {
            self.tokens
                .get(index)
                .is_some_and(|token| is_css_module_from_source_token(kind, token.text))
        });
        self.builder.start_node(if has_valid_source {
            SyntaxKind::CssModuleFromClause
        } else {
            SyntaxKind::BogusFromClause
        });
        self.token_current();
        while !self.at_end() {
            match self.current_kind() {
                Some(kind) if recovery.contains(&kind) => break,
                Some(_) => self.token_current(),
                None => break,
            }
        }
        if !has_source {
            self.error_at_current(
                ParseErrorCode::UnexpectedCharacter,
                "expected CSS Modules from-clause source",
            );
        } else if !has_valid_source {
            self.error_at_current(
                ParseErrorCode::ExpectedValue,
                "invalid CSS Modules from-clause source",
            );
        }
        self.builder.finish_node();
    }

    fn current_css_module_scope_context(&self) -> Option<&'static str> {
        let mut open_blocks = Vec::new();
        for (index, token) in self.tokens.iter().take(self.position).enumerate() {
            match token.kind {
                SyntaxKind::LeftBrace | SyntaxKind::SassIndent => open_blocks.push(index),
                SyntaxKind::RightBrace | SyntaxKind::SassDedent => {
                    open_blocks.pop();
                }
                _ => {}
            }
        }

        if let Some(scope) = open_blocks.iter().copied().find_map(|block_start| {
            let header_start = self.header_start_for_block(block_start);
            css_module_block_scope_marker_in_header(&self.tokens, header_start, block_start)
        }) {
            return Some(scope);
        }

        let block_start = open_blocks.last().copied()?;
        let header_start = self.header_start_for_block(block_start);
        css_module_header_is_global_only(&self.tokens, header_start, block_start)
            .then_some("global")
    }

    fn header_start_for_block(&self, block_start: usize) -> usize {
        let mut index = block_start;
        while index > 0 {
            let previous = index - 1;
            if matches!(
                self.tokens[previous].kind,
                SyntaxKind::LeftBrace
                    | SyntaxKind::RightBrace
                    | SyntaxKind::SassIndent
                    | SyntaxKind::SassDedent
                    | SyntaxKind::Semicolon
                    | SyntaxKind::SassOptionalSemicolon
            ) {
                break;
            }
            index = previous;
        }
        index
    }

    fn parse_dialect_at_rule(&mut self) {
        let Some(spec) = self.current_dialect_at_rule_spec() else {
            self.parse_at_rule();
            return;
        };

        self.builder
            .start_node(self.current_dialect_at_rule_node_kind(spec));
        if self.current_kind() == Some(SyntaxKind::AtKeyword) {
            self.token_current();
        }
        if matches!(
            spec.node_kind,
            SyntaxKind::ScssUseRule | SyntaxKind::ScssForwardRule
        ) {
            self.parse_scss_module_prelude(spec.node_kind);
        }
        if is_scss_control_rule_kind(spec.node_kind)
            && !self.current_scss_control_prelude_is_valid(spec.node_kind)
        {
            self.error_at_current(
                ParseErrorCode::ExpectedValue,
                "invalid SCSS control prelude",
            );
        }
        while !self.at_end() {
            match self.current_kind() {
                Some(kind) if is_statement_end(kind) => {
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
                Some(SyntaxKind::SassIndent) => {
                    self.parse_sass_indented_at_rule_block(spec.block_kind);
                    break;
                }
                Some(_) => self.token_current(),
                None => break,
            }
        }
        self.builder.finish_node();
    }

    fn parse_scss_module_prelude(&mut self, node_kind: SyntaxKind) {
        self.validate_scss_module_prelude(node_kind);
        while !self.at_end() {
            match self.current_kind() {
                Some(kind)
                    if is_statement_end(kind)
                        || kind == SyntaxKind::LeftBrace
                        || kind == SyntaxKind::SassIndent =>
                {
                    break;
                }
                Some(SyntaxKind::Ident | SyntaxKind::KeywordWith)
                    if self.current_text() == Some("with")
                        && self
                            .non_trivia_token_from(self.position + 1)
                            .is_some_and(|(_, kind)| kind == SyntaxKind::LeftParen) =>
                {
                    self.parse_scss_module_config()
                }
                Some(kind) if is_interpolation_start(kind) => self.parse_interpolation(
                    kind,
                    &[
                        SyntaxKind::Semicolon,
                        SyntaxKind::SassOptionalSemicolon,
                        SyntaxKind::LeftBrace,
                        SyntaxKind::SassIndent,
                    ],
                ),
                Some(_) => self.token_current(),
                None => break,
            }
        }
    }

    fn validate_scss_module_prelude(&mut self, node_kind: SyntaxKind) {
        let recovery = [
            SyntaxKind::Semicolon,
            SyntaxKind::SassOptionalSemicolon,
            SyntaxKind::LeftBrace,
            SyntaxKind::SassIndent,
        ];
        let Some((source_index, source_kind)) = self.non_trivia_token_from(self.position) else {
            self.error_at_current(ParseErrorCode::ExpectedValue, "expected SCSS module source");
            return;
        };
        if recovery.contains(&source_kind) || !is_scss_module_source_token(source_kind) {
            let range = self
                .tokens
                .get(source_index)
                .map(|token| token.range)
                .unwrap_or_else(|| self.current_range());
            self.errors.push(ParseError {
                code: ParseErrorCode::ExpectedValue,
                range,
                message: "expected SCSS module source",
            });
        }

        let mut index = source_index;
        while let Some(token) = self.tokens.get(index).copied() {
            if recovery.contains(&token.kind) {
                break;
            }
            if token.kind == SyntaxKind::Ident {
                if token.text.eq_ignore_ascii_case("as") {
                    let next_kind = self.non_trivia_token_from(index + 1).map(|(_, kind)| kind);
                    if next_kind.is_none_or(|kind| {
                        recovery.contains(&kind) || !is_scss_module_namespace_token(kind)
                    }) {
                        self.errors.push(ParseError {
                            code: ParseErrorCode::ExpectedValue,
                            range: token.range,
                            message: "expected SCSS module namespace",
                        });
                    }
                } else if token.text.eq_ignore_ascii_case("with") {
                    let next_kind = self.non_trivia_token_from(index + 1).map(|(_, kind)| kind);
                    if next_kind != Some(SyntaxKind::LeftParen) {
                        self.errors.push(ParseError {
                            code: ParseErrorCode::ExpectedValue,
                            range: token.range,
                            message: "expected SCSS module configuration",
                        });
                    }
                } else if matches_ignore_ascii_case(token.text, &["show", "hide"]) {
                    if node_kind != SyntaxKind::ScssForwardRule {
                        self.errors.push(ParseError {
                            code: ParseErrorCode::UnexpectedCharacter,
                            range: token.range,
                            message: "unexpected SCSS module visibility clause",
                        });
                    }
                    let next_kind = self.non_trivia_token_from(index + 1).map(|(_, kind)| kind);
                    if next_kind.is_none_or(|kind| {
                        recovery.contains(&kind) || !is_scss_module_visibility_name_token(kind)
                    }) {
                        self.errors.push(ParseError {
                            code: ParseErrorCode::ExpectedValue,
                            range: token.range,
                            message: "expected SCSS module visibility name",
                        });
                    }
                }
            }
            index += 1;
        }
    }

    fn current_scss_control_prelude_is_valid(&self, node_kind: SyntaxKind) -> bool {
        let recovery = [
            SyntaxKind::LeftBrace,
            SyntaxKind::SassIndent,
            SyntaxKind::Semicolon,
            SyntaxKind::SassOptionalSemicolon,
            SyntaxKind::RightBrace,
            SyntaxKind::SassDedent,
        ];
        match node_kind {
            SyntaxKind::ScssControlIf | SyntaxKind::ScssControlWhile => self
                .non_trivia_token_from(self.position)
                .is_some_and(|(_, kind)| !recovery.contains(&kind)),
            SyntaxKind::ScssControlFor => {
                self.non_trivia_token_from(self.position)
                    .is_some_and(|(_, kind)| kind == SyntaxKind::ScssVariable)
                    && self.find_text_before_recovery("from", &recovery)
                    && (self.find_text_before_recovery("to", &recovery)
                        || self.find_text_before_recovery("through", &recovery))
            }
            SyntaxKind::ScssControlEach => {
                self.non_trivia_token_from(self.position)
                    .is_some_and(|(_, kind)| kind == SyntaxKind::ScssVariable)
                    && self.find_text_before_recovery("in", &recovery)
            }
            SyntaxKind::ScssControlElse => true,
            _ => true,
        }
    }

    fn parse_scss_module_config(&mut self) {
        let has_balanced_config = self.current_scss_module_config_has_balanced_parens();
        self.builder.start_node(if has_balanced_config {
            SyntaxKind::ScssModuleConfig
        } else {
            SyntaxKind::BogusScssModuleConfig
        });
        self.token_current();
        self.eat_trivia();
        if self.current_kind() == Some(SyntaxKind::LeftParen) {
            self.parse_balanced_parenthesized_prelude_until(
                None,
                &[
                    SyntaxKind::LeftBrace,
                    SyntaxKind::SassIndent,
                    SyntaxKind::Semicolon,
                    SyntaxKind::SassOptionalSemicolon,
                ],
            );
        }
        self.builder.finish_node();
    }

    fn parse_css_module_value_rule(&mut self) {
        let has_name = self
            .non_trivia_token_from(self.position + 1)
            .and_then(|(index, kind)| {
                self.tokens
                    .get(index)
                    .map(|token| (kind, token.text != "from"))
            })
            .is_some_and(|(kind, allowed_name)| {
                allowed_name && matches!(kind, SyntaxKind::Ident | SyntaxKind::CustomPropertyName)
            });
        let has_from = self.find_text_before_recovery(
            "from",
            &[
                SyntaxKind::Semicolon,
                SyntaxKind::SassOptionalSemicolon,
                SyntaxKind::LeftBrace,
                SyntaxKind::SassIndent,
            ],
        );
        let has_colon = self.find_before_recovery(
            SyntaxKind::Colon,
            &[
                SyntaxKind::Semicolon,
                SyntaxKind::SassOptionalSemicolon,
                SyntaxKind::LeftBrace,
                SyntaxKind::SassIndent,
            ],
        );
        let kind = if !has_name {
            SyntaxKind::BogusCssModuleBlock
        } else if has_from && !has_colon {
            SyntaxKind::CssModuleImportBlock
        } else {
            SyntaxKind::CssModuleExportBlock
        };

        self.builder.start_node(kind);
        self.token_current();
        if !has_name {
            self.error_at_current(
                ParseErrorCode::UnexpectedCharacter,
                "expected CSS Modules @value name",
            );
        }
        if has_colon {
            self.parse_css_module_value_export();
        } else {
            self.parse_css_module_value_import_or_statement();
        }
        if self.current_kind().is_some_and(is_statement_end) {
            self.token_current();
        }
        self.builder.finish_node();
    }

    fn parse_css_module_value_export(&mut self) {
        self.parse_css_module_token_definitions_until(&[
            SyntaxKind::Colon,
            SyntaxKind::Semicolon,
            SyntaxKind::SassOptionalSemicolon,
        ]);
        if self.current_kind() == Some(SyntaxKind::Colon) {
            self.token_current();
            self.builder.start_node(SyntaxKind::Value);
            self.parse_css_module_token_references_until(&[
                SyntaxKind::Semicolon,
                SyntaxKind::SassOptionalSemicolon,
            ]);
            self.builder.finish_node();
        }
    }

    fn parse_css_module_value_import_or_statement(&mut self) {
        self.parse_css_module_token_definitions_until(&[
            SyntaxKind::Semicolon,
            SyntaxKind::SassOptionalSemicolon,
        ]);
    }

    fn parse_css_module_token_definitions_until(&mut self, recovery: &[SyntaxKind]) {
        while !self.at_end() {
            match self.current_kind() {
                Some(kind) if recovery.contains(&kind) => break,
                Some(SyntaxKind::Ident) if self.current_text() == Some("from") => {
                    self.parse_css_module_from_clause(recovery);
                    break;
                }
                Some(SyntaxKind::Ident | SyntaxKind::CustomPropertyName) => {
                    self.builder.start_node(SyntaxKind::TokenDefinition);
                    self.token_current();
                    self.builder.finish_node();
                }
                Some(_) => self.token_current(),
                None => break,
            }
        }
    }

    fn parse_css_module_token_references_until(&mut self, recovery: &[SyntaxKind]) {
        while !self.at_end() {
            self.eat_value_trivia();
            match self.current_kind() {
                Some(kind) if recovery.contains(&kind) => break,
                Some(SyntaxKind::Ident | SyntaxKind::CustomPropertyName) => {
                    self.builder.start_node(SyntaxKind::TokenReference);
                    self.token_current();
                    self.builder.finish_node();
                }
                Some(kind) if is_interpolation_start(kind) => {
                    self.parse_interpolation(kind, recovery)
                }
                Some(_) => self.token_current(),
                None => break,
            }
        }
    }

    fn parse_less_mixin_header(&mut self) {
        self.builder.start_node(SyntaxKind::SelectorList);
        self.parse_until_recovery_with_optional_less_guard(&[SyntaxKind::LeftBrace]);
        self.builder.finish_node();
    }

    fn parse_less_mixin_call(&mut self) {
        self.builder.start_node(SyntaxKind::LessMixinCall);
        self.parse_until_recovery_with_optional_less_guard(&[
            SyntaxKind::Semicolon,
            SyntaxKind::SassOptionalSemicolon,
            SyntaxKind::RightBrace,
            SyntaxKind::SassDedent,
        ]);
        if self.current_kind().is_some_and(is_statement_end) {
            self.token_current();
        }
        self.builder.finish_node();
    }

    fn parse_less_namespace_access(&mut self) {
        self.builder.start_node(SyntaxKind::LessNamespaceAccess);
        while !self.at_end() {
            match self.current_kind() {
                Some(
                    SyntaxKind::Semicolon
                    | SyntaxKind::SassOptionalSemicolon
                    | SyntaxKind::RightBrace
                    | SyntaxKind::SassDedent
                    | SyntaxKind::LeftBrace
                    | SyntaxKind::SassIndent,
                ) => break,
                Some(_) if self.current_starts_less_mixin_call() => {
                    self.parse_less_mixin_call();
                    break;
                }
                Some(_) => self.token_current(),
                None => break,
            }
        }
        if self.current_kind().is_some_and(is_statement_end) {
            self.token_current();
        }
        self.builder.finish_node();
    }

    fn parse_until_recovery_with_optional_less_guard(&mut self, recovery: &[SyntaxKind]) {
        let mut guard_open = false;
        while !self.at_end() {
            match self.current_kind() {
                Some(kind) if recovery.contains(&kind) => break,
                Some(SyntaxKind::Ident) if self.current_text() == Some("when") && !guard_open => {
                    self.builder.start_node(
                        if self.current_less_guard_has_condition_before(recovery) {
                            SyntaxKind::LessMixinGuard
                        } else {
                            SyntaxKind::BogusLessGuard
                        },
                    );
                    guard_open = true;
                    self.token_current();
                }
                Some(_) => self.token_current(),
                None => break,
            }
        }
        if guard_open {
            self.builder.finish_node();
        }
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

    fn parse_value_or_value_list_until(&mut self, recovery: &[SyntaxKind]) {
        if self.current_value_has_top_level_comma_before(recovery) {
            self.parse_value_list_until(recovery);
        } else {
            self.parse_value_until(recovery);
        }
    }

    fn parse_declaration_value_or_value_list_until(&mut self, recovery: &[SyntaxKind]) {
        if self.current_value_has_top_level_comma_before(recovery) {
            self.parse_declaration_value_list_until(recovery);
        } else {
            self.parse_declaration_value_until(recovery);
        }
    }

    fn parse_declaration_value_until(&mut self, recovery: &[SyntaxKind]) {
        let mut saw_value = false;
        while !self.at_end() {
            self.eat_value_trivia();
            if matches!(self.current_kind(), Some(kind) if recovery.contains(&kind)) {
                break;
            }
            if saw_value && self.current_starts_missing_semicolon_declaration(recovery) {
                self.error_at_current(
                    ParseErrorCode::UnexpectedCharacter,
                    "expected semicolon between declarations",
                );
                break;
            }
            if self.at_end() {
                break;
            }
            self.parse_value_expression(0, recovery);
            saw_value = true;
        }
    }

    fn parse_declaration_value_list_until(&mut self, recovery: &[SyntaxKind]) {
        self.builder
            .start_node(if self.current_value_list_is_bogus(recovery) {
                SyntaxKind::BogusValueList
            } else {
                SyntaxKind::ValueList
            });
        let item_recovery = value_list_item_recovery(recovery);
        let mut saw_item = false;
        while !self.at_end() {
            self.eat_value_trivia();
            match self.current_kind() {
                Some(kind) if recovery.contains(&kind) => break,
                Some(SyntaxKind::Comma) => self.token_current(),
                Some(_)
                    if saw_item && self.current_starts_missing_semicolon_declaration(recovery) =>
                {
                    self.error_at_current(
                        ParseErrorCode::UnexpectedCharacter,
                        "expected semicolon between declarations",
                    );
                    break;
                }
                Some(_) => {
                    self.parse_value_expression(0, &item_recovery);
                    saw_item = true;
                }
                None => break,
            }
        }
        self.builder.finish_node();
    }

    fn parse_value_list_until(&mut self, recovery: &[SyntaxKind]) {
        self.builder
            .start_node(if self.current_value_list_is_bogus(recovery) {
                SyntaxKind::BogusValueList
            } else {
                SyntaxKind::ValueList
            });
        let item_recovery = value_list_item_recovery(recovery);
        while !self.at_end() {
            self.eat_value_trivia();
            match self.current_kind() {
                Some(kind) if recovery.contains(&kind) => break,
                Some(SyntaxKind::Comma) => self.token_current(),
                Some(_) => self.parse_value_expression(0, &item_recovery),
                None => break,
            }
        }
        self.builder.finish_node();
    }

    fn parse_component_value(&mut self, recovery: &[SyntaxKind]) {
        self.builder.start_node(SyntaxKind::ComponentValue);
        self.parse_component_value_inner(recovery);
        self.builder.finish_node();
    }

    fn parse_component_value_list_until(&mut self, recovery: &[SyntaxKind]) {
        self.builder.start_node(SyntaxKind::ComponentValueList);
        while !self.at_end() {
            self.eat_value_trivia();
            match self.current_kind() {
                Some(kind) if recovery.contains(&kind) => break,
                Some(_) => self.parse_component_value(recovery),
                None => break,
            }
        }
        self.builder.finish_node();
    }

    fn parse_comma_separated_component_value_list_until(&mut self, recovery: &[SyntaxKind]) {
        self.builder
            .start_node(SyntaxKind::CommaSeparatedComponentValueList);
        let item_recovery = comma_separated_component_value_list_item_recovery(recovery);
        while !self.at_end() {
            self.eat_value_trivia();
            match self.current_kind() {
                Some(kind) if recovery.contains(&kind) => break,
                Some(SyntaxKind::Comma) => self.token_current(),
                Some(_) => self.parse_component_value(&item_recovery),
                None => break,
            }
        }
        self.builder.finish_node();
    }

    fn parse_component_value_inner(&mut self, recovery: &[SyntaxKind]) {
        self.eat_value_trivia();
        match self.current_kind() {
            Some(kind) if recovery.contains(&kind) => {
                self.empty_bogus_node(
                    SyntaxKind::BogusValue,
                    ParseErrorCode::ExpectedValue,
                    "expected component value",
                );
            }
            Some(SyntaxKind::LeftBrace | SyntaxKind::LeftBracket | SyntaxKind::LeftParen) => {
                self.parse_simple_block(recovery)
            }
            Some(SyntaxKind::Ident) if self.next_kind() == Some(SyntaxKind::LeftParen) => {
                self.parse_function_call(recovery)
            }
            Some(kind) if is_component_value_atom_start(kind) => self.parse_value_prefix(recovery),
            Some(_) => self.token_current(),
            None => {
                self.empty_bogus_node(
                    SyntaxKind::BogusValue,
                    ParseErrorCode::ExpectedValue,
                    "expected component value",
                );
            }
        }
    }

    fn parse_simple_block_entry_point(&mut self, recovery: &[SyntaxKind]) {
        self.eat_value_trivia();
        match self.current_kind() {
            Some(SyntaxKind::LeftBrace | SyntaxKind::LeftBracket | SyntaxKind::LeftParen) => {
                self.parse_simple_block(recovery)
            }
            Some(_) | None => {
                self.empty_bogus_node(
                    SyntaxKind::BogusSimpleBlock,
                    ParseErrorCode::ExpectedValue,
                    "expected simple block",
                );
            }
        }
    }

    fn parse_simple_block(&mut self, recovery: &[SyntaxKind]) {
        let Some(open_kind) = self.current_kind() else {
            self.empty_bogus_node(
                SyntaxKind::BogusSimpleBlock,
                ParseErrorCode::ExpectedValue,
                "expected simple block",
            );
            return;
        };
        let Some(close_kind) = matching_simple_block_close(open_kind) else {
            self.empty_bogus_node(
                SyntaxKind::BogusSimpleBlock,
                ParseErrorCode::ExpectedValue,
                "expected simple block",
            );
            return;
        };

        let block_kind = if self.current_simple_block_has_matching_close(recovery) {
            SyntaxKind::SimpleBlock
        } else {
            SyntaxKind::BogusSimpleBlock
        };
        self.builder.start_node(block_kind);
        self.token_current();

        let block_recovery = simple_block_recovery(close_kind, recovery);
        while !self.at_end() {
            self.eat_value_trivia();
            match self.current_kind() {
                Some(kind) if kind == close_kind => break,
                Some(kind) if recovery.contains(&kind) => break,
                Some(_) => self.parse_component_value(&block_recovery),
                None => break,
            }
        }

        if self.current_kind() == Some(close_kind) {
            self.token_current();
        } else {
            self.error_at_current(
                ParseErrorCode::UnexpectedCharacter,
                "unterminated simple block",
            );
        }
        self.builder.finish_node();
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
                self.parse_value_expression(UNARY_PREFIX_RIGHT_BINDING_POWER, recovery);
                self.builder.finish_node();
            }
            Some(SyntaxKind::Ident)
                if self
                    .current_text()
                    .is_some_and(|text| text.eq_ignore_ascii_case("url"))
                    && self.next_kind() == Some(SyntaxKind::LeftParen) =>
            {
                self.builder.start_node(SyntaxKind::UrlValue);
                self.parse_function_call(recovery);
                self.builder.finish_node();
            }
            Some(SyntaxKind::Ident) if self.next_kind() == Some(SyntaxKind::LeftParen) => {
                self.parse_function_call(recovery)
            }
            Some(SyntaxKind::Number) => {
                self.builder.start_node(SyntaxKind::NumberValue);
                self.token_current();
                self.builder.finish_node();
            }
            Some(SyntaxKind::Percentage) => {
                self.builder.start_node(SyntaxKind::PercentageValue);
                self.token_current();
                self.builder.finish_node();
            }
            Some(SyntaxKind::Dimension) => {
                self.builder.start_node(SyntaxKind::DimensionValue);
                self.token_current();
                self.builder.finish_node();
            }
            Some(SyntaxKind::Ident | SyntaxKind::CustomPropertyName) => {
                self.builder.start_node(SyntaxKind::IdentifierValue);
                self.token_current();
                self.builder.finish_node();
            }
            Some(SyntaxKind::String | SyntaxKind::LessEscapedString) => {
                self.builder.start_node(SyntaxKind::StringValue);
                self.token_current();
                self.builder.finish_node();
            }
            Some(SyntaxKind::UnicodeRange) => {
                self.builder.start_node(SyntaxKind::UnicodeRangeValue);
                self.token_current();
                self.builder.finish_node();
            }
            Some(SyntaxKind::Hash) => {
                self.builder.start_node(SyntaxKind::ColorValue);
                self.token_current();
                self.builder.finish_node();
            }
            Some(SyntaxKind::Url) => {
                self.builder.start_node(SyntaxKind::UrlValue);
                self.token_current();
                self.builder.finish_node();
            }
            Some(SyntaxKind::BadUrl) => {
                self.builder.start_node(SyntaxKind::BogusValue);
                self.token_current();
                self.builder.finish_node();
            }
            Some(SyntaxKind::BadString) => {
                self.builder.start_node(SyntaxKind::BogusValue);
                self.token_current();
                self.builder.finish_node();
            }
            Some(SyntaxKind::Important) => {
                self.builder.start_node(SyntaxKind::ImportantAnnotation);
                self.token_current();
                self.builder.finish_node();
            }
            Some(SyntaxKind::Delim) if self.current_split_important_annotation() => {
                self.parse_split_important_annotation()
            }
            Some(SyntaxKind::Delim) if self.current_scss_variable_flag_annotation() => {
                self.parse_scss_variable_flag_annotation()
            }
            Some(kind) if is_interpolation_start(kind) => self.parse_interpolation(kind, recovery),
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
            Some(SyntaxKind::LessPropertyVariableToken) => {
                self.builder.start_node(SyntaxKind::LessPropertyVariable);
                self.token_current();
                self.builder.finish_node();
            }
            Some(SyntaxKind::LeftBrace) => self.parse_simple_block(recovery),
            Some(SyntaxKind::LeftParen) => self.parse_parenthesized_expression(recovery),
            Some(SyntaxKind::LeftBracket) => self.parse_bracketed_value(recovery),
            Some(kind) if recovery.contains(&kind) => {
                self.empty_bogus_node(
                    SyntaxKind::BogusValue,
                    ParseErrorCode::ExpectedValue,
                    "expected value",
                );
            }
            Some(SyntaxKind::Delim) => {
                self.builder.start_node(SyntaxKind::BogusToken);
                self.token_current();
                self.builder.finish_node();
            }
            Some(_) => {
                self.builder.start_node(SyntaxKind::BogusValue);
                self.error_at_current(ParseErrorCode::ExpectedValue, "expected value");
                self.token_current();
                self.builder.finish_node();
            }
            None => {
                self.empty_bogus_node(
                    SyntaxKind::BogusValue,
                    ParseErrorCode::ExpectedValue,
                    "expected value",
                );
            }
        }
    }

    fn parse_split_important_annotation(&mut self) {
        self.builder.start_node(SyntaxKind::ImportantAnnotation);
        self.token_current();
        self.eat_value_trivia();
        if self
            .current_text()
            .is_some_and(|text| text.eq_ignore_ascii_case("important"))
        {
            self.token_current();
        }
        self.builder.finish_node();
    }

    fn parse_scss_variable_flag_annotation(&mut self) {
        self.builder.start_node(SyntaxKind::ScssVariableFlag);
        self.token_current();
        self.eat_value_trivia();
        self.token_current();
        self.builder.finish_node();
    }

    fn eat_value_trivia(&mut self) {
        while matches!(self.current_kind(), Some(kind) if kind.is_trivia()) {
            self.token_current();
        }
    }

    fn parse_function_call(&mut self, recovery: &[SyntaxKind]) {
        let function_name = self.current_text().map(str::to_owned);
        let function_range = self.current_range();
        let argument_count = self.current_function_top_level_argument_count_before(recovery);
        let has_empty_argument_slot =
            self.current_function_has_empty_top_level_argument_slot_before(recovery);
        let argument_head = self.current_function_first_argument_token_before(recovery);
        let specialized_kind = function_name.as_deref().and_then(specialized_function_kind);
        let uses_component_value_arguments = function_name.as_deref().is_some_and(|name| {
            matches_ignore_ascii_case(name, &["if", "media", "supports", "style"])
        });
        let closed = self.current_function_has_closing_paren_before(recovery);
        let function_kind = if closed {
            SyntaxKind::FunctionCall
        } else {
            SyntaxKind::BogusFunctionCall
        };
        let arguments_kind = if closed {
            SyntaxKind::FunctionArguments
        } else {
            SyntaxKind::BogusFunctionArguments
        };

        self.builder.start_node(function_kind);
        if let Some(kind) = specialized_kind {
            self.builder.start_node(kind);
        }
        self.token_current();
        if self.current_kind() == Some(SyntaxKind::LeftParen) {
            self.token_current();
            self.builder.start_node(arguments_kind);
            let mut argument_recovery = function_argument_recovery(recovery);
            if function_name
                .as_deref()
                .is_some_and(|name| name.eq_ignore_ascii_case("if"))
            {
                argument_recovery.retain(|kind| {
                    !matches!(
                        kind,
                        SyntaxKind::Semicolon | SyntaxKind::SassOptionalSemicolon
                    )
                });
            }
            if uses_component_value_arguments {
                self.parse_component_value_list_until(&argument_recovery);
            } else {
                self.parse_value_or_value_list_until(&argument_recovery);
            }
            self.builder.finish_node();
            if self.current_kind() == Some(SyntaxKind::RightParen) {
                self.token_current();
            } else {
                self.error_at_current(
                    ParseErrorCode::UnexpectedCharacter,
                    "unterminated function call",
                );
            }
        }
        if let Some(function_name) = function_name {
            if let Some(argument_count) = argument_count {
                self.validate_function_argument_count(
                    &function_name,
                    argument_count,
                    function_range,
                );
            }
            if let Some(true) = has_empty_argument_slot {
                self.validate_function_argument_slots(&function_name, function_range);
            }
            self.validate_function_argument_head(&function_name, argument_head, function_range);
        }
        if specialized_kind.is_some() {
            self.builder.finish_node();
        }
        self.builder.finish_node();
    }

    fn current_function_top_level_argument_count_before(
        &self,
        recovery: &[SyntaxKind],
    ) -> Option<usize> {
        if self.next_kind() != Some(SyntaxKind::LeftParen) {
            return None;
        }

        let mut index = self.position + 2;
        let mut depth = 0usize;
        let mut comma_count = 0usize;
        let mut saw_argument = false;
        while let Some(token) = self.tokens.get(index) {
            match token.kind {
                kind if depth == 0 && recovery.contains(&kind) => return None,
                SyntaxKind::RightParen if depth == 0 => {
                    return Some(if saw_argument { comma_count + 1 } else { 0 });
                }
                SyntaxKind::Comma if depth == 0 => {
                    comma_count += 1;
                    saw_argument = false;
                }
                kind if kind.is_trivia() => {}
                SyntaxKind::LeftBrace | SyntaxKind::LeftBracket | SyntaxKind::LeftParen => {
                    depth += 1;
                    saw_argument = true;
                }
                SyntaxKind::RightBrace | SyntaxKind::RightBracket | SyntaxKind::RightParen => {
                    depth = depth.saturating_sub(1);
                    saw_argument = true;
                }
                _ => saw_argument = true,
            }
            index += 1;
        }
        None
    }

    fn current_function_has_empty_top_level_argument_slot_before(
        &self,
        recovery: &[SyntaxKind],
    ) -> Option<bool> {
        if self.next_kind() != Some(SyntaxKind::LeftParen) {
            return None;
        }

        let mut index = self.position + 2;
        let mut depth = 0usize;
        let mut expecting_argument = true;
        let mut saw_argument = false;
        while let Some(token) = self.tokens.get(index) {
            match token.kind {
                kind if depth == 0 && recovery.contains(&kind) => return None,
                SyntaxKind::RightParen if depth == 0 => {
                    return Some(expecting_argument && saw_argument);
                }
                SyntaxKind::Comma if depth == 0 => {
                    if expecting_argument {
                        return Some(true);
                    }
                    expecting_argument = true;
                }
                kind if kind.is_trivia() => {}
                SyntaxKind::LeftBrace | SyntaxKind::LeftBracket | SyntaxKind::LeftParen => {
                    depth += 1;
                    expecting_argument = false;
                    saw_argument = true;
                }
                SyntaxKind::RightBrace | SyntaxKind::RightBracket | SyntaxKind::RightParen => {
                    depth = depth.saturating_sub(1);
                    expecting_argument = false;
                    saw_argument = true;
                }
                _ => {
                    expecting_argument = false;
                    saw_argument = true;
                }
            }
            index += 1;
        }
        None
    }

    fn current_function_first_argument_token_before(
        &self,
        recovery: &[SyntaxKind],
    ) -> Option<Token<'text>> {
        if self.next_kind() != Some(SyntaxKind::LeftParen) {
            return None;
        }

        let mut index = self.position + 2;
        while let Some(token) = self.tokens.get(index).copied() {
            match token.kind {
                kind if recovery.contains(&kind) => return None,
                SyntaxKind::RightParen => return None,
                kind if kind.is_trivia() => {}
                _ => return Some(token),
            }
            index += 1;
        }
        None
    }

    fn validate_function_argument_count(
        &mut self,
        function_name: &str,
        argument_count: usize,
        range: TextRange,
    ) {
        if function_argument_count_is_valid(function_name, argument_count) {
            return;
        }
        self.errors.push(ParseError {
            code: ParseErrorCode::ExpectedValue,
            range,
            message: "invalid function argument count",
        });
    }

    fn validate_function_argument_slots(&mut self, function_name: &str, range: TextRange) {
        if !function_requires_filled_top_level_arguments(function_name) {
            return;
        }
        self.errors.push(ParseError {
            code: ParseErrorCode::ExpectedValue,
            range,
            message: "empty function argument",
        });
    }

    fn validate_function_argument_head(
        &mut self,
        function_name: &str,
        argument_head: Option<Token<'text>>,
        range: TextRange,
    ) {
        let head_kind = argument_head.map(|token| token.kind);
        let valid = if function_name.eq_ignore_ascii_case("var") {
            matches!(head_kind, Some(SyntaxKind::CustomPropertyName))
                || head_kind.is_some_and(is_dynamic_function_argument_head)
        } else if function_name.eq_ignore_ascii_case("env") {
            matches!(
                head_kind,
                Some(SyntaxKind::Ident | SyntaxKind::CustomPropertyName)
            ) || head_kind.is_some_and(is_dynamic_function_argument_head)
        } else if function_name.eq_ignore_ascii_case("attr") {
            matches!(head_kind, Some(SyntaxKind::Ident))
                || head_kind.is_some_and(is_dynamic_function_argument_head)
        } else if function_name.eq_ignore_ascii_case("color-mix") {
            argument_head.is_some_and(|token| token.text.eq_ignore_ascii_case("in"))
                || head_kind.is_some_and(is_dynamic_function_argument_head)
        } else {
            true
        };

        if valid {
            return;
        }
        self.errors.push(ParseError {
            code: ParseErrorCode::ExpectedValue,
            range,
            message: "invalid function argument head",
        });
    }

    fn parse_bracketed_value(&mut self, recovery: &[SyntaxKind]) {
        let closed = self.current_bracketed_value_has_closing_bracket_before(recovery);
        self.builder.start_node(if closed {
            SyntaxKind::BracketedValue
        } else {
            SyntaxKind::BogusBracketedValue
        });
        self.token_current();
        let bracket_recovery = bracketed_value_recovery(recovery);
        self.parse_value_until(&bracket_recovery);
        if self.current_kind() == Some(SyntaxKind::RightBracket) {
            self.token_current();
        } else {
            self.error_at_current(
                ParseErrorCode::UnexpectedCharacter,
                "unterminated bracketed value",
            );
        }
        self.builder.finish_node();
    }

    fn parse_parenthesized_expression(&mut self, recovery: &[SyntaxKind]) {
        self.builder.start_node(SyntaxKind::ParenthesizedExpression);
        self.token_current();
        let paren_recovery = function_argument_recovery(recovery);
        self.parse_value_until(&paren_recovery);
        if self.current_kind() == Some(SyntaxKind::RightParen) {
            self.token_current();
        }
        self.builder.finish_node();
    }

    fn parse_at_rule(&mut self) {
        let spec = self.current_text().and_then(at_rule_spec);
        let at_rule_kind = if spec.is_none() && self.current_text() == Some("@") {
            SyntaxKind::BogusAtRule
        } else {
            SyntaxKind::AtRule
        };
        self.builder.start_node(at_rule_kind);
        if at_rule_kind == SyntaxKind::BogusAtRule {
            self.error_at_current(ParseErrorCode::UnexpectedCharacter, "expected at-rule name");
        }
        if let Some(spec) = spec {
            self.builder.start_node(spec.node_kind);
        }

        if self.current_kind() == Some(SyntaxKind::AtKeyword) {
            self.token_current();
        }
        if let Some(spec) = spec {
            self.parse_at_rule_prelude(spec.node_kind);
        } else {
            self.consume_at_rule_prelude_tokens();
        }

        while !self.at_end() {
            match self.current_kind() {
                Some(kind) if is_statement_end(kind) => {
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
                Some(SyntaxKind::SassIndent) => {
                    self.parse_sass_indented_at_rule_block(
                        spec.map(|spec| spec.block_kind)
                            .unwrap_or(AtRuleBlockKind::Raw),
                    );
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

    fn parse_at_rule_prelude(&mut self, node_kind: SyntaxKind) {
        match node_kind {
            SyntaxKind::MediaRule => self.parse_media_query_list(),
            SyntaxKind::SupportsRule => self.parse_supports_rule_prelude(),
            SyntaxKind::ContainerRule => self.parse_container_rule_prelude(),
            SyntaxKind::ImportRule => self.parse_import_prelude(),
            SyntaxKind::CharsetRule => self.parse_charset_rule_prelude(),
            SyntaxKind::NamespaceRule => self.parse_namespace_rule_prelude(),
            SyntaxKind::KeyframesRule => self.parse_keyframes_rule_prelude(),
            SyntaxKind::PageRule => self.parse_page_rule_prelude(),
            SyntaxKind::FontFaceRule
            | SyntaxKind::StartingStyleRule
            | SyntaxKind::PageMarginRule
            | SyntaxKind::FontFeatureValuesStylisticRule
            | SyntaxKind::FontFeatureValuesStylesetRule
            | SyntaxKind::FontFeatureValuesCharacterVariantRule
            | SyntaxKind::FontFeatureValuesSwashRule
            | SyntaxKind::FontFeatureValuesOrnamentsRule
            | SyntaxKind::FontFeatureValuesAnnotationRule
            | SyntaxKind::FontFeatureValuesHistoricalFormsRule
            | SyntaxKind::ViewTransitionRule => {
                self.parse_empty_at_rule_prelude("unexpected at-rule prelude")
            }
            SyntaxKind::PropertyRule => self.parse_named_at_rule_prelude(
                at_rule_prelude_head_is_custom_property_name,
                "invalid @property name",
            ),
            SyntaxKind::FontPaletteValuesRule
            | SyntaxKind::ColorProfileRule
            | SyntaxKind::PositionTryRule => self.parse_named_at_rule_prelude(
                at_rule_prelude_head_is_custom_property_name,
                "invalid at-rule custom property name",
            ),
            SyntaxKind::CustomMediaRule => self.parse_custom_media_rule_prelude(),
            SyntaxKind::CounterStyleRule => self.parse_named_at_rule_prelude(
                at_rule_prelude_head_is_custom_ident,
                "invalid @counter-style name",
            ),
            SyntaxKind::FontFeatureValuesRule => self.parse_font_feature_values_prelude(),
            SyntaxKind::LayerRule => self.parse_layer_rule_prelude(),
            SyntaxKind::ScopeRule => self.parse_scope_rule_prelude(),
            _ => self.consume_at_rule_prelude_tokens(),
        }
    }

    fn parse_media_query_list(&mut self) {
        self.builder.start_node(SyntaxKind::MediaQueryList);
        let mut saw_query = false;
        let mut expecting_query = true;
        while !self.at_end() {
            match self.current_kind() {
                Some(kind) if is_at_rule_prelude_boundary(kind) => break,
                Some(SyntaxKind::Comma) => {
                    if expecting_query {
                        self.error_at_current(
                            ParseErrorCode::ExpectedValue,
                            "invalid @media prelude",
                        );
                        self.builder.start_node(SyntaxKind::BogusMediaQuery);
                        self.token_current();
                        self.builder.finish_node();
                    } else {
                        self.token_current();
                        expecting_query = true;
                    }
                }
                Some(_) => {
                    let valid = self.current_media_query_is_valid();
                    if !valid {
                        self.error_at_current(
                            ParseErrorCode::ExpectedValue,
                            "invalid @media prelude",
                        );
                    }
                    self.parse_media_query(valid);
                    saw_query = true;
                    expecting_query = false;
                }
                None => break,
            }
        }
        if !saw_query || expecting_query {
            self.error_at_current(ParseErrorCode::ExpectedValue, "invalid @media prelude");
            self.builder.start_node(SyntaxKind::BogusMediaQuery);
            self.builder.finish_node();
        }
        self.builder.finish_node();
    }

    fn parse_media_query(&mut self, valid: bool) {
        self.builder.start_node(if valid {
            SyntaxKind::MediaQuery
        } else {
            SyntaxKind::BogusMediaQuery
        });
        while !self.at_end() {
            match self.current_kind() {
                Some(kind) if is_at_rule_prelude_boundary(kind) || kind == SyntaxKind::Comma => {
                    break;
                }
                Some(SyntaxKind::LeftParen) => self.parse_balanced_parenthesized_prelude_until(
                    Some(SyntaxKind::MediaFeature),
                    &[
                        SyntaxKind::Comma,
                        SyntaxKind::LeftBrace,
                        SyntaxKind::Semicolon,
                    ],
                ),
                Some(kind) if is_interpolation_start(kind) => self.parse_interpolation(
                    kind,
                    &[
                        SyntaxKind::Comma,
                        SyntaxKind::LeftBrace,
                        SyntaxKind::Semicolon,
                    ],
                ),
                Some(_) => self.token_current(),
                None => break,
            }
        }
        self.builder.finish_node();
    }

    fn current_media_query_is_valid(&self) -> bool {
        let Some((first_index, first_kind)) = self.non_trivia_token_from(self.position) else {
            return false;
        };
        if is_at_rule_prelude_boundary(first_kind) || first_kind == SyntaxKind::Comma {
            return false;
        }
        if !self.current_prelude_parentheses_are_balanced_until(&[
            SyntaxKind::Comma,
            SyntaxKind::LeftBrace,
            SyntaxKind::SassIndent,
            SyntaxKind::Semicolon,
            SyntaxKind::SassOptionalSemicolon,
        ]) {
            return false;
        }
        self.media_query_starts_at(first_index, first_kind)
    }

    fn media_query_starts_at(&self, index: usize, kind: SyntaxKind) -> bool {
        match kind {
            SyntaxKind::Ident | SyntaxKind::LeftParen => true,
            SyntaxKind::KeywordNot | SyntaxKind::KeywordOnly => self
                .non_trivia_token_from(index + 1)
                .is_some_and(|(_, next_kind)| {
                    matches!(next_kind, SyntaxKind::Ident | SyntaxKind::LeftParen)
                        || is_interpolation_start(next_kind)
                }),
            kind if is_interpolation_start(kind) => true,
            _ => false,
        }
    }

    fn parse_charset_rule_prelude(&mut self) {
        if !self.charset_rule_prelude_is_valid() {
            self.error_at_current(ParseErrorCode::ExpectedValue, "invalid @charset prelude");
        }
        self.consume_at_rule_prelude_tokens();
    }

    fn charset_rule_prelude_is_valid(&self) -> bool {
        let Some((source_index, SyntaxKind::String)) = self.non_trivia_token_from(self.position)
        else {
            return false;
        };
        self.non_trivia_token_from(source_index + 1)
            .is_none_or(|(_, kind)| is_at_rule_prelude_boundary(kind))
    }

    fn parse_namespace_rule_prelude(&mut self) {
        if !self.namespace_rule_prelude_is_valid() {
            self.error_at_current(ParseErrorCode::ExpectedValue, "invalid @namespace prelude");
        }
        self.consume_at_rule_prelude_tokens();
    }

    fn parse_custom_media_rule_prelude(&mut self) {
        self.eat_trivia();
        let valid = self.custom_media_rule_prelude_is_valid();
        if !valid {
            self.error_at_current(
                ParseErrorCode::ExpectedValue,
                "invalid @custom-media prelude",
            );
        }
        self.builder.start_node(if valid {
            SyntaxKind::AtRulePrelude
        } else {
            SyntaxKind::BogusAtRulePrelude
        });
        self.consume_at_rule_prelude_tokens_without_wrapping();
        self.builder.finish_node();
    }

    fn custom_media_rule_prelude_is_valid(&self) -> bool {
        let Some((name_index, name_kind)) = self.non_trivia_token_from(self.position) else {
            return false;
        };
        if !self.current_prelude_parentheses_are_balanced_until(&[
            SyntaxKind::Semicolon,
            SyntaxKind::SassOptionalSemicolon,
        ]) {
            return false;
        }
        let tail = if name_kind == SyntaxKind::CustomPropertyName {
            self.non_trivia_token_from(name_index + 1)
        } else if is_interpolation_start(name_kind) {
            self.non_trivia_token_after_interpolation(name_index, name_kind)
        } else {
            return false;
        };
        let Some((tail_index, tail_kind)) = tail else {
            return false;
        };
        if is_at_rule_prelude_boundary(tail_kind) {
            return false;
        }
        self.media_query_starts_at(tail_index, tail_kind)
    }

    fn namespace_rule_prelude_is_valid(&self) -> bool {
        let Some((first_index, first_kind)) = self.non_trivia_token_from(self.position) else {
            return false;
        };

        if self.namespace_source_starts_at(first_index, first_kind) {
            return true;
        }
        if !matches!(
            first_kind,
            SyntaxKind::Ident | SyntaxKind::CustomPropertyName
        ) {
            return false;
        }
        self.non_trivia_token_from(first_index + 1)
            .is_some_and(|(source_index, source_kind)| {
                self.namespace_source_starts_at(source_index, source_kind)
            })
    }

    fn namespace_source_starts_at(&self, index: usize, kind: SyntaxKind) -> bool {
        matches!(kind, SyntaxKind::String | SyntaxKind::Url)
            || is_interpolation_start(kind)
            || self.token_starts_url_function(index, kind)
    }

    fn token_starts_url_function(&self, index: usize, kind: SyntaxKind) -> bool {
        kind == SyntaxKind::Ident
            && self
                .tokens
                .get(index)
                .is_some_and(|token| token.text.eq_ignore_ascii_case("url"))
            && self
                .non_trivia_token_from(index + 1)
                .is_some_and(|(_, next_kind)| next_kind == SyntaxKind::LeftParen)
    }

    fn parse_keyframes_rule_prelude(&mut self) {
        if !self.keyframes_rule_prelude_is_valid() {
            self.error_at_current(ParseErrorCode::ExpectedValue, "invalid @keyframes name");
        }
        self.consume_at_rule_prelude_tokens();
    }

    fn keyframes_rule_prelude_is_valid(&self) -> bool {
        let Some((name_index, name_kind)) = self.non_trivia_token_from(self.position) else {
            return false;
        };
        if is_interpolation_start(name_kind) {
            return true;
        }
        if !matches!(name_kind, SyntaxKind::Ident | SyntaxKind::String) {
            return false;
        }
        self.non_trivia_token_from(name_index + 1)
            .is_none_or(|(_, kind)| is_at_rule_prelude_boundary(kind))
    }

    fn parse_empty_at_rule_prelude(&mut self, message: &'static str) {
        self.eat_trivia();
        if self
            .current_kind()
            .is_some_and(|kind| !is_at_rule_prelude_boundary(kind))
        {
            self.error_at_current(ParseErrorCode::ExpectedValue, message);
            self.consume_at_rule_prelude_tokens();
        }
    }

    fn parse_font_feature_values_prelude(&mut self) {
        if !self.font_feature_values_prelude_is_valid() {
            self.error_at_current(
                ParseErrorCode::ExpectedValue,
                "invalid @font-feature-values family name",
            );
        }
        self.consume_at_rule_prelude_tokens();
    }

    fn font_feature_values_prelude_is_valid(&self) -> bool {
        self.non_trivia_token_from(self.position)
            .is_some_and(|(_, kind)| {
                matches!(kind, SyntaxKind::Ident | SyntaxKind::String)
                    || is_interpolation_start(kind)
            })
    }

    fn parse_layer_rule_prelude(&mut self) {
        self.eat_trivia();
        match self.current_kind() {
            Some(SyntaxKind::LeftBrace | SyntaxKind::SassIndent) => return,
            Some(SyntaxKind::Semicolon | SyntaxKind::SassOptionalSemicolon) | None => {
                self.empty_bogus_node(
                    SyntaxKind::BogusLayerName,
                    ParseErrorCode::ExpectedValue,
                    "invalid @layer prelude",
                );
                return;
            }
            Some(_) => {}
        }

        let valid = self.layer_rule_prelude_is_valid();
        if !valid {
            self.error_at_current(ParseErrorCode::ExpectedValue, "invalid @layer prelude");
        }
        self.builder.start_node(if valid {
            SyntaxKind::LayerName
        } else {
            SyntaxKind::BogusLayerName
        });
        self.consume_at_rule_prelude_tokens_without_wrapping();
        self.builder.finish_node();
    }

    fn layer_rule_prelude_is_valid(&self) -> bool {
        let mut saw_name = false;
        let mut expecting_segment = true;
        let mut index = self.position;

        while let Some(token) = self.tokens.get(index) {
            if token.kind.is_trivia() {
                index += 1;
                continue;
            }
            if is_at_rule_prelude_boundary(token.kind) {
                return saw_name && !expecting_segment;
            }
            if is_interpolation_start(token.kind) {
                return true;
            }
            match token.kind {
                SyntaxKind::Ident if expecting_segment => {
                    saw_name = true;
                    expecting_segment = false;
                }
                SyntaxKind::Comma if saw_name && !expecting_segment => {
                    expecting_segment = true;
                }
                SyntaxKind::Dot if saw_name && !expecting_segment => {
                    expecting_segment = true;
                }
                _ => return false,
            }
            index += 1;
        }

        saw_name && !expecting_segment
    }

    fn parse_container_rule_prelude(&mut self) {
        self.eat_trivia();
        let valid = self.container_rule_prelude_is_valid();
        if !valid {
            self.error_at_current(ParseErrorCode::ExpectedValue, "invalid @container prelude");
        }
        self.builder.start_node(if valid {
            SyntaxKind::ContainerCondition
        } else {
            SyntaxKind::BogusContainerCondition
        });
        self.consume_at_rule_prelude_tokens_without_wrapping();
        self.builder.finish_node();
    }

    fn container_rule_prelude_is_valid(&self) -> bool {
        let Some((first_index, first_kind)) = self.non_trivia_token_from(self.position) else {
            return false;
        };
        if is_at_rule_prelude_boundary(first_kind) {
            return false;
        }
        if !self.current_prelude_parentheses_are_balanced_until(&[
            SyntaxKind::LeftBrace,
            SyntaxKind::SassIndent,
            SyntaxKind::Semicolon,
            SyntaxKind::SassOptionalSemicolon,
        ]) {
            return false;
        }
        if self.container_condition_starts_at(first_index, first_kind) {
            return true;
        }
        if first_kind != SyntaxKind::Ident {
            return false;
        }
        self.non_trivia_token_from(first_index + 1).is_some_and(
            |(condition_index, condition_kind)| {
                self.container_condition_starts_at(condition_index, condition_kind)
            },
        )
    }

    fn container_condition_starts_at(&self, index: usize, kind: SyntaxKind) -> bool {
        if matches!(kind, SyntaxKind::LeftParen | SyntaxKind::KeywordNot)
            || is_interpolation_start(kind)
        {
            return true;
        }
        kind == SyntaxKind::Ident
            && self
                .non_trivia_token_from(index + 1)
                .is_some_and(|(_, next_kind)| next_kind == SyntaxKind::LeftParen)
    }

    fn parse_supports_rule_prelude(&mut self) {
        self.eat_trivia();
        let valid = self.supports_rule_prelude_is_valid();
        if !valid {
            self.error_at_current(ParseErrorCode::ExpectedValue, "invalid @supports prelude");
        }
        self.builder.start_node(if valid {
            SyntaxKind::SupportsCondition
        } else {
            SyntaxKind::BogusSupportsCondition
        });
        self.consume_at_rule_prelude_tokens_without_wrapping();
        self.builder.finish_node();
    }

    fn supports_rule_prelude_is_valid(&self) -> bool {
        let Some((first_index, first_kind)) = self.non_trivia_token_from(self.position) else {
            return false;
        };
        if is_at_rule_prelude_boundary(first_kind) {
            return false;
        }
        if !self.current_prelude_parentheses_are_balanced_until(&[
            SyntaxKind::LeftBrace,
            SyntaxKind::SassIndent,
            SyntaxKind::Semicolon,
            SyntaxKind::SassOptionalSemicolon,
        ]) {
            return false;
        }
        self.supports_condition_starts_at(first_index, first_kind)
    }

    fn supports_condition_starts_at(&self, index: usize, kind: SyntaxKind) -> bool {
        if kind == SyntaxKind::KeywordNot {
            return self
                .non_trivia_token_from(index + 1)
                .is_some_and(|(next_index, next_kind)| {
                    self.supports_condition_starts_at(next_index, next_kind)
                });
        }
        if kind == SyntaxKind::LeftParen || is_interpolation_start(kind) {
            return true;
        }
        kind == SyntaxKind::Ident
            && self
                .non_trivia_token_from(index + 1)
                .is_some_and(|(_, next_kind)| next_kind == SyntaxKind::LeftParen)
    }

    fn parse_scope_rule_prelude(&mut self) {
        self.eat_trivia();
        let valid = self.scope_rule_prelude_is_valid();
        if !valid {
            self.error_at_current(ParseErrorCode::ExpectedValue, "invalid @scope prelude");
        }
        self.builder.start_node(if valid {
            SyntaxKind::ScopeRange
        } else {
            SyntaxKind::BogusScopeRange
        });
        self.consume_at_rule_prelude_tokens_without_wrapping();
        self.builder.finish_node();
    }

    fn scope_rule_prelude_is_valid(&self) -> bool {
        let Some((start_index, start_kind)) = self.non_trivia_token_from(self.position) else {
            return false;
        };
        if is_at_rule_prelude_boundary(start_kind) {
            return false;
        }
        if !self.current_prelude_parentheses_are_balanced_until(&[
            SyntaxKind::LeftBrace,
            SyntaxKind::SassIndent,
            SyntaxKind::Semicolon,
            SyntaxKind::SassOptionalSemicolon,
        ]) {
            return false;
        }
        if is_interpolation_start(start_kind) {
            return true;
        }
        if start_kind != SyntaxKind::LeftParen {
            return false;
        }

        let Some(start_close_index) = self.parenthesized_prelude_close_index(start_index) else {
            return false;
        };
        let Some((after_start_index, after_start_kind)) =
            self.non_trivia_token_from(start_close_index + 1)
        else {
            return true;
        };
        if is_at_rule_prelude_boundary(after_start_kind) {
            return true;
        }
        if after_start_kind != SyntaxKind::Ident
            || !self
                .tokens
                .get(after_start_index)
                .is_some_and(|token| token.text.eq_ignore_ascii_case("to"))
        {
            return false;
        }

        let Some((end_index, end_kind)) = self.non_trivia_token_from(after_start_index + 1) else {
            return false;
        };
        if is_interpolation_start(end_kind) {
            return true;
        }
        if end_kind != SyntaxKind::LeftParen {
            return false;
        }
        let Some(end_close_index) = self.parenthesized_prelude_close_index(end_index) else {
            return false;
        };
        self.non_trivia_token_from(end_close_index + 1)
            .is_none_or(|(_, kind)| is_at_rule_prelude_boundary(kind))
    }

    fn parenthesized_prelude_close_index(&self, open_index: usize) -> Option<usize> {
        let mut depth = 0usize;
        for (index, token) in self.tokens.iter().enumerate().skip(open_index) {
            match token.kind {
                SyntaxKind::LeftParen => depth += 1,
                SyntaxKind::RightParen => {
                    depth = depth.saturating_sub(1);
                    if depth == 0 {
                        return Some(index);
                    }
                }
                kind if depth == 0 && is_at_rule_prelude_boundary(kind) => return None,
                _ => {}
            }
        }
        None
    }

    fn parse_page_rule_prelude(&mut self) {
        self.eat_trivia();
        if self.current_kind().is_none_or(is_at_rule_prelude_boundary) {
            return;
        }
        let valid = self.page_rule_prelude_is_valid();
        if !valid {
            self.error_at_current(ParseErrorCode::ExpectedValue, "invalid @page prelude");
        }
        self.builder.start_node(if valid {
            SyntaxKind::AtRulePrelude
        } else {
            SyntaxKind::BogusAtRulePrelude
        });
        self.consume_at_rule_prelude_tokens_without_wrapping();
        self.builder.finish_node();
    }

    fn page_rule_prelude_is_valid(&self) -> bool {
        let mut expecting_selector = true;
        let mut expecting_pseudo_name = false;
        let mut saw_selector = false;

        for token in self.tokens.iter().skip(self.position) {
            if token.kind.is_trivia() {
                continue;
            }
            if is_at_rule_prelude_boundary(token.kind) {
                return saw_selector && !expecting_selector && !expecting_pseudo_name;
            }
            if is_interpolation_start(token.kind) {
                return true;
            }
            if expecting_pseudo_name {
                if token.kind != SyntaxKind::Ident {
                    return false;
                }
                saw_selector = true;
                expecting_selector = false;
                expecting_pseudo_name = false;
                continue;
            }
            match token.kind {
                SyntaxKind::Ident if expecting_selector => {
                    saw_selector = true;
                    expecting_selector = false;
                }
                SyntaxKind::Colon => {
                    expecting_pseudo_name = true;
                }
                SyntaxKind::Comma if saw_selector && !expecting_selector => {
                    expecting_selector = true;
                }
                _ => return false,
            }
        }

        saw_selector && !expecting_selector && !expecting_pseudo_name
    }

    fn parse_import_prelude(&mut self) {
        self.eat_trivia();
        if self.dialect == StyleDialect::Less && self.current_kind() == Some(SyntaxKind::LeftParen)
        {
            self.builder.start_node(SyntaxKind::AtRulePrelude);
            self.parse_balanced_parenthesized_prelude(None);
            self.builder.finish_node();
            self.eat_trivia();
        }
        if !self.parse_import_source() {
            self.parse_bogus_import_prelude();
            return;
        }
        while !self.at_end() {
            match self.current_kind() {
                Some(kind) if is_at_rule_prelude_boundary(kind) => break,
                Some(kind) if kind.is_trivia() => self.token_current(),
                Some(SyntaxKind::Ident) if self.current_text() == Some("layer") => {
                    self.parse_import_layer_tail_node()
                }
                Some(SyntaxKind::Ident) if self.current_text() == Some("supports") => {
                    self.parse_import_supports_tail_node()
                }
                Some(_) => {
                    self.parse_media_query_list();
                    break;
                }
                None => break,
            }
        }
    }

    fn parse_import_source(&mut self) -> bool {
        match self.current_kind() {
            Some(SyntaxKind::Url) => {
                self.builder.start_node(SyntaxKind::UrlValue);
                self.token_current();
                self.builder.finish_node();
                true
            }
            Some(SyntaxKind::Ident)
                if self
                    .current_text()
                    .is_some_and(|text| text.eq_ignore_ascii_case("url"))
                    && self.next_kind() == Some(SyntaxKind::LeftParen) =>
            {
                self.builder.start_node(SyntaxKind::UrlValue);
                self.parse_function_call(&[SyntaxKind::LeftBrace, SyntaxKind::Semicolon]);
                self.builder.finish_node();
                true
            }
            Some(SyntaxKind::String) => {
                self.token_current();
                true
            }
            Some(kind) if is_interpolation_start(kind) => {
                self.parse_interpolation(kind, &[SyntaxKind::LeftBrace, SyntaxKind::Semicolon]);
                true
            }
            Some(_) | None => false,
        }
    }

    fn parse_bogus_import_prelude(&mut self) {
        self.builder.start_node(SyntaxKind::BogusAtRulePrelude);
        self.error_at_current(ParseErrorCode::ExpectedValue, "invalid @import source");
        self.consume_at_rule_prelude_tokens_without_wrapping();
        self.builder.finish_node();
    }

    fn parse_named_at_rule_prelude(
        &mut self,
        valid_head: fn(SyntaxKind) -> bool,
        message: &'static str,
    ) {
        if self.current_kind().is_none_or(is_at_rule_prelude_boundary) {
            return;
        }
        let valid_name = self
            .non_trivia_token_from(self.position)
            .is_some_and(|(_, kind)| valid_head(kind));
        if !valid_name {
            self.error_at_current(ParseErrorCode::ExpectedValue, message);
        }
        self.consume_at_rule_prelude_tokens();
    }

    fn parse_import_layer_tail_node(&mut self) {
        let valid = self.import_layer_tail_is_valid();
        if !valid {
            self.error_at_current(ParseErrorCode::ExpectedValue, "invalid @import layer tail");
        }
        self.builder.start_node(if valid {
            SyntaxKind::LayerName
        } else {
            SyntaxKind::BogusLayerName
        });
        self.token_current();
        if self.current_kind() == Some(SyntaxKind::LeftParen) {
            self.parse_balanced_parenthesized_prelude(None);
        }
        self.builder.finish_node();
    }

    fn import_layer_tail_is_valid(&self) -> bool {
        let Some((open_index, next_kind)) = self.non_trivia_token_from(self.position + 1) else {
            return true;
        };
        if next_kind != SyntaxKind::LeftParen {
            return true;
        }
        let Some(close_index) = self.parenthesized_prelude_close_index(open_index) else {
            return false;
        };
        self.layer_name_is_valid_between(open_index + 1, close_index)
    }

    fn layer_name_is_valid_between(&self, start: usize, end: usize) -> bool {
        let mut saw_name = false;
        let mut expecting_segment = true;

        for token in self.tokens[start..end]
            .iter()
            .filter(|token| !token.kind.is_trivia())
        {
            if is_interpolation_start(token.kind) {
                return true;
            }
            match token.kind {
                SyntaxKind::Ident if expecting_segment => {
                    saw_name = true;
                    expecting_segment = false;
                }
                SyntaxKind::Dot if saw_name && !expecting_segment => {
                    expecting_segment = true;
                }
                _ => return false,
            }
        }

        saw_name && !expecting_segment
    }

    fn parse_import_supports_tail_node(&mut self) {
        let valid = self.import_supports_tail_is_valid();
        if !valid {
            self.error_at_current(
                ParseErrorCode::ExpectedValue,
                "invalid @import supports tail",
            );
        }
        self.builder.start_node(if valid {
            SyntaxKind::SupportsCondition
        } else {
            SyntaxKind::BogusSupportsCondition
        });
        self.token_current();
        if self.current_kind() == Some(SyntaxKind::LeftParen) {
            self.parse_balanced_parenthesized_prelude(None);
        }
        self.builder.finish_node();
    }

    fn import_supports_tail_is_valid(&self) -> bool {
        let Some((open_index, SyntaxKind::LeftParen)) =
            self.non_trivia_token_from(self.position + 1)
        else {
            return false;
        };
        let Some(close_index) = self.parenthesized_prelude_close_index(open_index) else {
            return false;
        };
        self.non_trivia_token_from(open_index + 1)
            .is_some_and(|(inner_index, inner_kind)| {
                inner_index < close_index && inner_kind != SyntaxKind::RightParen
            })
    }

    fn consume_at_rule_prelude_tokens(&mut self) {
        if self.current_kind().is_none_or(is_at_rule_prelude_boundary) {
            return;
        }
        self.builder
            .start_node(self.current_generic_at_rule_prelude_node_kind());
        self.consume_at_rule_prelude_tokens_without_wrapping();
        self.builder.finish_node();
    }

    fn consume_at_rule_prelude_tokens_without_wrapping(&mut self) {
        while !self.at_end() {
            match self.current_kind() {
                Some(kind) if is_at_rule_prelude_boundary(kind) => break,
                Some(SyntaxKind::LeftParen) => self.parse_balanced_parenthesized_prelude(None),
                Some(kind) if is_interpolation_start(kind) => {
                    self.parse_interpolation(kind, &[SyntaxKind::LeftBrace, SyntaxKind::Semicolon])
                }
                Some(_) => self.token_current(),
                None => break,
            }
        }
    }

    fn parse_balanced_parenthesized_prelude(&mut self, node_kind: Option<SyntaxKind>) {
        self.parse_balanced_parenthesized_prelude_until(
            node_kind,
            &[SyntaxKind::LeftBrace, SyntaxKind::Semicolon],
        );
    }

    fn parse_balanced_parenthesized_prelude_until(
        &mut self,
        node_kind: Option<SyntaxKind>,
        recovery: &[SyntaxKind],
    ) {
        if let Some(kind) = node_kind {
            self.builder.start_node(kind);
        }
        let mut depth = 0usize;
        let mut closed = false;
        while !self.at_end() {
            match self.current_kind() {
                Some(SyntaxKind::LeftParen) => {
                    depth += 1;
                    self.token_current();
                }
                Some(SyntaxKind::RightParen) => {
                    self.token_current();
                    depth = depth.saturating_sub(1);
                    if depth == 0 {
                        closed = true;
                        break;
                    }
                }
                Some(kind) if recovery.contains(&kind) => break,
                Some(kind) if is_interpolation_start(kind) => {
                    self.parse_interpolation(kind, &[SyntaxKind::LeftBrace, SyntaxKind::Semicolon])
                }
                Some(_) => self.token_current(),
                None => break,
            }
        }
        if node_kind.is_some() {
            self.builder.finish_node();
        }
        if !closed {
            self.error_at_current(
                ParseErrorCode::UnexpectedCharacter,
                "unterminated parenthesized prelude",
            );
        }
    }

    fn parse_interpolation(&mut self, start_kind: SyntaxKind, recovery: &[SyntaxKind]) {
        let Some(end_kind) = interpolation_end_kind(start_kind) else {
            self.token_current();
            return;
        };
        let closed = self.find_before_recovery(end_kind, recovery);
        self.builder.start_node(if closed {
            SyntaxKind::Interpolation
        } else {
            SyntaxKind::BogusInterpolation
        });
        if self.current_kind() == Some(start_kind) {
            self.token_current();
        }
        while !self.at_end() {
            match self.current_kind() {
                Some(kind) if kind == end_kind => {
                    self.token_current();
                    break;
                }
                Some(kind) if !closed && recovery.contains(&kind) => break,
                Some(_) => self.token_current(),
                None => break,
            }
        }
        if !closed {
            self.error_at_current(
                ParseErrorCode::UnexpectedCharacter,
                "unterminated interpolation",
            );
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
                Some(SyntaxKind::RightBrace | SyntaxKind::SassDedent) | None => break,
                Some(SyntaxKind::Semicolon | SyntaxKind::SassOptionalSemicolon) => {
                    self.token_current()
                }
                Some(SyntaxKind::AtKeyword) if self.current_is_css_module_value_rule() => {
                    self.parse_css_module_value_rule()
                }
                Some(SyntaxKind::AtKeyword) if self.current_dialect_at_rule_spec().is_some() => {
                    self.parse_dialect_at_rule()
                }
                Some(SyntaxKind::AtKeyword) => self.parse_at_rule(),
                Some(_) => self.parse_rule(),
            }
        }
    }

    fn parse_declaration_block(&mut self) {
        self.token_current();
        self.builder
            .start_node(if self.previous_left_brace_has_match() {
                SyntaxKind::DeclarationList
            } else {
                SyntaxKind::BogusDeclarationList
            });
        self.parse_declaration_list();
        self.builder.finish_node();
        if self.current_kind() == Some(SyntaxKind::RightBrace) {
            self.token_current();
        } else {
            self.missing_token_bogus_trivia(
                ParseErrorCode::UnexpectedCharacter,
                "unterminated declaration block",
            );
        }
    }

    fn parse_sass_indented_at_rule_block(&mut self, block_kind: AtRuleBlockKind) {
        self.builder.start_node(SyntaxKind::SassIndentedBlock);
        if self.current_kind() == Some(SyntaxKind::SassIndent) {
            self.token_current();
        }
        match block_kind {
            AtRuleBlockKind::GroupRuleList => {
                self.builder.start_node(SyntaxKind::RuleList);
                self.parse_rule_list_items();
                self.builder.finish_node();
            }
            AtRuleBlockKind::DeclarationList | AtRuleBlockKind::Keyframes => {
                self.builder.start_node(SyntaxKind::DeclarationList);
                self.parse_declaration_list();
                self.builder.finish_node();
            }
            AtRuleBlockKind::Raw => self.consume_sass_indented_raw_body(),
        }
        if self.current_kind() == Some(SyntaxKind::SassDedent) {
            self.token_current();
        } else {
            self.error_at_current(
                ParseErrorCode::UnexpectedCharacter,
                "unterminated Sass indented at-rule block",
            );
        }
        self.builder.finish_node();
    }

    fn consume_sass_indented_raw_body(&mut self) {
        let mut depth = 0usize;
        while !self.at_end() {
            match self.current_kind() {
                Some(SyntaxKind::SassIndent) => {
                    depth += 1;
                    self.token_current();
                }
                Some(SyntaxKind::SassDedent) if depth == 0 => break,
                Some(SyntaxKind::SassDedent) => {
                    depth = depth.saturating_sub(1);
                    self.token_current();
                }
                Some(_) => self.token_current(),
                None => break,
            }
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
        let has_block = self.find_before_recovery(SyntaxKind::LeftBrace, &[SyntaxKind::RightBrace]);
        self.builder.start_node(if has_block {
            SyntaxKind::KeyframeBlock
        } else {
            SyntaxKind::BogusKeyframeBlock
        });
        if has_block && !self.keyframe_selector_list_is_valid() {
            self.error_at_current(ParseErrorCode::ExpectedValue, "invalid keyframe selector");
        }
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
        if !has_block {
            self.error_at_current(
                ParseErrorCode::UnexpectedCharacter,
                "expected keyframe declaration block",
            );
        }
        self.builder.finish_node();
    }

    fn keyframe_selector_list_is_valid(&self) -> bool {
        let mut index = self.position;
        let mut saw_selector = false;
        let mut expect_selector = true;
        loop {
            let Some((token_index, kind)) = self.non_trivia_token_from(index) else {
                return false;
            };
            if kind == SyntaxKind::LeftBrace {
                return saw_selector && !expect_selector;
            }
            if expect_selector {
                if is_interpolation_start(kind) {
                    return true;
                }
                if !keyframe_selector_token_is_valid(self.tokens[token_index]) {
                    return false;
                }
                saw_selector = true;
                expect_selector = false;
                index = token_index + 1;
                continue;
            }
            if kind != SyntaxKind::Comma {
                return false;
            }
            expect_selector = true;
            index = token_index + 1;
        }
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
        let should_wrap = self
            .current_kind()
            .is_some_and(|kind| !recovery.contains(&kind));
        if should_wrap {
            self.builder.start_node(SyntaxKind::BogusRecovery);
        }
        while !self.at_end() {
            match self.current_kind() {
                Some(kind) if recovery.contains(&kind) => break,
                Some(_) => self.token_current(),
                None => break,
            }
        }
        if should_wrap {
            self.builder.finish_node();
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

    fn find_rule_block_open_before_recovery(&self, recovery: &[SyntaxKind]) -> bool {
        let mut index = self.position;
        while let Some(token) = self.tokens.get(index) {
            if token.kind == SyntaxKind::LeftBrace
                || (self.dialect == StyleDialect::Sass && token.kind == SyntaxKind::SassIndent)
            {
                return true;
            }
            if recovery.contains(&token.kind) {
                return false;
            }
            index += 1;
        }
        false
    }

    fn find_text_before_recovery(&self, target: &str, recovery: &[SyntaxKind]) -> bool {
        let mut index = self.position;
        while let Some(token) = self.tokens.get(index) {
            if token.text == target {
                return true;
            }
            if recovery.contains(&token.kind) {
                return false;
            }
            index += 1;
        }
        false
    }

    fn current_function_has_closing_paren_before(&self, recovery: &[SyntaxKind]) -> bool {
        let Some(open_index) = self.position.checked_add(1) else {
            return false;
        };
        if self
            .tokens
            .get(open_index)
            .is_none_or(|token| token.kind != SyntaxKind::LeftParen)
        {
            return false;
        }

        let mut depth = 0usize;
        for token in self.tokens.iter().skip(open_index) {
            match token.kind {
                SyntaxKind::LeftParen => depth += 1,
                SyntaxKind::RightParen => {
                    depth = depth.saturating_sub(1);
                    if depth == 0 {
                        return true;
                    }
                }
                kind if depth == 1 && recovery.contains(&kind) => return false,
                _ => {}
            }
        }
        false
    }

    fn current_split_important_annotation(&self) -> bool {
        self.current_text() == Some("!")
            && self
                .non_trivia_token_from(self.position + 1)
                .is_some_and(|(index, kind)| {
                    matches!(kind, SyntaxKind::Ident | SyntaxKind::KeywordImportant)
                        && self
                            .tokens
                            .get(index)
                            .is_some_and(|token| token.text.eq_ignore_ascii_case("important"))
                })
    }

    fn current_scss_variable_flag_annotation(&self) -> bool {
        matches!(self.dialect, StyleDialect::Scss | StyleDialect::Sass)
            && self.current_text() == Some("!")
            && self
                .non_trivia_token_from(self.position + 1)
                .is_some_and(|(index, kind)| {
                    kind == SyntaxKind::Ident
                        && self.tokens.get(index).is_some_and(|token| {
                            token.text.eq_ignore_ascii_case("default")
                                || token.text.eq_ignore_ascii_case("global")
                        })
                })
    }

    fn current_bracketed_value_has_closing_bracket_before(&self, recovery: &[SyntaxKind]) -> bool {
        let mut depth = 0usize;
        for token in self.tokens.iter().skip(self.position) {
            match token.kind {
                SyntaxKind::LeftBracket => depth += 1,
                SyntaxKind::RightBracket => {
                    depth = depth.saturating_sub(1);
                    if depth == 0 {
                        return true;
                    }
                }
                kind if depth == 1 && recovery.contains(&kind) => return false,
                _ => {}
            }
        }
        false
    }

    fn current_simple_block_has_matching_close(&self, recovery: &[SyntaxKind]) -> bool {
        let Some(open_kind) = self.current_kind() else {
            return false;
        };
        if matching_simple_block_close(open_kind).is_none() {
            return false;
        }

        let mut expected_closes = Vec::new();
        for token in self.tokens.iter().skip(self.position) {
            if let Some(close_kind) = matching_simple_block_close(token.kind) {
                expected_closes.push(close_kind);
                continue;
            }

            if expected_closes.last().copied() == Some(token.kind) {
                expected_closes.pop();
                if expected_closes.is_empty() {
                    return true;
                }
                continue;
            }

            if expected_closes.len() == 1 && recovery.contains(&token.kind) {
                return false;
            }
        }
        false
    }

    fn current_dialect_at_rule_node_kind(&self, spec: AtRuleSpec) -> SyntaxKind {
        if !self.find_rule_block_open_before_recovery(&[
            SyntaxKind::Semicolon,
            SyntaxKind::SassOptionalSemicolon,
            SyntaxKind::RightBrace,
            SyntaxKind::SassDedent,
        ]) {
            return match spec.node_kind {
                SyntaxKind::ScssMixinDeclaration => SyntaxKind::BogusScssMixin,
                SyntaxKind::ScssFunctionDeclaration => SyntaxKind::BogusScssFunction,
                SyntaxKind::ScssControlIf
                | SyntaxKind::ScssControlElse
                | SyntaxKind::ScssControlEach
                | SyntaxKind::ScssControlFor
                | SyntaxKind::ScssControlWhile => SyntaxKind::BogusScssControl,
                _ => spec.node_kind,
            };
        }
        spec.node_kind
    }

    fn current_less_guard_has_condition_before(&self, recovery: &[SyntaxKind]) -> bool {
        let mut index = self.position + 1;
        while let Some(token) = self.tokens.get(index) {
            if recovery.contains(&token.kind) {
                return false;
            }
            if token.kind == SyntaxKind::LeftParen {
                return true;
            }
            index += 1;
        }
        false
    }

    fn current_scss_module_config_has_balanced_parens(&self) -> bool {
        let Some((_, SyntaxKind::LeftParen)) = self.non_trivia_token_from(self.position + 1) else {
            return false;
        };
        self.current_prelude_parentheses_are_balanced_until(&[
            SyntaxKind::Semicolon,
            SyntaxKind::SassOptionalSemicolon,
            SyntaxKind::LeftBrace,
            SyntaxKind::SassIndent,
        ])
    }

    fn current_value_has_top_level_comma_before(&self, recovery: &[SyntaxKind]) -> bool {
        let mut paren_depth = 0usize;
        let mut bracket_depth = 0usize;
        for token in self.tokens.iter().skip(self.position) {
            match token.kind {
                kind if paren_depth == 0 && bracket_depth == 0 && recovery.contains(&kind) => {
                    return false;
                }
                SyntaxKind::LeftParen => paren_depth += 1,
                SyntaxKind::RightParen => paren_depth = paren_depth.saturating_sub(1),
                SyntaxKind::LeftBracket => bracket_depth += 1,
                SyntaxKind::RightBracket => bracket_depth = bracket_depth.saturating_sub(1),
                SyntaxKind::Comma if paren_depth == 0 && bracket_depth == 0 => return true,
                _ => {}
            }
        }
        false
    }

    fn current_value_list_is_bogus(&self, recovery: &[SyntaxKind]) -> bool {
        let mut paren_depth = 0usize;
        let mut bracket_depth = 0usize;
        let mut expecting_item = true;
        for token in self.tokens.iter().skip(self.position) {
            if token.kind.is_trivia() {
                continue;
            }
            match token.kind {
                kind if paren_depth == 0 && bracket_depth == 0 && recovery.contains(&kind) => {
                    return expecting_item;
                }
                SyntaxKind::LeftParen => {
                    paren_depth += 1;
                    expecting_item = false;
                }
                SyntaxKind::RightParen => {
                    paren_depth = paren_depth.saturating_sub(1);
                    expecting_item = false;
                }
                SyntaxKind::LeftBracket => {
                    bracket_depth += 1;
                    expecting_item = false;
                }
                SyntaxKind::RightBracket => {
                    bracket_depth = bracket_depth.saturating_sub(1);
                    expecting_item = false;
                }
                SyntaxKind::Comma if paren_depth == 0 && bracket_depth == 0 => {
                    if expecting_item {
                        return true;
                    }
                    expecting_item = true;
                }
                _ => expecting_item = false,
            }
        }
        expecting_item
    }

    fn current_starts_missing_semicolon_declaration(&self, recovery: &[SyntaxKind]) -> bool {
        match self.current_kind() {
            Some(SyntaxKind::Ident | SyntaxKind::CustomPropertyName) => {}
            _ => return false,
        }

        let mut index = self.position + 1;
        while let Some(token) = self.tokens.get(index) {
            if token.kind.is_trivia() {
                index += 1;
                continue;
            }
            if recovery.contains(&token.kind) {
                return false;
            }
            return token.kind == SyntaxKind::Colon;
        }
        false
    }

    fn current_selector_item_is_bogus(&self, recovery: &[SyntaxKind]) -> bool {
        self.selector_item_is_bogus_from(self.position, recovery)
    }

    fn selector_item_is_bogus_from(&self, start: usize, recovery: &[SyntaxKind]) -> bool {
        let mut paren_depth = 0usize;
        let mut bracket_depth = 0usize;
        let mut saw_selector_token = false;

        for token in self.tokens.iter().skip(start) {
            if token.kind.is_trivia() {
                continue;
            }
            if paren_depth == 0
                && bracket_depth == 0
                && (token.kind == SyntaxKind::Comma
                    || is_selector_boundary_until(token.kind, recovery))
            {
                break;
            }

            match token.kind {
                SyntaxKind::LeftParen => paren_depth += 1,
                SyntaxKind::RightParen => paren_depth = paren_depth.saturating_sub(1),
                SyntaxKind::LeftBracket => bracket_depth += 1,
                SyntaxKind::RightBracket => bracket_depth = bracket_depth.saturating_sub(1),
                _ => {}
            }

            if !selector_item_token_is_recoverable(token.kind) {
                return true;
            }
            saw_selector_token = true;
        }

        !saw_selector_token
    }

    fn selector_list_contains_bogus_item_until(&self, recovery: &[SyntaxKind]) -> bool {
        let mut index = self.position;
        while let Some(token) = self.tokens.get(index) {
            if token.kind.is_trivia() || token.kind == SyntaxKind::Comma {
                index += 1;
                continue;
            }
            if is_selector_boundary_until(token.kind, recovery) {
                return false;
            }
            if self.selector_item_is_bogus_from(index, recovery) {
                return true;
            }

            let mut paren_depth = 0usize;
            let mut bracket_depth = 0usize;
            while let Some(token) = self.tokens.get(index) {
                if paren_depth == 0
                    && bracket_depth == 0
                    && (token.kind == SyntaxKind::Comma
                        || is_selector_boundary_until(token.kind, recovery))
                {
                    break;
                }
                match token.kind {
                    SyntaxKind::LeftParen => paren_depth += 1,
                    SyntaxKind::RightParen => paren_depth = paren_depth.saturating_sub(1),
                    SyntaxKind::LeftBracket => bracket_depth += 1,
                    SyntaxKind::RightBracket => bracket_depth = bracket_depth.saturating_sub(1),
                    _ => {}
                }
                index += 1;
            }
        }
        false
    }

    fn current_generic_at_rule_prelude_node_kind(&self) -> SyntaxKind {
        if self.current_prelude_parentheses_are_balanced_until(&[
            SyntaxKind::LeftBrace,
            SyntaxKind::Semicolon,
        ]) {
            SyntaxKind::AtRulePrelude
        } else {
            SyntaxKind::BogusAtRulePrelude
        }
    }

    fn current_prelude_parentheses_are_balanced_until(&self, recovery: &[SyntaxKind]) -> bool {
        let mut depth = 0usize;
        for token in self.tokens.iter().skip(self.position) {
            match token.kind {
                kind if depth == 0 && recovery.contains(&kind) => return true,
                SyntaxKind::LeftParen => depth += 1,
                SyntaxKind::RightParen => {
                    if depth == 0 {
                        return false;
                    }
                    depth -= 1;
                }
                _ => {}
            }
        }
        depth == 0
    }

    fn previous_left_brace_has_match(&self) -> bool {
        let Some(open_index) = self.position.checked_sub(1) else {
            return false;
        };
        let Some(open) = self.tokens.get(open_index) else {
            return false;
        };
        if open.kind != SyntaxKind::LeftBrace {
            return false;
        }

        let mut depth = 0usize;
        for token in self.tokens.iter().skip(open_index) {
            match token.kind {
                SyntaxKind::LeftBrace => depth += 1,
                SyntaxKind::RightBrace => {
                    depth = depth.saturating_sub(1);
                    if depth == 0 {
                        return true;
                    }
                }
                _ => {}
            }
        }
        false
    }

    fn current_starts_nested_rule(&self) -> bool {
        matches!(
            self.current_kind(),
            Some(
                SyntaxKind::Dot
                    | SyntaxKind::Hash
                    | SyntaxKind::Ampersand
                    | SyntaxKind::Colon
                    | SyntaxKind::DoubleColon
                    | SyntaxKind::LeftBracket
            )
        ) && self.find_rule_block_open_before_recovery(&[
            SyntaxKind::Colon,
            SyntaxKind::Semicolon,
            SyntaxKind::SassOptionalSemicolon,
            SyntaxKind::RightBrace,
            SyntaxKind::SassDedent,
        ])
    }

    fn current_starts_scss_nested_property(&self) -> bool {
        if !matches!(self.dialect, StyleDialect::Scss | StyleDialect::Sass) {
            return false;
        }
        if !matches!(
            self.current_kind(),
            Some(SyntaxKind::Ident | SyntaxKind::CustomPropertyName)
        ) {
            return false;
        }

        let mut saw_colon = false;
        for token in self.tokens.iter().skip(self.position) {
            match token.kind {
                SyntaxKind::Colon => saw_colon = true,
                SyntaxKind::LeftBrace if saw_colon => return true,
                SyntaxKind::SassIndent if saw_colon && self.dialect == StyleDialect::Sass => {
                    return true;
                }
                SyntaxKind::Semicolon
                | SyntaxKind::SassOptionalSemicolon
                | SyntaxKind::RightBrace
                | SyntaxKind::SassDedent => return false,
                _ => {}
            }
        }
        false
    }

    fn current_starts_less_mixin_declaration(&self) -> bool {
        self.dialect == StyleDialect::Less
            && self.current_starts_less_callable_signature()
            && self.find_before_recovery(
                SyntaxKind::LeftBrace,
                &[SyntaxKind::Semicolon, SyntaxKind::RightBrace],
            )
    }

    fn current_starts_less_mixin_call(&self) -> bool {
        self.dialect == StyleDialect::Less
            && self.current_starts_less_callable_signature()
            && !self.find_before_recovery(
                SyntaxKind::LeftBrace,
                &[SyntaxKind::Semicolon, SyntaxKind::RightBrace],
            )
    }

    fn current_starts_less_callable_signature(&self) -> bool {
        match self.current_kind() {
            Some(SyntaxKind::Dot) => {
                let Some((index, SyntaxKind::Ident | SyntaxKind::CustomPropertyName)) =
                    self.non_trivia_token_from(self.position + 1)
                else {
                    return false;
                };
                self.non_trivia_token_from(index + 1)
                    .is_some_and(|(_, kind)| kind == SyntaxKind::LeftParen)
            }
            Some(SyntaxKind::Hash) => self
                .non_trivia_token_from(self.position + 1)
                .is_some_and(|(_, kind)| kind == SyntaxKind::LeftParen),
            _ => false,
        }
    }

    fn current_starts_less_extend_rule(&self) -> bool {
        self.dialect == StyleDialect::Less
            && self.current_kind() == Some(SyntaxKind::Colon)
            && self
                .non_trivia_token_from(self.position + 1)
                .is_some_and(|(index, kind)| {
                    kind == SyntaxKind::Ident
                        && self
                            .tokens
                            .get(index)
                            .is_some_and(|token| token.text == "extend")
                })
    }

    fn current_starts_less_namespace_access(&self) -> bool {
        self.dialect == StyleDialect::Less
            && matches!(
                self.current_kind(),
                Some(SyntaxKind::Dot | SyntaxKind::Hash)
            )
            && self.find_before_recovery(
                SyntaxKind::GreaterThan,
                &[
                    SyntaxKind::Semicolon,
                    SyntaxKind::LeftBrace,
                    SyntaxKind::RightBrace,
                ],
            )
            && self.find_before_recovery(
                SyntaxKind::LeftParen,
                &[
                    SyntaxKind::Semicolon,
                    SyntaxKind::LeftBrace,
                    SyntaxKind::RightBrace,
                ],
            )
    }

    fn current_left_brace_has_match(&self) -> bool {
        let mut depth = 0usize;
        for token in self.tokens.iter().skip(self.position) {
            match token.kind {
                SyntaxKind::LeftBrace => depth += 1,
                SyntaxKind::RightBrace => {
                    depth = depth.saturating_sub(1);
                    if depth == 0 {
                        return true;
                    }
                }
                _ => {}
            }
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

    fn missing_token_bogus_trivia(&mut self, code: ParseErrorCode, message: &'static str) {
        self.builder.start_node(SyntaxKind::BogusTrivia);
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

    fn current_is_css_module_value_rule(&self) -> bool {
        self.current_text() == Some("@value")
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

    fn non_trivia_token_from(&self, mut index: usize) -> Option<(usize, SyntaxKind)> {
        while let Some(token) = self.tokens.get(index) {
            if !token.kind.is_trivia() {
                return Some((index, token.kind));
            }
            index += 1;
        }
        None
    }

    fn non_trivia_token_after_interpolation(
        &self,
        mut index: usize,
        start_kind: SyntaxKind,
    ) -> Option<(usize, SyntaxKind)> {
        let end_kind = interpolation_end_kind(start_kind)?;
        index += 1;
        while let Some(token) = self.tokens.get(index) {
            if token.kind == end_kind {
                return self.non_trivia_token_from(index + 1);
            }
            if is_at_rule_prelude_boundary(token.kind) {
                return None;
            }
            index += 1;
        }
        None
    }

    fn current_starts_namespace_qualified_selector(&self, kind: SyntaxKind) -> bool {
        match kind {
            SyntaxKind::Ident | SyntaxKind::Star => {
                self.next_kind() == Some(SyntaxKind::Pipe)
                    && self
                        .tokens
                        .get(self.position + 2)
                        .is_some_and(|token| namespace_selector_target_can_start(token.kind))
            }
            SyntaxKind::Pipe => self
                .tokens
                .get(self.position + 1)
                .is_some_and(|token| namespace_selector_target_can_start(token.kind)),
            _ => false,
        }
    }

    fn namespace_qualified_selector_target_kind(&self) -> Option<SyntaxKind> {
        let target_index = if self.current_kind() == Some(SyntaxKind::Pipe) {
            self.position + 1
        } else {
            self.position + 2
        };
        self.tokens.get(target_index).map(|token| token.kind)
    }

    fn at_end(&self) -> bool {
        self.position >= self.tokens.len()
    }
}

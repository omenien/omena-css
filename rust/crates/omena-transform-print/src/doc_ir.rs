use omena_transform_cst::{
    IrEditRegionV0, IrNodeKindV0, IrTransactionV0, StyleDialect, TransformIrV0,
    lower_transform_ir_from_source, materialize_transform_ir_printed_source,
};
use serde::Serialize;
use std::collections::BTreeSet;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PrettyFormatOptionsV0 {
    pub line_width: usize,
    pub indent_width: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum FormatIrCoverageStrategyV0 {
    Structured,
    VerbatimLeaf,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FormatIrCoverageEntryV0 {
    pub node_kind: &'static str,
    pub strategy: FormatIrCoverageStrategyV0,
}

pub const FORMAT_IR_COVERAGE_MANIFEST_V0: &[FormatIrCoverageEntryV0] = &[
    FormatIrCoverageEntryV0 {
        node_kind: "style-rule",
        strategy: FormatIrCoverageStrategyV0::Structured,
    },
    FormatIrCoverageEntryV0 {
        node_kind: "at-rule",
        strategy: FormatIrCoverageStrategyV0::Structured,
    },
    FormatIrCoverageEntryV0 {
        node_kind: "declaration",
        strategy: FormatIrCoverageStrategyV0::Structured,
    },
    FormatIrCoverageEntryV0 {
        node_kind: "selector",
        strategy: FormatIrCoverageStrategyV0::Structured,
    },
    FormatIrCoverageEntryV0 {
        node_kind: "value",
        strategy: FormatIrCoverageStrategyV0::VerbatimLeaf,
    },
    FormatIrCoverageEntryV0 {
        node_kind: "url-value",
        strategy: FormatIrCoverageStrategyV0::VerbatimLeaf,
    },
];

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PrettyFormatReportV0 {
    pub schema_version: &'static str,
    pub product: &'static str,
    pub line_width: usize,
    pub indent_width: usize,
    pub covered_node_count: usize,
    pub structured_node_count: usize,
    pub verbatim_leaf_count: usize,
    pub fallback_node_count: usize,
    pub edit_count: usize,
    pub fallback_reasons: Vec<&'static str>,
    pub coverage_manifest: Vec<FormatIrCoverageEntryV0>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct PrettyRenderResultV0 {
    pub css: String,
    pub generated_offset_lookup: Vec<usize>,
    pub report: PrettyFormatReportV0,
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum FormatDocV0 {
    Text {
        value: String,
        source_span: Option<(usize, usize)>,
    },
    Sequence(Vec<FormatDocV0>),
    Group(Vec<FormatDocV0>),
    Indent(Vec<FormatDocV0>),
    SoftLine,
    HardLine,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum TokenKindV0 {
    Text,
    Whitespace,
    Comment,
    OpenBrace,
    CloseBrace,
    Semicolon,
    Comma,
    Colon,
    OpenParen,
    CloseParen,
    OpenBracket,
    CloseBracket,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct FormatTokenV0 {
    kind: TokenKindV0,
    text: String,
    source_start: usize,
    source_end: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct RenderedAnchorV0 {
    original_start: usize,
    original_end: usize,
    generated_start: usize,
    generated_end: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct FormatEditV0 {
    root_node_index: usize,
    source_start: usize,
    source_end: usize,
    replacement: String,
    anchors: Vec<RenderedAnchorV0>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum RenderModeV0 {
    Flat,
    Break,
}

struct RenderStateV0 {
    output: String,
    column: usize,
    pending_indent: usize,
    anchors: Vec<RenderedAnchorV0>,
    options: PrettyFormatOptionsV0,
}

pub const fn default_pretty_format_options() -> PrettyFormatOptionsV0 {
    PrettyFormatOptionsV0 {
        line_width: 100,
        indent_width: 2,
    }
}

pub(crate) fn render_pretty_css_through_transform_ir(
    source: &str,
    dialect: StyleDialect,
    source_id: &str,
    options: PrettyFormatOptionsV0,
) -> PrettyRenderResultV0 {
    let mut ir = lower_transform_ir_from_source(source, dialect, source_id);
    let (structured_node_count, verbatim_leaf_count) = coverage_counts(&ir);
    let covered_node_count = structured_node_count + verbatim_leaf_count;
    let mut report = PrettyFormatReportV0 {
        schema_version: "0",
        product: "omena-transform-print.pretty-format-report",
        line_width: options.line_width,
        indent_width: options.indent_width,
        covered_node_count,
        structured_node_count,
        verbatim_leaf_count,
        fallback_node_count: 0,
        edit_count: 0,
        fallback_reasons: Vec::new(),
        coverage_manifest: FORMAT_IR_COVERAGE_MANIFEST_V0.to_vec(),
    };

    if dialect == StyleDialect::Sass {
        report.fallback_node_count = ir.nodes.len();
        report
            .fallback_reasons
            .push("indented-sass-stable-fallback");
        return stable_result(source, report);
    }
    if ir.parser_error_count > 0 {
        report.fallback_node_count = ir.nodes.len();
        report.fallback_reasons.push("parse-error-stable-fallback");
        return stable_result(source, report);
    }

    let edits = build_format_edits(source, &ir, options);
    if edits.is_empty() {
        report.fallback_node_count = ir.nodes.len();
        report
            .fallback_reasons
            .push("no-structured-root-stable-fallback");
        return stable_result(source, report);
    }
    report.edit_count = edits.len();

    let node_count = ir.nodes.len();
    let mut transaction =
        IrTransactionV0::new(&mut ir, "pretty-format", IrEditRegionV0::full(source.len()));
    for edit in &edits {
        let node_id = omena_transform_cst::IrNodeIdV0(edit.root_node_index);
        if transaction
            .replace_node_covering_span(
                node_id,
                edit.replacement.clone(),
                edit.source_start,
                edit.source_end,
            )
            .is_err()
        {
            report.fallback_node_count = node_count;
            report.fallback_reasons.push("edit-plan-stable-fallback");
            return stable_result(source, report);
        }
    }
    if transaction.commit().is_err() {
        report.fallback_node_count = node_count;
        report.fallback_reasons.push("transaction-stable-fallback");
        return stable_result(source, report);
    }
    let Ok(css) = materialize_transform_ir_printed_source(&mut ir) else {
        report.fallback_node_count = ir.nodes.len();
        report
            .fallback_reasons
            .push("materialization-stable-fallback");
        return stable_result(source, report);
    };

    let anchors = edits
        .iter()
        .scan(0usize, |generated_base, edit| {
            let base = *generated_base;
            *generated_base += edit.replacement.len();
            Some(edit.anchors.iter().map(move |anchor| RenderedAnchorV0 {
                original_start: anchor.original_start,
                original_end: anchor.original_end,
                generated_start: base + anchor.generated_start,
                generated_end: base + anchor.generated_end,
            }))
        })
        .flatten()
        .collect::<Vec<_>>();
    let generated_offset_lookup = generated_lookup_from_anchors(source.len(), css.len(), &anchors);
    PrettyRenderResultV0 {
        css,
        generated_offset_lookup,
        report,
    }
}

fn stable_result(source: &str, report: PrettyFormatReportV0) -> PrettyRenderResultV0 {
    PrettyRenderResultV0 {
        css: source.to_string(),
        generated_offset_lookup: (0..=source.len()).collect(),
        report,
    }
}

fn coverage_counts(ir: &TransformIrV0) -> (usize, usize) {
    ir.nodes.iter().fold(
        (0, 0),
        |(structured, verbatim), node| match coverage_strategy(node.kind) {
            FormatIrCoverageStrategyV0::Structured => (structured + 1, verbatim),
            FormatIrCoverageStrategyV0::VerbatimLeaf => (structured, verbatim + 1),
        },
    )
}

const fn coverage_strategy(kind: IrNodeKindV0) -> FormatIrCoverageStrategyV0 {
    match kind {
        IrNodeKindV0::StyleRule
        | IrNodeKindV0::AtRule
        | IrNodeKindV0::Declaration
        | IrNodeKindV0::Selector => FormatIrCoverageStrategyV0::Structured,
        IrNodeKindV0::Value | IrNodeKindV0::UrlValue => FormatIrCoverageStrategyV0::VerbatimLeaf,
    }
}

fn build_format_edits(
    source: &str,
    ir: &TransformIrV0,
    options: PrettyFormatOptionsV0,
) -> Vec<FormatEditV0> {
    let mut root_nodes = ir
        .root_nodes
        .iter()
        .filter_map(|node_id| ir.nodes.get(node_id.index()))
        .filter(|node| matches!(node.kind, IrNodeKindV0::StyleRule | IrNodeKindV0::AtRule))
        .collect::<Vec<_>>();
    root_nodes.sort_by_key(|node| (node.source_span_start, node.source_span_end));
    let declaration_colons = declaration_colon_offsets(source, ir);
    let root_count = root_nodes.len();

    root_nodes
        .iter()
        .enumerate()
        .filter_map(|(index, root)| {
            let source_start = if index == 0 {
                0
            } else {
                root.source_span_start
            };
            let source_end = root_nodes
                .get(index + 1)
                .map_or(source.len(), |next| next.source_span_start);
            let chunk = source.get(source_start..source_end)?;
            let tokens = tokenize(chunk, source_start);
            let document = document_from_tokens(&tokens, &declaration_colons);
            let mut rendered = render_document(&document, options);
            while rendered.output.ends_with([' ', '\t', '\n', '\r']) {
                rendered.output.pop();
            }
            rendered.output.push('\n');
            if index + 1 < root_count {
                rendered.output.push('\n');
            }
            Some(FormatEditV0 {
                root_node_index: root.node_id.index(),
                source_start,
                source_end,
                replacement: rendered.output,
                anchors: rendered.anchors,
            })
        })
        .collect()
}

fn declaration_colon_offsets(source: &str, ir: &TransformIrV0) -> BTreeSet<usize> {
    ir.nodes
        .iter()
        .filter(|node| node.kind == IrNodeKindV0::Declaration)
        .filter_map(|node| {
            let declaration = source.get(node.source_span_start..node.source_span_end)?;
            first_unquoted_colon(declaration).map(|offset| node.source_span_start + offset)
        })
        .collect()
}

fn first_unquoted_colon(source: &str) -> Option<usize> {
    let bytes = source.as_bytes();
    let mut index = 0;
    let mut quote = None;
    while index < bytes.len() {
        match (quote, bytes[index]) {
            (Some(_), b'\\') => index = (index + 2).min(bytes.len()),
            (Some(active), byte) if byte == active => {
                quote = None;
                index += 1;
            }
            (Some(_), _) => index += 1,
            (None, b'\'' | b'"') => {
                quote = Some(bytes[index]);
                index += 1;
            }
            (None, b':') => return Some(index),
            _ => index += 1,
        }
    }
    None
}

fn tokenize(source: &str, source_base: usize) -> Vec<FormatTokenV0> {
    let bytes = source.as_bytes();
    let mut tokens = Vec::new();
    let mut index = 0;
    while index < bytes.len() {
        let start = index;
        let (kind, end) = match bytes[index] {
            byte if byte.is_ascii_whitespace() => {
                index += 1;
                while index < bytes.len() && bytes[index].is_ascii_whitespace() {
                    index += 1;
                }
                (TokenKindV0::Whitespace, index)
            }
            b'/' if bytes.get(index + 1) == Some(&b'*') => {
                index += 2;
                while index + 1 < bytes.len() && !(bytes[index] == b'*' && bytes[index + 1] == b'/')
                {
                    index += 1;
                }
                index = (index + 2).min(bytes.len());
                (TokenKindV0::Comment, index)
            }
            b'/' if bytes.get(index + 1) == Some(&b'/') => {
                index += 2;
                while index < bytes.len() && bytes[index] != b'\n' {
                    index += 1;
                }
                (TokenKindV0::Comment, index)
            }
            b'\'' | b'"' => {
                let quote = bytes[index];
                index += 1;
                while index < bytes.len() {
                    if bytes[index] == b'\\' {
                        index = (index + 2).min(bytes.len());
                    } else if bytes[index] == quote {
                        index += 1;
                        break;
                    } else {
                        index += 1;
                    }
                }
                (TokenKindV0::Text, index)
            }
            b'#' | b'@' if bytes.get(index + 1) == Some(&b'{') => {
                index = scan_interpolation(bytes, index);
                (TokenKindV0::Text, index)
            }
            b'{' => (TokenKindV0::OpenBrace, index + 1),
            b'}' => (TokenKindV0::CloseBrace, index + 1),
            b';' => (TokenKindV0::Semicolon, index + 1),
            b',' => (TokenKindV0::Comma, index + 1),
            b':' => (TokenKindV0::Colon, index + 1),
            b'(' => (TokenKindV0::OpenParen, index + 1),
            b')' => (TokenKindV0::CloseParen, index + 1),
            b'[' => (TokenKindV0::OpenBracket, index + 1),
            b']' => (TokenKindV0::CloseBracket, index + 1),
            _ => {
                index += 1;
                while index < bytes.len() {
                    let byte = bytes[index];
                    let starts_comment =
                        byte == b'/' && matches!(bytes.get(index + 1), Some(b'*') | Some(b'/'));
                    if byte.is_ascii_whitespace()
                        || matches!(
                            byte,
                            b'{' | b'}'
                                | b';'
                                | b','
                                | b':'
                                | b'('
                                | b')'
                                | b'['
                                | b']'
                                | b'\''
                                | b'"'
                        )
                        || starts_comment
                    {
                        break;
                    }
                    index += 1;
                }
                (TokenKindV0::Text, index)
            }
        };
        index = end;
        tokens.push(FormatTokenV0 {
            kind,
            text: source[start..end].to_string(),
            source_start: source_base + start,
            source_end: source_base + end,
        });
    }
    tokens
}

fn scan_interpolation(bytes: &[u8], start: usize) -> usize {
    let mut depth = 1usize;
    let mut index = start + 2;
    while index < bytes.len() && depth > 0 {
        match bytes[index] {
            b'\\' => index = (index + 2).min(bytes.len()),
            b'{' => {
                depth += 1;
                index += 1;
            }
            b'}' => {
                depth -= 1;
                index += 1;
            }
            _ => index += 1,
        }
    }
    index
}

fn document_from_tokens(
    tokens: &[FormatTokenV0],
    declaration_colons: &BTreeSet<usize>,
) -> FormatDocV0 {
    let mut index = 0;
    FormatDocV0::Sequence(parse_document_sequence(
        tokens,
        &mut index,
        declaration_colons,
        false,
    ))
}

fn parse_document_sequence(
    tokens: &[FormatTokenV0],
    index: &mut usize,
    declaration_colons: &BTreeSet<usize>,
    stop_at_close: bool,
) -> Vec<FormatDocV0> {
    let mut output = Vec::new();
    let mut inline = Vec::new();
    let mut paren_depth = 0usize;
    let mut bracket_depth = 0usize;

    while *index < tokens.len() {
        let token = &tokens[*index];
        let structural = paren_depth == 0 && bracket_depth == 0;
        match token.kind {
            TokenKindV0::CloseBrace if structural && stop_at_close => {
                flush_inline(&mut output, &mut inline, declaration_colons);
                *index += 1;
                break;
            }
            TokenKindV0::OpenBrace if structural => {
                trim_inline_whitespace(&mut inline);
                flush_inline(&mut output, &mut inline, declaration_colons);
                push_text(&mut output, " {", Some(token));
                *index += 1;
                let body = parse_document_sequence(tokens, index, declaration_colons, true);
                if body.is_empty() {
                    if let Some(FormatDocV0::Text { value, .. }) = output.last_mut() {
                        value.push('}');
                    } else {
                        push_text(&mut output, "}", None);
                    }
                } else {
                    output.push(FormatDocV0::Indent({
                        let mut indented = vec![FormatDocV0::HardLine];
                        indented.extend(body);
                        indented
                    }));
                    push_hard_line(&mut output);
                    push_text(&mut output, "}", None);
                }
                if *index < tokens.len()
                    && !(stop_at_close && tokens[*index].kind == TokenKindV0::CloseBrace)
                {
                    push_hard_line(&mut output);
                }
            }
            TokenKindV0::Semicolon if structural => {
                trim_inline_whitespace(&mut inline);
                flush_inline(&mut output, &mut inline, declaration_colons);
                push_text(&mut output, ";", Some(token));
                push_hard_line(&mut output);
                *index += 1;
            }
            TokenKindV0::OpenParen => {
                paren_depth += 1;
                inline.push(token.clone());
                *index += 1;
            }
            TokenKindV0::CloseParen => {
                paren_depth = paren_depth.saturating_sub(1);
                inline.push(token.clone());
                *index += 1;
            }
            TokenKindV0::OpenBracket => {
                bracket_depth += 1;
                inline.push(token.clone());
                *index += 1;
            }
            TokenKindV0::CloseBracket => {
                bracket_depth = bracket_depth.saturating_sub(1);
                inline.push(token.clone());
                *index += 1;
            }
            TokenKindV0::Comment
                if inline
                    .iter()
                    .all(|item| item.kind == TokenKindV0::Whitespace) =>
            {
                inline.clear();
                push_text(&mut output, token.text.as_str(), Some(token));
                push_hard_line(&mut output);
                *index += 1;
            }
            TokenKindV0::CloseBrace if structural => {
                *index += 1;
            }
            _ => {
                inline.push(token.clone());
                *index += 1;
            }
        }
    }
    flush_inline(&mut output, &mut inline, declaration_colons);
    trim_trailing_lines(&mut output);
    output
}

fn flush_inline(
    output: &mut Vec<FormatDocV0>,
    inline: &mut Vec<FormatTokenV0>,
    declaration_colons: &BTreeSet<usize>,
) {
    trim_inline_whitespace(inline);
    if inline.is_empty() {
        return;
    }
    output.push(FormatDocV0::Group(inline_document(
        inline,
        declaration_colons,
    )));
    inline.clear();
}

fn inline_document(
    tokens: &[FormatTokenV0],
    declaration_colons: &BTreeSet<usize>,
) -> Vec<FormatDocV0> {
    let mut output = Vec::new();
    let mut suppress_whitespace = false;
    for token in tokens {
        match token.kind {
            TokenKindV0::Whitespace => {
                if !suppress_whitespace {
                    push_soft_line(&mut output);
                }
            }
            TokenKindV0::Comma => {
                trim_trailing_soft_lines(&mut output);
                push_text(&mut output, token.text.as_str(), Some(token));
                push_soft_line(&mut output);
                suppress_whitespace = true;
                continue;
            }
            TokenKindV0::Colon if declaration_colons.contains(&token.source_start) => {
                trim_trailing_soft_lines(&mut output);
                push_text(&mut output, token.text.as_str(), Some(token));
                push_soft_line(&mut output);
                suppress_whitespace = true;
                continue;
            }
            TokenKindV0::Colon => {
                trim_trailing_soft_lines(&mut output);
                push_text(&mut output, token.text.as_str(), Some(token));
                suppress_whitespace = true;
                continue;
            }
            TokenKindV0::CloseParen | TokenKindV0::CloseBracket => {
                trim_trailing_soft_lines(&mut output);
                push_text(&mut output, token.text.as_str(), Some(token));
            }
            TokenKindV0::OpenParen | TokenKindV0::OpenBracket => {
                trim_trailing_soft_lines(&mut output);
                push_text(&mut output, token.text.as_str(), Some(token));
                suppress_whitespace = true;
                continue;
            }
            _ => push_text(&mut output, token.text.as_str(), Some(token)),
        }
        suppress_whitespace = false;
    }
    trim_trailing_soft_lines(&mut output);
    output
}

fn trim_inline_whitespace(tokens: &mut Vec<FormatTokenV0>) {
    while tokens
        .last()
        .is_some_and(|token| token.kind == TokenKindV0::Whitespace)
    {
        tokens.pop();
    }
    let first_non_whitespace = tokens
        .iter()
        .position(|token| token.kind != TokenKindV0::Whitespace)
        .unwrap_or(tokens.len());
    tokens.drain(..first_non_whitespace);
}

fn push_text(output: &mut Vec<FormatDocV0>, value: &str, token: Option<&FormatTokenV0>) {
    output.push(FormatDocV0::Text {
        value: value.to_string(),
        source_span: token.map(|token| (token.source_start, token.source_end)),
    });
}

fn push_soft_line(output: &mut Vec<FormatDocV0>) {
    if !output
        .last()
        .is_some_and(|item| matches!(item, FormatDocV0::SoftLine | FormatDocV0::HardLine))
    {
        output.push(FormatDocV0::SoftLine);
    }
}

fn push_hard_line(output: &mut Vec<FormatDocV0>) {
    trim_trailing_soft_lines(output);
    if !output
        .last()
        .is_some_and(|item| matches!(item, FormatDocV0::HardLine))
    {
        output.push(FormatDocV0::HardLine);
    }
}

fn trim_trailing_soft_lines(output: &mut Vec<FormatDocV0>) {
    while output
        .last()
        .is_some_and(|item| matches!(item, FormatDocV0::SoftLine))
    {
        output.pop();
    }
}

fn trim_trailing_lines(output: &mut Vec<FormatDocV0>) {
    while output
        .last()
        .is_some_and(|item| matches!(item, FormatDocV0::SoftLine | FormatDocV0::HardLine))
    {
        output.pop();
    }
}

fn render_document(document: &FormatDocV0, options: PrettyFormatOptionsV0) -> RenderStateV0 {
    let mut state = RenderStateV0 {
        output: String::new(),
        column: 0,
        pending_indent: 0,
        anchors: Vec::new(),
        options,
    };
    render_doc(document, RenderModeV0::Break, 0, &mut state);
    state
}

fn render_doc(doc: &FormatDocV0, mode: RenderModeV0, indent: usize, state: &mut RenderStateV0) {
    match doc {
        FormatDocV0::Text { value, source_span } => {
            write_indent_if_needed(state);
            let generated_start = state.output.len();
            state.output.push_str(value);
            state.column += value.chars().count();
            if let Some((original_start, original_end)) = source_span {
                state.anchors.push(RenderedAnchorV0 {
                    original_start: *original_start,
                    original_end: *original_end,
                    generated_start,
                    generated_end: state.output.len(),
                });
            }
        }
        FormatDocV0::Sequence(items) => render_sequence(items, mode, indent, state),
        FormatDocV0::Group(items) => {
            let remaining = state.options.line_width.saturating_sub(state.column);
            let group_mode = if flat_width(items).is_some_and(|width| width <= remaining) {
                RenderModeV0::Flat
            } else {
                RenderModeV0::Break
            };
            render_sequence(items, group_mode, indent, state);
        }
        FormatDocV0::Indent(items) => {
            render_sequence(items, mode, indent + state.options.indent_width, state);
        }
        FormatDocV0::SoftLine => render_soft_line(mode, indent, 0, state),
        FormatDocV0::HardLine => write_newline(indent, state),
    }
}

fn render_sequence(
    items: &[FormatDocV0],
    mode: RenderModeV0,
    indent: usize,
    state: &mut RenderStateV0,
) {
    for (index, item) in items.iter().enumerate() {
        if matches!(item, FormatDocV0::SoftLine) {
            let next_width = flat_width_until_break(&items[index + 1..]);
            render_soft_line(mode, indent, next_width, state);
        } else {
            render_doc(item, mode, indent, state);
        }
    }
}

fn render_soft_line(
    mode: RenderModeV0,
    indent: usize,
    next_width: usize,
    state: &mut RenderStateV0,
) {
    if mode == RenderModeV0::Flat || state.column + 1 + next_width <= state.options.line_width {
        if !state.output.ends_with([' ', '\n']) {
            state.output.push(' ');
            state.column += 1;
        }
    } else {
        write_newline(indent, state);
    }
}

fn write_newline(indent: usize, state: &mut RenderStateV0) {
    while state.output.ends_with([' ', '\t']) {
        state.output.pop();
    }
    if !state.output.ends_with('\n') {
        state.output.push('\n');
    }
    state.column = 0;
    state.pending_indent = indent;
}

fn write_indent_if_needed(state: &mut RenderStateV0) {
    if state.column == 0 && state.pending_indent > 0 {
        state
            .output
            .extend(std::iter::repeat_n(' ', state.pending_indent));
        state.column = state.pending_indent;
        state.pending_indent = 0;
    }
}

fn flat_width(items: &[FormatDocV0]) -> Option<usize> {
    items.iter().try_fold(0usize, |width, item| {
        flat_doc_width(item).map(|item_width| width + item_width)
    })
}

fn flat_doc_width(doc: &FormatDocV0) -> Option<usize> {
    match doc {
        FormatDocV0::Text { value, .. } => Some(value.chars().count()),
        FormatDocV0::Sequence(items) | FormatDocV0::Group(items) | FormatDocV0::Indent(items) => {
            flat_width(items)
        }
        FormatDocV0::SoftLine => Some(1),
        FormatDocV0::HardLine => None,
    }
}

fn flat_width_until_break(items: &[FormatDocV0]) -> usize {
    let mut width = 0;
    for item in items {
        match item {
            FormatDocV0::SoftLine | FormatDocV0::HardLine => break,
            _ => match flat_doc_width(item) {
                Some(item_width) => width += item_width,
                None => break,
            },
        }
    }
    width
}

fn generated_lookup_from_anchors(
    source_len: usize,
    generated_len: usize,
    anchors: &[RenderedAnchorV0],
) -> Vec<usize> {
    let mut anchors = anchors.to_vec();
    anchors.sort_by_key(|anchor| (anchor.original_start, anchor.original_end));
    let mut lookup = vec![0usize; source_len + 1];
    let mut original_cursor = 0usize;
    let mut generated_cursor = 0usize;

    for anchor in anchors {
        if anchor.original_start < original_cursor || anchor.original_end > source_len {
            continue;
        }
        fill_lookup_range(
            &mut lookup,
            original_cursor,
            anchor.original_start,
            generated_cursor,
            anchor.generated_start,
        );
        fill_lookup_range(
            &mut lookup,
            anchor.original_start,
            anchor.original_end,
            anchor.generated_start,
            anchor.generated_end,
        );
        original_cursor = anchor.original_end;
        generated_cursor = anchor.generated_end;
    }
    fill_lookup_range(
        &mut lookup,
        original_cursor,
        source_len,
        generated_cursor,
        generated_len,
    );
    for index in 1..lookup.len() {
        lookup[index] = lookup[index].max(lookup[index - 1]).min(generated_len);
    }
    lookup
}

fn fill_lookup_range(
    lookup: &mut [usize],
    source_start: usize,
    source_end: usize,
    generated_start: usize,
    generated_end: usize,
) {
    if source_start >= lookup.len() || source_end >= lookup.len() || source_start > source_end {
        return;
    }
    let source_len = source_end.saturating_sub(source_start);
    let generated_len = generated_end.saturating_sub(generated_start);
    for (relative, mapped_offset) in lookup[source_start..=source_end].iter_mut().enumerate() {
        let generated_relative = generated_len
            .saturating_mul(relative)
            .checked_div(source_len)
            .unwrap_or(0);
        *mapped_offset = generated_start + generated_relative;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn width_budget_changes_selector_list_layout() {
        let source = ".componentAlphaLongerState, .componentBetaLongerState, .componentGammaLongerState, .componentDeltaLongerState, .componentEpsilonLongerState { color:red; }";
        let narrow = render_pretty_css_through_transform_ir(
            source,
            StyleDialect::Css,
            "width-narrow",
            PrettyFormatOptionsV0 {
                line_width: 80,
                indent_width: 2,
            },
        );
        let medium = render_pretty_css_through_transform_ir(
            source,
            StyleDialect::Css,
            "width-medium",
            PrettyFormatOptionsV0 {
                line_width: 100,
                indent_width: 2,
            },
        );
        let wide = render_pretty_css_through_transform_ir(
            source,
            StyleDialect::Css,
            "width-wide",
            PrettyFormatOptionsV0 {
                line_width: 120,
                indent_width: 2,
            },
        );

        assert_ne!(narrow.css, medium.css);
        assert_ne!(medium.css, wide.css);
        assert_ne!(narrow.css, wide.css);
    }

    #[test]
    fn comments_strings_and_custom_property_values_are_preserved() {
        let source = "/* lead */ .card,.panel{--label:\"a,b\";color:var(--brand);/* tail */}";
        let rendered = render_pretty_css_through_transform_ir(
            source,
            StyleDialect::Css,
            "trivia",
            default_pretty_format_options(),
        );

        assert!(rendered.css.contains("/* lead */"));
        assert!(rendered.css.contains("/* tail */"));
        assert!(rendered.css.contains("\"a,b\""));
        assert!(rendered.css.contains("var(--brand)"));
    }

    #[test]
    fn indented_sass_and_parse_errors_report_stable_fallback() {
        let sass = render_pretty_css_through_transform_ir(
            ".card\n  color: red\n",
            StyleDialect::Sass,
            "sass",
            default_pretty_format_options(),
        );
        assert_eq!(sass.css, ".card\n  color: red\n");
        assert_eq!(
            sass.report.fallback_reasons,
            vec!["indented-sass-stable-fallback"]
        );

        let invalid = render_pretty_css_through_transform_ir(
            ".card { color: red;",
            StyleDialect::Css,
            "invalid",
            default_pretty_format_options(),
        );
        assert_eq!(invalid.css, ".card { color: red;");
        assert_eq!(
            invalid.report.fallback_reasons,
            vec!["parse-error-stable-fallback"]
        );
    }

    #[test]
    fn coverage_manifest_classifies_every_transform_ir_kind() {
        let labels = FORMAT_IR_COVERAGE_MANIFEST_V0
            .iter()
            .map(|entry| entry.node_kind)
            .collect::<BTreeSet<_>>();
        assert_eq!(
            labels,
            BTreeSet::from([
                "at-rule",
                "declaration",
                "selector",
                "style-rule",
                "url-value",
                "value",
            ])
        );
    }
}

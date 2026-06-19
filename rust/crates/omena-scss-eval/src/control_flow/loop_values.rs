use omena_parser::{StyleDialect, lex};
use omena_syntax::SyntaxKind;

use super::{
    model::OmenaScssEvalControlFlowBlockV0,
    tokens::next_non_trivia_token_index,
    variables::{canonical_scss_variable_name, variable_names_in_text_preserving_order},
};

pub(super) fn loop_carried_bindings(header: &str) -> Vec<String> {
    let separator = if header
        .split_whitespace()
        .any(|part| part.eq_ignore_ascii_case("from"))
    {
        "from"
    } else {
        "in"
    };
    let before_separator = split_header_at_keyword(header, separator)
        .map(|(left, _)| left)
        .unwrap_or(header);
    variable_names_in_text_preserving_order(before_separator)
}

pub(super) fn split_header_at_keyword<'a>(
    header: &'a str,
    keyword: &str,
) -> Option<(&'a str, &'a str)> {
    let lower_header = header.to_ascii_lowercase();
    let lower_keyword = keyword.to_ascii_lowercase();
    let mut search_start = 0usize;
    while search_start < lower_header.len() {
        let relative_index = lower_header
            .get(search_start..)?
            .find(lower_keyword.as_str())?;
        let index = search_start + relative_index;
        let right_start = index + keyword.len();
        if header_keyword_has_boundaries(header, index, right_start) {
            let left = header.get(..index)?;
            let right = header.get(right_start..)?;
            return Some((left, right));
        }
        search_start = right_start;
    }
    None
}

pub(super) fn while_loop_body_assignment_names(
    source: &str,
    block: &OmenaScssEvalControlFlowBlockV0,
) -> Vec<String> {
    let Some(body) = control_flow_block_body_text(source, block) else {
        return Vec::new();
    };
    let lexed = lex(body, StyleDialect::Scss);
    let tokens = lexed.tokens();
    let mut names: Vec<String> = Vec::new();
    for (index, token) in tokens.iter().enumerate() {
        if token.kind != SyntaxKind::ScssVariable {
            continue;
        }
        let Some(colon_index) = next_non_trivia_token_index(tokens, index + 1) else {
            continue;
        };
        if tokens[colon_index].kind != SyntaxKind::Colon {
            continue;
        }
        let name = token.text.to_string();
        if !names.iter().any(|existing| {
            canonical_scss_variable_name(existing.as_str())
                == canonical_scss_variable_name(name.as_str())
        }) {
            names.push(name);
        }
    }
    names
}

fn header_keyword_has_boundaries(header: &str, start: usize, end: usize) -> bool {
    let before_ok = header.get(..start).is_none_or(|text| {
        text.chars()
            .next_back()
            .is_none_or(|ch| ch.is_ascii_whitespace())
    });
    let after_ok = header.get(end..).is_none_or(|text| {
        text.chars()
            .next()
            .is_none_or(|ch| ch.is_ascii_whitespace())
    });
    before_ok && after_ok
}

fn control_flow_block_body_text<'a>(
    source: &'a str,
    block: &OmenaScssEvalControlFlowBlockV0,
) -> Option<&'a str> {
    let block_text = source.get(block.source_span_start..block.source_span_end)?;
    let open = block_text.find('{')?;
    let close = block_text.rfind('}')?;
    (open < close)
        .then(|| block_text.get(open + '{'.len_utf8()..close))
        .flatten()
}

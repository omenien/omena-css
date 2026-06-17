use omena_parser::StyleDialect;
use omena_syntax::SyntaxKind;

use crate::runtime::lex_cache::lex_cached as lex;

pub(crate) fn remove_source_ranges(source: &str, ranges: &[(usize, usize)]) -> (String, usize) {
    if ranges.is_empty() {
        return (source.to_string(), 0);
    }

    let mut ranges = ranges.to_vec();
    ranges.sort_by_key(|(start, _)| *start);
    let mut output = String::with_capacity(source.len());
    let mut cursor = 0;
    let mut removed_count = 0;
    for (start, end) in &ranges {
        if *start < cursor {
            continue;
        }
        if *start > cursor {
            output.push_str(&source[cursor..*start]);
        }
        cursor = *end;
        removed_count += 1;
    }
    if cursor < source.len() {
        output.push_str(&source[cursor..]);
    }

    (output, removed_count)
}

pub(crate) fn replace_source_ranges(
    source: &str,
    replacements: &[(usize, usize, String)],
) -> (String, usize) {
    if replacements.is_empty() {
        return (source.to_string(), 0);
    }

    let mut replacements = replacements.to_vec();
    replacements.sort_by_key(|(start, _, _)| *start);
    let mut output = String::with_capacity(source.len());
    let mut cursor = 0;
    let mut replacement_count = 0;
    for (start, end, replacement) in &replacements {
        if *start < cursor {
            continue;
        }
        if *start > cursor {
            output.push_str(&source[cursor..*start]);
        }
        output.push_str(replacement);
        cursor = *end;
        replacement_count += 1;
    }
    if cursor < source.len() {
        output.push_str(&source[cursor..]);
    }

    (output, replacement_count)
}

pub(crate) fn rewrite_lexer_tokens(
    source: &str,
    dialect: StyleDialect,
    mut rewrite: impl FnMut(SyntaxKind, &str) -> Option<String>,
) -> (String, usize) {
    let lexed = lex(source, dialect);
    let mut output = String::with_capacity(source.len());
    let mut cursor = 0;
    let mut mutation_count = 0;

    for token in lexed.tokens() {
        let start = u32::from(token.range.start()) as usize;
        let end = u32::from(token.range.end()) as usize;
        if start > cursor {
            output.push_str(&source[cursor..start]);
        }
        if let Some(replacement) = rewrite(token.kind, &token.text) {
            if replacement != token.text {
                mutation_count += 1;
            }
            output.push_str(&replacement);
        } else {
            output.push_str(&source[start..end]);
        }
        cursor = end;
    }

    if cursor < source.len() {
        output.push_str(&source[cursor..]);
    }

    (output, mutation_count)
}

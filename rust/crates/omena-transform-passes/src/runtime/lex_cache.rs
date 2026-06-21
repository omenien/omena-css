use std::{cell::RefCell, collections::BTreeMap, rc::Rc};

use cstree::text::{TextRange, TextSize};
use omena_parser::{LexedToken, StyleDialect, lex};

use crate::TransformProvenanceMutationSpanV0;

#[derive(Debug, Clone)]
pub(crate) struct CachedLexResultV0 {
    tokens: Rc<Vec<LexedToken>>,
}

impl CachedLexResultV0 {
    pub(crate) fn tokens(&self) -> &[LexedToken] {
        self.tokens.as_slice()
    }
}

#[derive(Default)]
struct TransformLexCacheV0 {
    entries: BTreeMap<(StyleDialect, String), Rc<Vec<LexedToken>>>,
}

thread_local! {
    static ACTIVE_TRANSFORM_LEX_CACHES: RefCell<Vec<TransformLexCacheV0>> =
        const { RefCell::new(Vec::new()) };
}

struct TransformLexCacheScopeGuard;

impl Drop for TransformLexCacheScopeGuard {
    fn drop(&mut self) {
        ACTIVE_TRANSFORM_LEX_CACHES.with(|caches| {
            caches.borrow_mut().pop();
        });
    }
}

pub(crate) fn with_transform_lex_cache<T>(operation: impl FnOnce() -> T) -> T {
    ACTIVE_TRANSFORM_LEX_CACHES.with(|caches| {
        caches.borrow_mut().push(TransformLexCacheV0::default());
    });
    let _guard = TransformLexCacheScopeGuard;
    operation()
}

pub(crate) fn lex_cached(source: &str, dialect: StyleDialect) -> CachedLexResultV0 {
    ACTIVE_TRANSFORM_LEX_CACHES.with(|caches| {
        let mut caches = caches.borrow_mut();
        let Some(cache) = caches.last_mut() else {
            return CachedLexResultV0 {
                tokens: Rc::new(materialize_lex_tokens(source, dialect)),
            };
        };

        let key = (dialect, source.to_string());
        if let Some(cached) = cache.entries.get(&key) {
            return CachedLexResultV0 {
                tokens: Rc::clone(cached),
            };
        }

        let tokens = Rc::new(materialize_lex_tokens(source, dialect));
        cache.entries.insert(key, Rc::clone(&tokens));
        CachedLexResultV0 { tokens }
    })
}

pub(crate) fn update_cached_lex_from_splice(
    input: &str,
    output: &str,
    dialect: StyleDialect,
    mutation_spans: &[TransformProvenanceMutationSpanV0],
) {
    if input == output || mutation_spans.is_empty() {
        return;
    }

    ACTIVE_TRANSFORM_LEX_CACHES.with(|caches| {
        let mut caches = caches.borrow_mut();
        let Some(cache) = caches.last_mut() else {
            return;
        };

        let input_key = (dialect, input.to_string());
        let input_tokens = cache
            .entries
            .entry(input_key)
            .or_insert_with(|| Rc::new(materialize_lex_tokens(input, dialect)))
            .clone();
        let Some(windows) = restart_windows_for_mutation_spans(
            input,
            output,
            input_tokens.as_slice(),
            mutation_spans,
        ) else {
            return;
        };
        let relex_byte_len = windows
            .iter()
            .map(|window| window.output_end.saturating_sub(window.output_start))
            .sum::<usize>();
        if relex_byte_len >= output.len() {
            return;
        }
        let Some(output_tokens) =
            spliced_tokens_for_windows(output, dialect, input_tokens.as_slice(), windows)
        else {
            return;
        };
        cache
            .entries
            .insert((dialect, output.to_string()), Rc::new(output_tokens));
    });
}

fn materialize_lex_tokens(source: &str, dialect: StyleDialect) -> Vec<LexedToken> {
    lex(source, dialect).tokens().to_vec()
}

#[cfg(test)]
fn spliced_tokens_for_output(
    input: &str,
    output: &str,
    dialect: StyleDialect,
    input_tokens: &[LexedToken],
    mutation_spans: &[TransformProvenanceMutationSpanV0],
) -> Option<Vec<LexedToken>> {
    if input == output {
        return Some(input_tokens.to_vec());
    }
    if mutation_spans.is_empty() {
        return None;
    }

    let windows = restart_windows_for_mutation_spans(input, output, input_tokens, mutation_spans)?;
    spliced_tokens_for_windows(output, dialect, input_tokens, windows)
}

fn spliced_tokens_for_windows(
    output: &str,
    dialect: StyleDialect,
    input_tokens: &[LexedToken],
    windows: Vec<SpliceRestartWindowV0>,
) -> Option<Vec<LexedToken>> {
    let mut tokens = Vec::with_capacity(input_tokens.len());
    let mut source_cursor = 0usize;
    let mut current_delta = 0isize;

    for window in windows {
        tokens.extend(
            input_tokens
                .iter()
                .filter(|token| {
                    token_start(token) >= source_cursor && token_end(token) <= window.source_start
                })
                .cloned()
                .map(|token| offset_token(token, current_delta))
                .collect::<Option<Vec<_>>>()?,
        );
        tokens.extend(
            materialize_lex_tokens(&output[window.output_start..window.output_end], dialect)
                .into_iter()
                .map(|token| offset_token(token, window.output_start as isize))
                .collect::<Option<Vec<_>>>()?,
        );
        source_cursor = window.source_end;
        current_delta = window.output_end as isize - window.source_end as isize;
    }
    tokens.extend(
        input_tokens
            .iter()
            .filter(|token| token_start(token) >= source_cursor)
            .cloned()
            .map(|token| offset_token(token, current_delta))
            .collect::<Option<Vec<_>>>()?,
    );
    Some(tokens)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct SpliceRestartWindowV0 {
    source_start: usize,
    source_end: usize,
    output_start: usize,
    output_end: usize,
}

fn restart_windows_for_mutation_spans(
    input: &str,
    output: &str,
    input_tokens: &[LexedToken],
    mutation_spans: &[TransformProvenanceMutationSpanV0],
) -> Option<Vec<SpliceRestartWindowV0>> {
    let mut windows = mutation_spans
        .iter()
        .map(|span| restart_window_for_mutation_span(input, output, input_tokens, span))
        .collect::<Option<Vec<_>>>()?;
    windows.sort_by(|left, right| {
        left.source_start
            .cmp(&right.source_start)
            .then_with(|| left.source_end.cmp(&right.source_end))
    });

    let mut merged = Vec::<SpliceRestartWindowV0>::new();
    for window in windows {
        let Some(last) = merged.last_mut() else {
            merged.push(window);
            continue;
        };
        if window.source_start <= last.source_end || window.output_start <= last.output_end {
            last.source_end = last.source_end.max(window.source_end);
            last.output_end = last.output_end.max(window.output_end);
        } else {
            merged.push(window);
        }
    }
    Some(merged)
}

fn restart_window_for_mutation_span(
    input: &str,
    output: &str,
    input_tokens: &[LexedToken],
    span: &TransformProvenanceMutationSpanV0,
) -> Option<SpliceRestartWindowV0> {
    let source_start = span.source_span_start.min(input.len());
    let source_end = span.source_span_end.min(input.len());
    let generated_start = span.generated_span_start.min(output.len());
    let generated_end = span.generated_span_end.min(output.len());
    if source_start > source_end || generated_start > generated_end {
        return None;
    }

    let (source_window_start, source_window_end) =
        source_restart_window(input, input_tokens, source_start, source_end);
    let left_context_len = source_start.saturating_sub(source_window_start);
    let right_context_len = source_window_end.saturating_sub(source_end);
    let output_window_start = floor_char_boundary(
        output,
        generated_start
            .saturating_sub(left_context_len)
            .min(output.len()),
    );
    let output_window_end = ceil_char_boundary(
        output,
        generated_end
            .saturating_add(right_context_len)
            .min(output.len()),
    );
    if output_window_start > output_window_end {
        return None;
    }

    Some(SpliceRestartWindowV0 {
        source_start: source_window_start,
        source_end: source_window_end,
        output_start: output_window_start,
        output_end: output_window_end,
    })
}

fn source_restart_window(
    input: &str,
    input_tokens: &[LexedToken],
    source_start: usize,
    source_end: usize,
) -> (usize, usize) {
    if input_tokens.is_empty() {
        return (0, input.len());
    }

    let first_touching = input_tokens
        .iter()
        .position(|token| token_end(token) > source_start)
        .unwrap_or(input_tokens.len().saturating_sub(1));
    let last_touching = input_tokens
        .iter()
        .rposition(|token| token_start(token) < source_end)
        .unwrap_or(first_touching);

    let left_index = first_touching.saturating_sub(1);
    let right_index = (last_touching + 1).min(input_tokens.len().saturating_sub(1));

    (
        floor_char_boundary(input, token_start(&input_tokens[left_index])),
        ceil_char_boundary(input, token_end(&input_tokens[right_index])),
    )
}

fn offset_token(token: LexedToken, offset: isize) -> Option<LexedToken> {
    let start = apply_offset(token_start(&token), offset)?;
    let end = apply_offset(token_end(&token), offset)?;
    Some(LexedToken {
        kind: token.kind,
        range: text_range(start, end),
        text: token.text,
    })
}

fn apply_offset(value: usize, offset: isize) -> Option<usize> {
    if offset >= 0 {
        value.checked_add(offset as usize)
    } else {
        value.checked_sub((-offset) as usize)
    }
}

fn token_start(token: &LexedToken) -> usize {
    token.range.start().into()
}

fn token_end(token: &LexedToken) -> usize {
    token.range.end().into()
}

fn text_range(start: usize, end: usize) -> TextRange {
    TextRange::new(TextSize::from(start as u32), TextSize::from(end as u32))
}

fn floor_char_boundary(source: &str, mut index: usize) -> usize {
    index = index.min(source.len());
    while index > 0 && !source.is_char_boundary(index) {
        index -= 1;
    }
    index
}

fn ceil_char_boundary(source: &str, mut index: usize) -> usize {
    index = index.min(source.len());
    while index < source.len() && !source.is_char_boundary(index) {
        index += 1;
    }
    index
}

#[cfg(test)]
mod tests {
    use super::{
        lex_cached, materialize_lex_tokens, spliced_tokens_for_output, with_transform_lex_cache,
    };
    use omena_parser::{StyleDialect, with_omena_parser_lex_instrumentation};

    use crate::runtime::provenance::derive_transform_mutation_spans;

    #[test]
    fn transform_lex_cache_materializes_identical_source_once_per_scope() {
        let source = ".button { color: red; }";
        let (token_kinds, instrumentation) = with_omena_parser_lex_instrumentation(|| {
            with_transform_lex_cache(|| {
                let first = lex_cached(source, StyleDialect::Css);
                let second = lex_cached(source, StyleDialect::Css);

                first
                    .tokens()
                    .iter()
                    .zip(second.tokens())
                    .map(|(left, right)| {
                        assert_eq!(left, right);
                        left.kind
                    })
                    .collect::<Vec<_>>()
            })
        });

        assert!(!token_kinds.is_empty());
        assert_eq!(instrumentation.lex_invocation_count, 1);
    }

    #[test]
    fn transform_lex_cache_is_scoped_to_an_execution() {
        let source = ".button { color: red; }";
        let (_, instrumentation) = with_omena_parser_lex_instrumentation(|| {
            with_transform_lex_cache(|| {
                let _ = lex_cached(source, StyleDialect::Css);
            });
            with_transform_lex_cache(|| {
                let _ = lex_cached(source, StyleDialect::Css);
            });
        });

        assert_eq!(instrumentation.lex_invocation_count, 2);
    }

    #[test]
    fn lex_splice_equivalence_property_covers_generated_edits() {
        for (input, output) in splice_equivalence_cases() {
            assert_splice_equivalent_to_full_relex(&input, &output, StyleDialect::Css);
        }
    }

    fn assert_splice_equivalent_to_full_relex(input: &str, output: &str, dialect: StyleDialect) {
        let input_tokens = materialize_lex_tokens(input, dialect);
        let mutation_spans = derive_transform_mutation_spans(input, output);
        let incremental = spliced_tokens_for_output(
            input,
            output,
            dialect,
            input_tokens.as_slice(),
            mutation_spans.as_slice(),
        );
        let full = materialize_lex_tokens(output, dialect);

        assert!(
            incremental.is_some(),
            "splice equivalence case fell back before producing tokens\ninput: {input}\noutput: {output}",
        );
        if let Some(incremental) = incremental {
            assert_eq!(incremental, full, "input: {input}\noutput: {output}");
        }
    }

    fn splice_equivalence_cases() -> Vec<(String, String)> {
        let mut cases = vec![
            (
                ".a { color: red; margin: 0px; }".to_string(),
                ".a { color: blue; margin: 0px; }".to_string(),
            ),
            (
                ".a { content: \"open\"; color: red; }".to_string(),
                ".a { content: \"opened\"; color: red; }".to_string(),
            ),
            (
                ".한글 { color: red; margin: 0px; }".to_string(),
                ".한글 { color: blue; margin: 0px; }".to_string(),
            ),
            (
                ".a { color: red; }\n.b { color: blue; }".to_string(),
                ".a { color: green; }\n.b { color: navy; }".to_string(),
            ),
            (
                ".a { --x: 1px; }".to_string(),
                ".a { --xy: 1px; }".to_string(),
            ),
            (
                ".a { color: red; }".to_string(),
                ".a { /*c*/ color: red; }".to_string(),
            ),
        ];

        let seed = concat!(
            ".a { color: red; margin: 0px; padding: 1px; }\n",
            ".b { content: \"open\"; --x: 1px; }\n",
            ".한글 { transform: translateX(1px); }\n",
        );
        for (from, to) in [
            ("red", "blue"),
            ("0px", "10px"),
            ("1px", "2px"),
            ("\"open\"", "\"opened\""),
            ("--x", "--xy"),
            ("translateX(1px)", "translateX(2px)"),
        ] {
            push_replacement_case(&mut cases, seed, from, to);
        }

        let mut cumulative = seed.to_string();
        for (from, to) in [
            ("red", "green"),
            ("0px", "4px"),
            ("padding: 1px", "padding: calc(1px + 1px)"),
            ("\"open\"", "\"열림\""),
            ("translateX(1px)", "translateX(3px) rotate(1deg)"),
        ] {
            let Some(next) = replace_once(&cumulative, from, to) else {
                assert!(
                    cumulative.contains(from),
                    "missing generated edit fixture token: {from}",
                );
                continue;
            };
            cases.push((cumulative, next.clone()));
            cumulative = next;
        }

        push_replacement_case(
            &mut cases,
            ".a { color: red; }\n.b { color: blue; }",
            "red; }\n.b { color",
            "green; }\n.b { background-color",
        );
        push_replacement_case(
            &mut cases,
            ".a { color: red; margin: 0px; padding: 1px; }",
            "red; margin: 0px",
            "blue; margin: 10px",
        );

        cases
    }

    fn push_replacement_case(cases: &mut Vec<(String, String)>, input: &str, from: &str, to: &str) {
        let Some(output) = replace_once(input, from, to) else {
            assert!(
                input.contains(from),
                "missing generated edit fixture token: {from}"
            );
            return;
        };
        cases.push((input.to_string(), output));
    }

    fn replace_once(input: &str, from: &str, to: &str) -> Option<String> {
        let start = input.find(from)?;
        let end = start + from.len();
        let mut output = String::with_capacity(input.len() + to.len().saturating_sub(from.len()));
        output.push_str(&input[..start]);
        output.push_str(to);
        output.push_str(&input[end..]);
        Some(output)
    }
}

//! Lightweight instrumentation for parser materialization.
//!
//! The counters in this module support regression gates that verify parser
//! consumers do not accidentally rematerialize token streams.

use std::cell::Cell;
#[cfg(test)]
use std::sync::atomic::{AtomicUsize, Ordering};

use serde::Serialize;

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OmenaParserLexInstrumentationSnapshotV0 {
    pub lex_invocation_count: u64,
    pub lex_token_count: u64,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OmenaParserParseInstrumentationSnapshotV0 {
    pub parse_invocation_count: u64,
    pub parse_token_count: u64,
}

thread_local! {
    static LEX_INSTRUMENTATION: Cell<Option<OmenaParserLexInstrumentationSnapshotV0>> =
        const { Cell::new(None) };
    static PARSE_INSTRUMENTATION: Cell<Option<OmenaParserParseInstrumentationSnapshotV0>> =
        const { Cell::new(None) };
    #[cfg(test)]
    static SYNTAX_ROOT_MATERIALIZATION_COUNT: AtomicUsize =
        const { AtomicUsize::new(0) };
}

pub fn with_omena_parser_lex_instrumentation<T>(
    operation: impl FnOnce() -> T,
) -> (T, OmenaParserLexInstrumentationSnapshotV0) {
    LEX_INSTRUMENTATION.with(|instrumentation| {
        let previous =
            instrumentation.replace(Some(OmenaParserLexInstrumentationSnapshotV0::default()));
        let value = operation();
        let snapshot = instrumentation
            .replace(previous)
            .unwrap_or_else(OmenaParserLexInstrumentationSnapshotV0::default);
        (value, snapshot)
    })
}

pub fn with_omena_parser_parse_instrumentation<T>(
    operation: impl FnOnce() -> T,
) -> (T, OmenaParserParseInstrumentationSnapshotV0) {
    PARSE_INSTRUMENTATION.with(|instrumentation| {
        let previous =
            instrumentation.replace(Some(OmenaParserParseInstrumentationSnapshotV0::default()));
        let value = operation();
        let snapshot = instrumentation
            .replace(previous)
            .unwrap_or_else(OmenaParserParseInstrumentationSnapshotV0::default);
        (value, snapshot)
    })
}

pub(crate) fn record_omena_parser_lex_materialization(token_count: usize) {
    LEX_INSTRUMENTATION.with(|instrumentation| {
        if let Some(mut snapshot) = instrumentation.get() {
            snapshot.lex_invocation_count += 1;
            snapshot.lex_token_count += token_count as u64;
            instrumentation.set(Some(snapshot));
        }
    });
}

pub(crate) fn record_omena_parser_parse_materialization(token_count: usize) {
    PARSE_INSTRUMENTATION.with(|instrumentation| {
        if let Some(mut snapshot) = instrumentation.get() {
            snapshot.parse_invocation_count += 1;
            snapshot.parse_token_count += token_count as u64;
            instrumentation.set(Some(snapshot));
        }
    });
}

#[cfg(test)]
pub(crate) fn record_omena_parser_syntax_root_materialization() {
    SYNTAX_ROOT_MATERIALIZATION_COUNT.with(|counter| counter.fetch_add(1, Ordering::SeqCst));
}

#[cfg(not(test))]
pub(crate) fn record_omena_parser_syntax_root_materialization() {}

#[cfg(test)]
pub(crate) fn reset_omena_parser_syntax_root_materialization_count() {
    SYNTAX_ROOT_MATERIALIZATION_COUNT.with(|counter| counter.store(0, Ordering::SeqCst));
}

#[cfg(test)]
pub(crate) fn omena_parser_syntax_root_materialization_count() -> usize {
    SYNTAX_ROOT_MATERIALIZATION_COUNT.with(|counter| counter.load(Ordering::SeqCst))
}

use std::cell::Cell;

use serde::Serialize;

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OmenaParserLexInstrumentationSnapshotV0 {
    pub lex_invocation_count: u64,
    pub lex_token_count: u64,
}

thread_local! {
    static LEX_INSTRUMENTATION: Cell<Option<OmenaParserLexInstrumentationSnapshotV0>> =
        const { Cell::new(None) };
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

pub(crate) fn record_omena_parser_lex_materialization(token_count: usize) {
    LEX_INSTRUMENTATION.with(|instrumentation| {
        if let Some(mut snapshot) = instrumentation.get() {
            snapshot.lex_invocation_count += 1;
            snapshot.lex_token_count += token_count as u64;
            instrumentation.set(Some(snapshot));
        }
    });
}

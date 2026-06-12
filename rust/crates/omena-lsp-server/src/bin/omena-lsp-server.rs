use std::io::{self, BufRead, Write};
use std::sync::mpsc;
use std::sync::{Arc, Mutex};
use std::thread::{self, JoinHandle};
use std::time::Duration;
use std::{collections::BTreeMap, sync::MutexGuard};

use omena_lsp_server::{
    LspLoopTurnV0, LspQueryDispatchV0, LspShellState, ScheduledLspOutput,
    dispatched_query_internal_error_response, handle_lsp_message_scheduled_outputs_or_dispatch,
    resolve_dispatched_query_response,
};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let stdin = io::stdin();
    run_stdio_server(&mut stdin.lock(), io::stdout())?;
    Ok(())
}

fn run_stdio_server<R: BufRead, W: Write + Send + 'static>(
    reader: &mut R,
    writer: W,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut state = LspShellState::default();
    let writer = Arc::new(Mutex::new(writer));
    let coalescer = Arc::new(Mutex::new(ScheduledOutputCoalescer::default()));
    let mut delayed_outputs: Vec<JoinHandle<Result<(), String>>> = Vec::new();
    // RFC 0009 Pillar A (rfcs#67, slice A-min): one worker thread answers the
    // heaviest read-only request class (hover/definition) from loop-built
    // copy-on-write snapshots, so a heavy resolve no longer stalls every queued
    // message behind it. Worker responses go to the shared writer DIRECTLY —
    // never through the ScheduledOutputCoalescer: responses must never be
    // pruned (a pruned response hangs the client), and routing them through
    // `write_scheduled_lsp_output` at completion time would allocate coalescer
    // revisions in completion order (revision inversion: a slow stale result
    // could clobber a fresher publish). Server-side ordering caveat: a worker
    // response may interleave ahead of loop-queued notifications; the LSP
    // allows responses to be sent in any order relative to other messages —
    // what matters is that the response reflects the document state at
    // dispatch, which the loop-built snapshot guarantees.
    // Bounded so a pathological request flood degrades to the old blocking
    // loop behavior instead of queueing unbounded snapshots.
    let (query_sender, query_receiver) = mpsc::sync_channel::<Box<LspQueryDispatchV0>>(256);
    let query_worker: JoinHandle<Result<(), String>> = {
        let writer = Arc::clone(&writer);
        thread::spawn(move || {
            while let Ok(dispatch) = query_receiver.recv() {
                // A resolver panic must not kill the worker (every queued
                // dispatch would go unanswered and the client would hang):
                // answer the panicked request with -32603 and keep serving.
                let response = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                    resolve_dispatched_query_response(&dispatch)
                }))
                .unwrap_or_else(|_| dispatched_query_internal_error_response(&dispatch));
                let Some(response) = response else {
                    continue;
                };
                let mut writer = writer
                    .lock()
                    .map_err(|_| "stdout lock poisoned".to_string())?;
                write_lsp_response(&mut *writer, &response).map_err(|error| error.to_string())?;
            }
            Ok(())
        })
    };

    while let Some(payload) = read_lsp_payload(reader)? {
        let message: serde_json::Value = serde_json::from_str(&payload)?;
        match handle_lsp_message_scheduled_outputs_or_dispatch(&mut state, message) {
            LspLoopTurnV0::DispatchQuery(dispatch) => {
                query_sender
                    .send(dispatch)
                    .map_err(|_| "query worker exited before shutdown")?;
            }
            LspLoopTurnV0::Outputs(outputs) => {
                for output in outputs {
                    write_scheduled_lsp_output(&writer, &coalescer, output, &mut delayed_outputs)?;
                }
            }
        }
        if state.should_exit {
            break;
        }
    }

    // Drain exactly like delayed_outputs: closing the channel lets the worker
    // finish every dispatched request before exiting, so shutdown/exit never
    // drops an in-flight hover/definition response.
    drop(query_sender);
    query_worker
        .join()
        .map_err(|_| "query worker panicked".to_string())?
        .map_err(|error| format!("query worker failed: {error}"))?;

    for handle in delayed_outputs {
        handle
            .join()
            .map_err(|_| "delayed LSP writer panicked".to_string())?
            .map_err(|error| format!("delayed LSP writer failed: {error}"))?;
    }

    Ok(())
}

#[derive(Debug, Default)]
struct ScheduledOutputCoalescer {
    latest_revision_by_key: BTreeMap<String, u64>,
}

impl ScheduledOutputCoalescer {
    fn schedule(&mut self, key: &str) -> u64 {
        let next_revision = self
            .latest_revision_by_key
            .get(key)
            .copied()
            .unwrap_or(0)
            .saturating_add(1);
        self.latest_revision_by_key
            .insert(key.to_string(), next_revision);
        next_revision
    }

    fn is_current(&self, key: &str, revision: u64) -> bool {
        self.latest_revision_by_key
            .get(key)
            .is_some_and(|latest_revision| *latest_revision == revision)
    }
}

fn read_lsp_payload<R: BufRead>(
    reader: &mut R,
) -> Result<Option<String>, Box<dyn std::error::Error>> {
    let mut content_length: Option<usize> = None;

    loop {
        let mut line = String::new();
        let read = reader.read_line(&mut line)?;
        if read == 0 {
            return Ok(None);
        }
        let trimmed = line.trim_end_matches(['\r', '\n']);
        if trimmed.is_empty() {
            break;
        }
        if let Some(value) = trimmed.strip_prefix("Content-Length:") {
            content_length = Some(value.trim().parse::<usize>()?);
        }
    }

    let Some(length) = content_length else {
        return Err("missing Content-Length header".into());
    };
    let mut buffer = vec![0; length];
    reader.read_exact(&mut buffer)?;
    let payload = String::from_utf8(buffer)?;
    Ok(Some(payload))
}

fn write_lsp_response<W: Write>(
    writer: &mut W,
    response: &serde_json::Value,
) -> Result<(), Box<dyn std::error::Error>> {
    let body = serde_json::to_vec(response)?;
    write!(writer, "Content-Length: {}\r\n\r\n", body.len())?;
    writer.write_all(&body)?;
    writer.flush()?;
    Ok(())
}

fn write_scheduled_lsp_output<W: Write + Send + 'static>(
    writer: &Arc<Mutex<W>>,
    coalescer: &Arc<Mutex<ScheduledOutputCoalescer>>,
    output: ScheduledLspOutput,
    delayed_outputs: &mut Vec<JoinHandle<Result<(), String>>>,
) -> Result<(), Box<dyn std::error::Error>> {
    let scheduled_revision = output
        .coalesce_key
        .as_ref()
        .map(|key| lock_coalescer(coalescer).schedule(key));

    if let Some(delay_millis) = output.delay_millis {
        let writer = Arc::clone(writer);
        let coalescer = Arc::clone(coalescer);
        delayed_outputs.push(thread::spawn(move || {
            thread::sleep(Duration::from_millis(delay_millis));
            if let (Some(key), Some(revision)) = (output.coalesce_key.as_ref(), scheduled_revision)
                && !lock_coalescer(&coalescer).is_current(key, revision)
            {
                return Ok(());
            }
            let mut writer = writer
                .lock()
                .map_err(|_| "stdout lock poisoned".to_string())?;
            write_lsp_response(&mut *writer, &output.value).map_err(|error| error.to_string())
        }));
        return Ok(());
    }

    let mut writer = writer.lock().map_err(|_| "stdout lock poisoned")?;
    write_lsp_response(&mut *writer, &output.value)
}

fn lock_coalescer(
    coalescer: &Arc<Mutex<ScheduledOutputCoalescer>>,
) -> MutexGuard<'_, ScheduledOutputCoalescer> {
    coalescer.lock().unwrap_or_else(|error| error.into_inner())
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::{Value, json};

    const APP_STYLE_URI: &str = "file:///workspace-a/src/App.module.scss";
    const THEME_STYLE_URI: &str = "file:///workspace-a/src/_theme.scss";
    /// Distinct, non-substring color keywords: one per didChange generation so a
    /// hover response pins exactly which corpus generation it was computed from.
    const GENERATION_COLORS: [&str; 7] = [
        "blue",
        "tomato",
        "orchid",
        "salmon",
        "sienna",
        "peachpuff",
        "honeydew",
    ];

    #[derive(Clone, Default)]
    struct SharedBufferWriter(Arc<Mutex<Vec<u8>>>);

    impl Write for SharedBufferWriter {
        fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
            self.0
                .lock()
                .map_err(|_| io::Error::other("shared writer poisoned"))?
                .extend_from_slice(buf);
            Ok(buf.len())
        }

        fn flush(&mut self) -> io::Result<()> {
            Ok(())
        }
    }

    fn frame(message: &Value) -> Result<Vec<u8>, String> {
        let body = serde_json::to_vec(message).map_err(|error| error.to_string())?;
        let mut framed = format!("Content-Length: {}\r\n\r\n", body.len()).into_bytes();
        framed.extend_from_slice(&body);
        Ok(framed)
    }

    fn parse_lsp_frames(bytes: &[u8]) -> Result<Vec<Value>, String> {
        const HEADER_SEPARATOR: &[u8] = b"\r\n\r\n";
        let mut messages = Vec::new();
        let mut cursor = 0usize;
        while cursor < bytes.len() {
            let header_end = bytes[cursor..]
                .windows(HEADER_SEPARATOR.len())
                .position(|window| window == HEADER_SEPARATOR)
                .ok_or_else(|| "missing LSP header separator".to_string())?
                + cursor;
            let headers = std::str::from_utf8(&bytes[cursor..header_end])
                .map_err(|error| error.to_string())?;
            let length = headers
                .lines()
                .find_map(|line| line.strip_prefix("Content-Length:"))
                .ok_or_else(|| "missing Content-Length header".to_string())?
                .trim()
                .parse::<usize>()
                .map_err(|error| error.to_string())?;
            let body_start = header_end + HEADER_SEPARATOR.len();
            let body = bytes
                .get(body_start..body_start + length)
                .ok_or_else(|| "truncated LSP frame body".to_string())?;
            messages.push(serde_json::from_slice(body).map_err(|error| error.to_string())?);
            cursor = body_start + length;
        }
        Ok(messages)
    }

    fn theme_text_for_generation(generation: usize) -> String {
        // Each generation pads the selector down one line, so a definition
        // request at (line = generation) only resolves against the corpus that
        // includes that generation's didChange — a stale snapshot returns null.
        format!(
            "{}.btn {{ color: {}; }}",
            "\n".repeat(generation),
            GENERATION_COLORS[generation]
        )
    }

    fn did_change_theme(generation: usize) -> Value {
        json!({
            "jsonrpc": "2.0",
            "method": "textDocument/didChange",
            "params": {
                "textDocument": {
                    "uri": THEME_STYLE_URI,
                    "version": generation + 1,
                },
                "contentChanges": [
                    {
                        "text": theme_text_for_generation(generation),
                    },
                ],
            },
        })
    }

    fn hover_app_btn(id: u64) -> Value {
        json!({
            "jsonrpc": "2.0",
            "id": id,
            "method": "textDocument/hover",
            "params": {
                "textDocument": {
                    "uri": APP_STYLE_URI,
                },
                "position": {
                    "line": 1,
                    "character": 2,
                },
            },
        })
    }

    fn definition_theme_btn(id: u64, line: usize) -> Value {
        json!({
            "jsonrpc": "2.0",
            "id": id,
            "method": "textDocument/definition",
            "params": {
                "textDocument": {
                    "uri": THEME_STYLE_URI,
                },
                "position": {
                    "line": line,
                    "character": 2,
                },
            },
        })
    }

    /// RFC 0009 Pillar A (rfcs#67) dispatcher stress contract: a burst of
    /// didChange + hover/definition interleavings through the REAL
    /// `run_stdio_server` plumbing (loop + query worker + shared writer) must
    /// (a) answer every request exactly once, (b) answer from a state no older
    /// than the last didChange acknowledged before dispatch — asserted exactly:
    /// the loop is FIFO up to dispatch, so each response must reflect precisely
    /// the generation preceding it in the script — and (c) lose nothing on
    /// clean shutdown, including requests dispatched immediately before
    /// shutdown/exit.
    #[test]
    fn dispatcher_answers_did_change_hover_definition_burst_exactly_once_and_fresh()
    -> Result<(), String> {
        let mut script: Vec<Value> = vec![
            json!({
                "jsonrpc": "2.0",
                "id": 1,
                "method": "initialize",
                "params": {
                    "workspaceFolders": [
                        {
                            "uri": "file:///workspace-a",
                            "name": "workspace-a",
                        },
                    ],
                },
            }),
            json!({
                "jsonrpc": "2.0",
                "method": "textDocument/didOpen",
                "params": {
                    "textDocument": {
                        "uri": APP_STYLE_URI,
                        "languageId": "scss",
                        "version": 1,
                        "text": "@use \"./theme\";\n.btn { color: red; }\n.btn { color: green; }",
                    },
                },
            }),
            json!({
                "jsonrpc": "2.0",
                "method": "textDocument/didOpen",
                "params": {
                    "textDocument": {
                        "uri": THEME_STYLE_URI,
                        "languageId": "scss",
                        "version": 1,
                        "text": theme_text_for_generation(0),
                    },
                },
            }),
        ];
        let last_generation = GENERATION_COLORS.len() - 1;
        for generation in 1..=last_generation {
            script.push(did_change_theme(generation));
            script.push(hover_app_btn(100 + generation as u64));
            script.push(definition_theme_btn(200 + generation as u64, generation));
        }
        // Final volley right before shutdown: drain must deliver these even
        // though exit follows immediately in the input stream.
        script.push(hover_app_btn(300));
        script.push(definition_theme_btn(301, last_generation));
        script.push(json!({
            "jsonrpc": "2.0",
            "id": 999,
            "method": "shutdown",
        }));
        script.push(json!({
            "jsonrpc": "2.0",
            "method": "exit",
        }));

        let mut input: Vec<u8> = Vec::new();
        for message in &script {
            input.extend_from_slice(frame(message)?.as_slice());
        }
        let sink = SharedBufferWriter::default();
        let mut reader: &[u8] = input.as_slice();
        run_stdio_server(&mut reader, sink.clone()).map_err(|error| error.to_string())?;

        let output = sink
            .0
            .lock()
            .map_err(|_| "shared writer poisoned".to_string())?
            .clone();
        let messages = parse_lsp_frames(output.as_slice())?;

        // (a) + (c): every request answered exactly once, nothing lost at exit.
        let expected_ids: Vec<u64> = std::iter::once(1)
            .chain((1..=last_generation).flat_map(|g| [100 + g as u64, 200 + g as u64]))
            .chain([300, 301, 999])
            .collect();
        for id in &expected_ids {
            let responses = messages
                .iter()
                .filter(|message| message.get("id") == Some(&json!(id)))
                .collect::<Vec<_>>();
            assert_eq!(
                responses.len(),
                1,
                "request {id} must get exactly one response, got {responses:?}"
            );
            assert!(
                responses[0].get("error").is_none(),
                "request {id} must not error: {:?}",
                responses[0]
            );
        }
        let response_count = messages
            .iter()
            .filter(|message| message.get("id").is_some())
            .count();
        assert_eq!(
            response_count,
            expected_ids.len(),
            "no unexpected responses"
        );

        // (b): each dispatched response reflects exactly the generation that
        // the loop had acknowledged when it dispatched the request.
        for generation in 1..=last_generation {
            let hover_markdown = messages
                .iter()
                .find(|message| message.get("id") == Some(&json!(100 + generation as u64)))
                .and_then(|message| message.pointer("/result/contents/value"))
                .and_then(Value::as_str)
                .ok_or_else(|| format!("hover {generation} must render markdown"))?;
            assert!(
                hover_markdown.contains(GENERATION_COLORS[generation]),
                "hover after didChange {generation} must see that generation: {hover_markdown}"
            );
            assert!(
                !hover_markdown.contains(GENERATION_COLORS[generation - 1]),
                "hover after didChange {generation} must not see the previous generation: {hover_markdown}"
            );
            let definition_line = messages
                .iter()
                .find(|message| message.get("id") == Some(&json!(200 + generation as u64)))
                .and_then(|message| message.pointer("/result/0/range/start/line"))
                .and_then(Value::as_u64);
            assert_eq!(
                definition_line,
                Some(generation as u64),
                "definition after didChange {generation} must resolve in that generation's corpus"
            );
        }
        Ok(())
    }

    #[test]
    fn delayed_output_with_stale_coalesce_revision_is_skipped() -> Result<(), String> {
        let writer = Arc::new(Mutex::new(Vec::<u8>::new()));
        let coalescer = Arc::new(Mutex::new(ScheduledOutputCoalescer::default()));
        let mut delayed_outputs = Vec::new();
        let key = "textDocument/publishDiagnostics:file:///workspace/App.module.scss".to_string();

        write_scheduled_lsp_output(
            &writer,
            &coalescer,
            ScheduledLspOutput::delayed_coalesced(
                json!({
                    "jsonrpc": "2.0",
                    "method": "oldOptimizingDiagnostics",
                }),
                10,
                key.clone(),
            ),
            &mut delayed_outputs,
        )
        .map_err(|error| error.to_string())?;

        write_scheduled_lsp_output(
            &writer,
            &coalescer,
            ScheduledLspOutput::immediate_coalesced(
                json!({
                    "jsonrpc": "2.0",
                    "method": "newBaselineDiagnostics",
                }),
                key,
            ),
            &mut delayed_outputs,
        )
        .map_err(|error| error.to_string())?;

        for handle in delayed_outputs {
            handle
                .join()
                .map_err(|_| "delayed writer panicked".to_string())??;
        }

        let body = String::from_utf8(
            writer
                .lock()
                .map_err(|_| "writer lock poisoned".to_string())?
                .clone(),
        )
        .map_err(|error| error.to_string())?;

        assert!(body.contains("\"method\":\"newBaselineDiagnostics\""));
        assert!(!body.contains("\"method\":\"oldOptimizingDiagnostics\""));

        Ok(())
    }
}

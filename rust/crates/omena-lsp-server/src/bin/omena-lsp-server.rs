use std::io::{self, BufRead, Write};
use std::sync::{Arc, Mutex};
use std::thread::{self, JoinHandle};
use std::time::Duration;
use std::{collections::BTreeMap, sync::MutexGuard};

use omena_lsp_server::{LspShellState, ScheduledLspOutput, handle_lsp_message_scheduled_outputs};

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

    while let Some(payload) = read_lsp_payload(reader)? {
        let message: serde_json::Value = serde_json::from_str(&payload)?;
        for output in handle_lsp_message_scheduled_outputs(&mut state, message) {
            write_scheduled_lsp_output(&writer, &coalescer, output, &mut delayed_outputs)?;
        }
        if state.should_exit {
            break;
        }
    }

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

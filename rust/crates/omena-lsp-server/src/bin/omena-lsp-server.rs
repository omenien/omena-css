use std::io::{self, BufRead, Write};
use std::sync::{Arc, Mutex};
use std::thread::{self, JoinHandle};
use std::time::Duration;

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
    let mut delayed_outputs: Vec<JoinHandle<Result<(), String>>> = Vec::new();

    while let Some(payload) = read_lsp_payload(reader)? {
        let message: serde_json::Value = serde_json::from_str(&payload)?;
        for output in handle_lsp_message_scheduled_outputs(&mut state, message) {
            write_scheduled_lsp_output(&writer, output, &mut delayed_outputs)?;
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
    output: ScheduledLspOutput,
    delayed_outputs: &mut Vec<JoinHandle<Result<(), String>>>,
) -> Result<(), Box<dyn std::error::Error>> {
    if let Some(delay_millis) = output.delay_millis {
        let writer = Arc::clone(writer);
        delayed_outputs.push(thread::spawn(move || {
            thread::sleep(Duration::from_millis(delay_millis));
            let mut writer = writer.lock().map_err(|_| "stdout lock poisoned".to_string())?;
            write_lsp_response(&mut *writer, &output.value).map_err(|error| error.to_string())
        }));
        return Ok(());
    }

    let mut writer = writer.lock().map_err(|_| "stdout lock poisoned")?;
    write_lsp_response(&mut *writer, &output.value)
}

use std::io::{self, BufRead, Write};
use std::time::Duration;

use omena_lsp_server::{LspShellState, handle_lsp_message_scheduled_outputs};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    run_stdio_server(&mut io::stdin().lock(), &mut io::stdout())?;
    Ok(())
}

fn run_stdio_server<R: BufRead, W: Write>(
    reader: &mut R,
    writer: &mut W,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut state = LspShellState::default();

    while let Some(payload) = read_lsp_payload(reader)? {
        let message: serde_json::Value = serde_json::from_str(&payload)?;
        for output in handle_lsp_message_scheduled_outputs(&mut state, message) {
            if let Some(delay_millis) = output.delay_millis {
                std::thread::sleep(Duration::from_millis(delay_millis));
            }
            write_lsp_response(writer, &output.value)?;
        }
        if state.should_exit {
            break;
        }
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

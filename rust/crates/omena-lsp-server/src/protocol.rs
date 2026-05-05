use crate::LspTextDocumentState;
use omena_query::{ParserByteSpanV0, ParserPositionV0, ParserRangeV0, StyleLanguage};
use serde_json::Value;
use std::path::{Component, Path, PathBuf};

pub(crate) fn path_to_file_uri(path: &Path) -> String {
    format!("file://{}", path.to_string_lossy())
}

pub(crate) fn normalize_path(path: PathBuf) -> PathBuf {
    let mut normalized = PathBuf::new();
    for component in path.components() {
        match component {
            Component::CurDir => {}
            Component::ParentDir => {
                normalized.pop();
            }
            Component::Normal(_) | Component::RootDir | Component::Prefix(_) => {
                normalized.push(component.as_os_str());
            }
        }
    }
    normalized
}

pub(crate) fn workspace_folder_compatible(
    workspace_folder_uri: Option<&str>,
    document: &LspTextDocumentState,
) -> bool {
    match (
        workspace_folder_uri,
        document.workspace_folder_uri.as_deref(),
    ) {
        (Some(left), Some(right)) => workspace_folder_uri_equivalent(left, right),
        _ => true,
    }
}

pub(crate) fn workspace_folder_uri_equivalent(left: &str, right: &str) -> bool {
    if left == right {
        return true;
    }
    match (file_uri_to_path(left), file_uri_to_path(right)) {
        (Some(left_path), Some(right_path)) => {
            normalize_path(left_path) == normalize_path(right_path)
        }
        _ => false,
    }
}

pub(crate) fn is_style_document_uri(uri: &str) -> bool {
    StyleLanguage::from_module_path(uri).is_some()
}

pub(crate) fn file_uri_to_path(uri: &str) -> Option<PathBuf> {
    let raw_path = uri.strip_prefix("file://")?;
    Some(PathBuf::from(percent_decode_uri_path(raw_path)?))
}

fn percent_decode_uri_path(raw_path: &str) -> Option<String> {
    let bytes = raw_path.as_bytes();
    let mut decoded = Vec::with_capacity(bytes.len());
    let mut index = 0usize;
    while index < bytes.len() {
        if bytes[index] == b'%' {
            let high = bytes.get(index + 1).and_then(|byte| hex_value(*byte))?;
            let low = bytes.get(index + 2).and_then(|byte| hex_value(*byte))?;
            decoded.push((high << 4) | low);
            index += 3;
        } else {
            decoded.push(bytes[index]);
            index += 1;
        }
    }
    String::from_utf8(decoded).ok()
}

fn hex_value(byte: u8) -> Option<u8> {
    match byte {
        b'0'..=b'9' => Some(byte - b'0'),
        b'a'..=b'f' => Some(byte - b'a' + 10),
        b'A'..=b'F' => Some(byte - b'A' + 10),
        _ => None,
    }
}

pub(crate) fn location_sort_key(location: &Value) -> (String, u64, u64) {
    let uri = location
        .get("uri")
        .and_then(Value::as_str)
        .unwrap_or_default()
        .to_string();
    let line = location
        .pointer("/range/start/line")
        .and_then(Value::as_u64)
        .unwrap_or_default();
    let character = location
        .pointer("/range/start/character")
        .and_then(Value::as_u64)
        .unwrap_or_default();
    (uri, line, character)
}

pub(crate) fn location_identity_key(location: &Value) -> (String, u64, u64, u64, u64) {
    let uri = location
        .get("uri")
        .and_then(Value::as_str)
        .unwrap_or_default()
        .to_string();
    let start_line = location
        .pointer("/range/start/line")
        .and_then(Value::as_u64)
        .unwrap_or_default();
    let start_character = location
        .pointer("/range/start/character")
        .and_then(Value::as_u64)
        .unwrap_or_default();
    let end_line = location
        .pointer("/range/end/line")
        .and_then(Value::as_u64)
        .unwrap_or_default();
    let end_character = location
        .pointer("/range/end/character")
        .and_then(Value::as_u64)
        .unwrap_or_default();
    (uri, start_line, start_character, end_line, end_character)
}

pub(crate) fn lsp_range_start_sort_key(value: &Value) -> (u64, u64) {
    let line = value
        .pointer("/range/start/line")
        .and_then(Value::as_u64)
        .unwrap_or_default();
    let character = value
        .pointer("/range/start/character")
        .and_then(Value::as_u64)
        .unwrap_or_default();
    (line, character)
}

pub(crate) fn include_declaration_from_params(params: Option<&Value>) -> bool {
    params
        .and_then(|value| value.get("context"))
        .and_then(|value| value.get("includeDeclaration"))
        .and_then(Value::as_bool)
        .unwrap_or(false)
}

pub(crate) fn file_label_from_uri(uri: &str) -> &str {
    uri.rsplit('/')
        .next()
        .filter(|label| !label.is_empty())
        .unwrap_or(uri)
}

pub(crate) fn document_uri_from_params(params: Option<&Value>) -> String {
    params
        .and_then(|value| value.get("textDocument"))
        .and_then(|value| value.get("uri"))
        .and_then(Value::as_str)
        .unwrap_or_default()
        .to_string()
}

pub(crate) fn lsp_position_from_params(params: Option<&Value>) -> Option<ParserPositionV0> {
    let position = params.and_then(|value| value.get("position"))?;
    lsp_position_from_value(position)
}

pub(crate) fn lsp_range_from_value(value: &Value) -> Option<ParserRangeV0> {
    Some(ParserRangeV0 {
        start: lsp_position_from_value(value.get("start")?)?,
        end: lsp_position_from_value(value.get("end")?)?,
    })
}

pub(crate) fn lsp_position_from_value(position: &Value) -> Option<ParserPositionV0> {
    Some(ParserPositionV0 {
        line: position.get("line").and_then(Value::as_u64)? as usize,
        character: position.get("character").and_then(Value::as_u64)? as usize,
    })
}

pub(crate) fn is_css_identifier_continue(ch: char) -> bool {
    ch.is_ascii_alphanumeric() || matches!(ch, '-' | '_')
}

pub(crate) fn parser_range_for_byte_span(source: &str, span: ParserByteSpanV0) -> ParserRangeV0 {
    ParserRangeV0 {
        start: parser_position_for_byte_offset(source, span.start),
        end: parser_position_for_byte_offset(source, span.end),
    }
}

pub(crate) fn parser_position_for_byte_offset(source: &str, offset: usize) -> ParserPositionV0 {
    let clamped_offset = offset.min(source.len());
    let mut line = 0usize;
    let mut character = 0usize;

    for (byte_index, ch) in source.char_indices() {
        if byte_index >= clamped_offset {
            break;
        }
        if ch == '\n' {
            line += 1;
            character = 0;
        } else {
            character += ch.len_utf16();
        }
    }

    ParserPositionV0 { line, character }
}

pub(crate) fn byte_offset_for_parser_position(
    source: &str,
    position: ParserPositionV0,
) -> Option<usize> {
    let mut line = 0usize;
    let mut character = 0usize;

    for (byte_index, ch) in source.char_indices() {
        if line == position.line && character == position.character {
            return Some(byte_index);
        }
        if ch == '\n' {
            if line == position.line {
                return None;
            }
            line += 1;
            character = 0;
        } else {
            character += ch.len_utf16();
            if line == position.line && character > position.character {
                return None;
            }
        }
    }

    if line == position.line && character == position.character {
        Some(source.len())
    } else {
        None
    }
}

pub(crate) fn parser_range_contains_position(
    range: &ParserRangeV0,
    position: ParserPositionV0,
) -> bool {
    parser_position_is_after_or_equal(position, range.start)
        && parser_position_is_before(position, range.end)
}

pub(crate) fn parser_position_is_after_or_equal(
    position: ParserPositionV0,
    start: ParserPositionV0,
) -> bool {
    position.line > start.line
        || (position.line == start.line && position.character >= start.character)
}

pub(crate) fn parser_position_is_before(position: ParserPositionV0, end: ParserPositionV0) -> bool {
    position.line < end.line || (position.line == end.line && position.character < end.character)
}

pub(crate) fn style_language_label(language: StyleLanguage) -> &'static str {
    match language {
        StyleLanguage::Css => "css",
        StyleLanguage::Scss => "scss",
        StyleLanguage::Less => "less",
    }
}

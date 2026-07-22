use crate::LspTextDocumentState;
use omena_query::{ParserByteSpanV0, ParserPositionV0, ParserRangeV0, StyleLanguage};
use serde_json::Value;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::ffi::OsString;
use std::fs;
use std::path::{Component, Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

static CANONICALIZE_PATH_CACHE_VERSION: AtomicU64 = AtomicU64::new(0);

thread_local! {
    static CANONICALIZE_PATH_CACHE: RefCell<CanonicalizePathCache> = const {
        RefCell::new(CanonicalizePathCache {
            version: 0,
            paths: BTreeMap::new(),
        })
    };

    #[cfg(test)]
    static CANONICALIZE_SYSCALL_COUNT: std::cell::Cell<usize> = const { std::cell::Cell::new(0) };
}

struct CanonicalizePathCache {
    version: u64,
    paths: BTreeMap<PathBuf, Option<PathBuf>>,
}

pub(crate) fn invalidate_file_uri_identity_cache() {
    let next_version = CANONICALIZE_PATH_CACHE_VERSION
        .fetch_add(1, Ordering::AcqRel)
        .saturating_add(1);
    CANONICALIZE_PATH_CACHE.with(|cache| {
        let mut cache = cache.borrow_mut();
        cache.version = next_version;
        cache.paths.clear();
    });
}

#[cfg(test)]
pub(crate) fn reset_file_uri_identity_cache_for_test() {
    invalidate_file_uri_identity_cache();
    reset_file_uri_identity_canonicalize_syscall_count_for_test();
}

#[cfg(test)]
pub(crate) fn reset_file_uri_identity_canonicalize_syscall_count_for_test() {
    CANONICALIZE_SYSCALL_COUNT.with(|count| count.set(0));
}

#[cfg(test)]
pub(crate) fn file_uri_identity_canonicalize_syscall_count_for_test() -> usize {
    CANONICALIZE_SYSCALL_COUNT.with(std::cell::Cell::get)
}

pub(crate) fn path_to_file_uri(path: &Path) -> String {
    let path = normalize_path(path.to_path_buf());
    format!(
        "file://{}",
        percent_encode_uri_path(path.to_string_lossy().as_ref())
    )
}

pub(crate) fn canonical_file_uri(uri: &str) -> Option<String> {
    file_uri_to_path(uri).map(|path| path_to_file_uri(path.as_path()))
}

pub(crate) fn normalize_path(path: PathBuf) -> PathBuf {
    if let Some(canonical) = canonicalize_existing_path_or_parent(path.as_path()) {
        return normalize_path_lexical(canonical);
    }
    normalize_path_lexical(path)
}

fn canonicalize_existing_path_or_parent(path: &Path) -> Option<PathBuf> {
    if let Some(cached) = canonicalize_path_cache_get(path) {
        return cached;
    }

    let canonical = canonicalize_existing_path_or_parent_uncached(path);
    canonicalize_path_cache_insert(path.to_path_buf(), canonical.clone());
    if let Some(canonical_path) = canonical.as_ref() {
        canonicalize_path_cache_insert(canonical_path.clone(), Some(canonical_path.clone()));
    }
    canonical
}

fn canonicalize_existing_path_or_parent_uncached(path: &Path) -> Option<PathBuf> {
    if let Ok(canonical) = canonicalize_path_for_identity(path) {
        return Some(canonical);
    }

    let mut current = path.to_path_buf();
    let mut suffix = Vec::<OsString>::new();
    while let Some(parent) = current.parent() {
        if let Some(file_name) = current.file_name() {
            suffix.push(file_name.to_os_string());
        }
        if let Ok(mut canonical_parent) = canonicalize_path_for_identity(parent) {
            for segment in suffix.iter().rev() {
                canonical_parent.push(segment);
            }
            return Some(canonical_parent);
        }
        current = parent.to_path_buf();
    }
    None
}

fn canonicalize_path_cache_get(path: &Path) -> Option<Option<PathBuf>> {
    CANONICALIZE_PATH_CACHE.with(|cache| {
        let mut cache = cache.borrow_mut();
        sync_canonicalize_path_cache_version(&mut cache);
        cache.paths.get(path).cloned()
    })
}

fn canonicalize_path_cache_insert(path: PathBuf, canonical: Option<PathBuf>) {
    CANONICALIZE_PATH_CACHE.with(|cache| {
        let mut cache = cache.borrow_mut();
        sync_canonicalize_path_cache_version(&mut cache);
        cache.paths.insert(path, canonical);
    });
}

fn sync_canonicalize_path_cache_version(cache: &mut CanonicalizePathCache) {
    let current = CANONICALIZE_PATH_CACHE_VERSION.load(Ordering::Acquire);
    if cache.version != current {
        cache.version = current;
        cache.paths.clear();
    }
}

fn canonicalize_path_for_identity(path: &Path) -> std::io::Result<PathBuf> {
    #[cfg(test)]
    CANONICALIZE_SYSCALL_COUNT.with(|count| count.set(count.get().saturating_add(1)));
    fs::canonicalize(path)
}

fn normalize_path_lexical(path: PathBuf) -> PathBuf {
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
    file_uri_equivalent(left, right)
}

pub(crate) fn file_uri_equivalent(left: &str, right: &str) -> bool {
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

fn percent_encode_uri_path(path: &str) -> String {
    let mut encoded = String::with_capacity(path.len());
    for byte in path.as_bytes() {
        match *byte {
            b'A'..=b'Z'
            | b'a'..=b'z'
            | b'0'..=b'9'
            | b'-'
            | b'.'
            | b'_'
            | b'~'
            | b'/'
            | b'@'
            | b':'
            | b'!'
            | b'$'
            | b'&'
            | b'\''
            | b'*'
            | b'+'
            | b','
            | b';'
            | b'=' => encoded.push(*byte as char),
            _ => encoded.push_str(format!("%{byte:02X}").as_str()),
        }
    }
    encoded
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

pub(crate) fn file_label_from_uri(uri: &str) -> String {
    let label = uri
        .rsplit('/')
        .next()
        .filter(|label| !label.is_empty())
        .unwrap_or(uri);
    // Decode AFTER splitting so an encoded %2F cannot change segment boundaries;
    // fall back to the raw label on invalid percent sequences (rfcs#122).
    percent_decode_uri_path(label).unwrap_or_else(|| label.to_string())
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

#[cfg(test)]
mod file_label_tests {
    use super::file_label_from_uri;

    #[test]
    fn decodes_percent_encoded_non_ascii_labels() {
        assert_eq!(
            file_label_from_uri("file:///ws/src/%EC%83%98%ED%94%8C%EB%B0%B0%EB%84%88.module.scss"),
            "샘플배너.module.scss"
        );
    }

    #[test]
    fn decodes_after_splitting_so_encoded_slashes_stay_in_the_label() {
        assert_eq!(file_label_from_uri("file:///ws/a%2Fb.css"), "a/b.css");
    }

    #[test]
    fn falls_back_to_the_raw_label_on_invalid_percent_sequences() {
        assert_eq!(file_label_from_uri("file:///ws/bad%GG.css"), "bad%GG.css");
    }

    #[test]
    fn passes_plain_ascii_labels_through_unchanged() {
        assert_eq!(
            file_label_from_uri("file:///ws/App.module.css"),
            "App.module.css"
        );
        assert_eq!(file_label_from_uri("no-slash"), "no-slash");
    }
}

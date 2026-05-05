use serde::Serialize;

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SourceImportDeclarationSummaryV0 {
    pub schema_version: &'static str,
    pub product: &'static str,
    pub import_count: usize,
    pub imports: Vec<SourceImportDeclarationV0>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SourceImportDeclarationV0 {
    pub binding: String,
    pub specifier: String,
}

pub fn summarize_omena_bridge_source_import_declarations(
    source: &str,
) -> SourceImportDeclarationSummaryV0 {
    let mut imports = Vec::new();
    let mut cursor = 0usize;
    while let Some(identifier) = next_code_identifier(source, cursor) {
        cursor = identifier.end;
        if identifier.text != "import" {
            continue;
        }
        if let Some(import) = parse_source_import_declaration(source, identifier.end) {
            imports.push(import);
        }
    }
    imports.sort_by(|left, right| {
        left.binding
            .cmp(&right.binding)
            .then_with(|| left.specifier.cmp(&right.specifier))
    });
    imports.dedup();

    SourceImportDeclarationSummaryV0 {
        schema_version: "0",
        product: "omena-bridge.source-import-declarations",
        import_count: imports.len(),
        imports,
    }
}

fn parse_source_import_declaration(
    source: &str,
    after_import: usize,
) -> Option<SourceImportDeclarationV0> {
    let mut cursor = skip_js_trivia(source, after_import);
    match source.as_bytes().get(cursor).copied()? {
        b'(' | b'\'' | b'"' => return None,
        _ => {}
    }

    let clause_start = cursor;
    let mut clause_end = None;
    let mut specifier = None;
    while cursor < source.len() {
        cursor = skip_js_trivia(source, cursor);
        let Some(byte) = source.as_bytes().get(cursor).copied() else {
            break;
        };
        if matches!(byte, b'\'' | b'"') {
            if clause_end.is_some()
                && let Some((literal_start, literal_end, _)) =
                    js_string_literal_span(source, cursor, source.len())
            {
                specifier = source.get(literal_start..literal_end).map(str::to_string);
            }
            break;
        }
        if byte == b';' {
            break;
        }
        if byte.is_ascii_alphabetic() || matches!(byte, b'_' | b'$') {
            let (identifier, identifier_end) = read_js_identifier(source, cursor)?;
            if identifier == "from" && clause_end.is_none() {
                clause_end = Some(cursor);
            }
            cursor = identifier_end;
            continue;
        }
        cursor = advance_js_scan_cursor(source, cursor, source.len());
    }

    let clause = source.get(clause_start..clause_end?)?;
    Some(SourceImportDeclarationV0 {
        binding: import_binding_from_clause(clause)?.to_string(),
        specifier: specifier?,
    })
}

fn import_binding_from_clause(clause: &str) -> Option<&str> {
    let clause = clause.trim();
    if clause.is_empty() || clause.starts_with('{') {
        return None;
    }
    if let Some(namespace_clause) = clause.strip_prefix('*') {
        let namespace_clause = namespace_clause.trim_start();
        let namespace_clause = namespace_clause.strip_prefix("as")?.trim_start();
        let (binding, _) = read_js_identifier(namespace_clause, 0)?;
        return Some(binding);
    }

    let default_clause = clause.split(',').next()?.trim();
    let (binding, _) = read_js_identifier(default_clause, 0)?;
    if binding == "type" {
        return None;
    }
    Some(binding)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct CodeIdentifier<'a> {
    text: &'a str,
    end: usize,
}

fn next_code_identifier(source: &str, mut cursor: usize) -> Option<CodeIdentifier<'_>> {
    while cursor < source.len() {
        cursor = skip_js_trivia(source, cursor);
        let byte = source.as_bytes().get(cursor).copied()?;
        if matches!(byte, b'\'' | b'"' | b'`') {
            cursor = skip_js_string_literal(source, cursor, source.len()).unwrap_or(source.len());
            continue;
        }
        if byte.is_ascii_alphabetic() || matches!(byte, b'_' | b'$') {
            let (text, end) = read_js_identifier(source, cursor)?;
            return Some(CodeIdentifier { text, end });
        }
        cursor = advance_js_scan_cursor(source, cursor, source.len());
    }
    None
}

fn skip_js_trivia(source: &str, cursor: usize) -> usize {
    skip_js_trivia_until(source, cursor, source.len())
}

fn skip_js_trivia_until(source: &str, mut cursor: usize, limit: usize) -> usize {
    loop {
        cursor = skip_ascii_whitespace_until(source, cursor, limit);
        if source.as_bytes().get(cursor) == Some(&b'/') {
            match source.as_bytes().get(cursor + 1).copied() {
                Some(b'/') => {
                    cursor = skip_js_line_comment(source, cursor + 2, limit);
                    continue;
                }
                Some(b'*') => {
                    cursor = skip_js_block_comment(source, cursor + 2, limit);
                    continue;
                }
                _ => {}
            }
        }
        return cursor;
    }
}

fn skip_ascii_whitespace_until(source: &str, mut offset: usize, limit: usize) -> usize {
    while offset < limit
        && source
            .as_bytes()
            .get(offset)
            .is_some_and(u8::is_ascii_whitespace)
    {
        offset += 1;
    }
    offset
}

fn skip_js_line_comment(source: &str, mut cursor: usize, limit: usize) -> usize {
    let limit = char_boundary_floor(source, limit);
    while cursor < limit {
        if source.as_bytes().get(cursor) == Some(&b'\n') {
            return advance_js_scan_cursor(source, cursor, limit);
        }
        cursor = advance_js_scan_cursor(source, cursor, limit);
    }
    limit
}

fn skip_js_block_comment(source: &str, mut cursor: usize, limit: usize) -> usize {
    let limit = char_boundary_floor(source, limit);
    while cursor + 1 < limit {
        if source.as_bytes().get(cursor) == Some(&b'*')
            && source.as_bytes().get(cursor + 1) == Some(&b'/')
        {
            return cursor + 2;
        }
        cursor = advance_js_scan_cursor(source, cursor, limit);
    }
    limit
}

fn js_string_literal_span(
    source: &str,
    quote_offset: usize,
    limit: usize,
) -> Option<(usize, usize, usize)> {
    let quote = source.as_bytes().get(quote_offset).copied()?;
    if !matches!(quote, b'\'' | b'"' | b'`') {
        return None;
    }
    let literal_start = quote_offset + 1;
    let next_offset = skip_js_string_literal(source, quote_offset, limit)?;
    Some((literal_start, next_offset - 1, next_offset))
}

fn skip_js_string_literal(source: &str, quote_offset: usize, limit: usize) -> Option<usize> {
    let quote = source.as_bytes().get(quote_offset).copied()?;
    let limit = char_boundary_floor(source, limit);
    let mut cursor = quote_offset + 1;
    while cursor < limit {
        let byte = source.as_bytes().get(cursor).copied()?;
        if byte == b'\\' {
            cursor = advance_js_escaped_char(source, cursor, limit);
            continue;
        }
        if byte == quote {
            return Some(cursor + 1);
        }
        cursor = advance_js_scan_cursor(source, cursor, limit);
    }
    None
}

fn read_js_identifier(source: &str, start: usize) -> Option<(&str, usize)> {
    let start = char_boundary_ceil(source, start);
    let first = source.get(start..)?.chars().next()?;
    if !is_js_identifier_start(first) {
        return None;
    }
    let mut end = start + first.len_utf8();
    let scan_start = end;
    for (relative_index, ch) in source.get(scan_start..)?.char_indices() {
        if !is_js_identifier_continue(ch) {
            break;
        }
        end = scan_start + relative_index + ch.len_utf8();
    }
    Some((&source[start..end], end))
}

fn is_js_identifier_start(ch: char) -> bool {
    ch == '_' || ch == '$' || ch.is_ascii_alphabetic()
}

fn is_js_identifier_continue(ch: char) -> bool {
    is_js_identifier_start(ch) || ch.is_ascii_digit()
}

fn char_boundary_floor(source: &str, offset: usize) -> usize {
    let mut offset = offset.min(source.len());
    while offset > 0 && !source.is_char_boundary(offset) {
        offset -= 1;
    }
    offset
}

fn char_boundary_ceil(source: &str, offset: usize) -> usize {
    let mut offset = offset.min(source.len());
    while offset < source.len() && !source.is_char_boundary(offset) {
        offset += 1;
    }
    offset
}

fn advance_js_scan_cursor(source: &str, cursor: usize, limit: usize) -> usize {
    let limit = char_boundary_floor(source, limit);
    let cursor = char_boundary_ceil(source, cursor);
    if cursor >= limit {
        return limit;
    }
    source
        .get(cursor..limit)
        .and_then(|rest| rest.chars().next())
        .map(|ch| cursor + ch.len_utf8())
        .unwrap_or(limit)
}

fn advance_js_escaped_char(source: &str, backslash_offset: usize, limit: usize) -> usize {
    let after_backslash = advance_js_scan_cursor(source, backslash_offset, limit);
    advance_js_scan_cursor(source, after_backslash, limit)
}

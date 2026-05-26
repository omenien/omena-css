use super::*;

const OMENA_IGNORE_FILE: &str = "omena-ignore-file";
const OMENA_IGNORE_NEXT_LINE: &str = "omena-ignore-next-line";
const OMENA_EXPECT_ERROR: &str = "omena-expect-error";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum OmenaDiagnosticDirectiveKindV0 {
    IgnoreFile,
    IgnoreNextLine,
    ExpectError,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct OmenaDiagnosticDirectiveV0 {
    kind: OmenaDiagnosticDirectiveKindV0,
    target_line: Option<usize>,
    range: ParserRangeV0,
    codes: BTreeSet<String>,
}

#[derive(Debug, Clone, Copy)]
struct OmenaDiagnosticDirectiveLineV0<'a> {
    source: &'a str,
    line: &'a str,
    line_start_offset: usize,
}

#[derive(Debug, Clone, Copy)]
struct OmenaDiagnosticDirectiveSpecV0 {
    name: &'static str,
    kind: OmenaDiagnosticDirectiveKindV0,
    target_line: Option<usize>,
}

pub(super) fn apply_omena_query_style_diagnostic_suppressions(
    source: &str,
    summary: &mut OmenaQueryStyleDiagnosticsForFileV0,
) {
    let directives = parse_omena_query_diagnostic_directives(source);
    push_omena_query_ready_surface(&mut summary.ready_surfaces, "diagnosticSuppressionSyntax");
    if directives.is_empty() {
        summary.diagnostic_count = summary.diagnostics.len();
        return;
    }

    let mut consumed_expect_directives = BTreeSet::new();
    for (directive_index, directive) in directives.iter().enumerate() {
        if directive.kind != OmenaDiagnosticDirectiveKindV0::ExpectError {
            continue;
        }
        if summary
            .diagnostics
            .iter()
            .any(|diagnostic| diagnostic_matches_line_directive(diagnostic, directive))
        {
            consumed_expect_directives.insert(directive_index);
        }
    }

    summary
        .diagnostics
        .retain(|diagnostic| !diagnostic_is_suppressed(diagnostic, directives.as_slice()));

    for (directive_index, directive) in directives.iter().enumerate() {
        if directive.kind != OmenaDiagnosticDirectiveKindV0::ExpectError
            || consumed_expect_directives.contains(&directive_index)
            || file_directives_suppress_code(directives.as_slice(), "unusedOmenaExpectError")
        {
            continue;
        }

        summary.diagnostics.push(OmenaQueryStyleDiagnosticV0 {
            code: "unusedOmenaExpectError",
            severity: "warning",
            provenance: vec!["omena-query.diagnostic-suppression-syntax"],
            range: directive.range,
            message: "Unused omena-expect-error directive.".to_string(),
            tags: Vec::new(),
            create_custom_property: None,
        });
    }

    summary.diagnostic_count = summary.diagnostics.len();
}

fn diagnostic_is_suppressed(
    diagnostic: &OmenaQueryStyleDiagnosticV0,
    directives: &[OmenaDiagnosticDirectiveV0],
) -> bool {
    directives.iter().any(|directive| match directive.kind {
        OmenaDiagnosticDirectiveKindV0::IgnoreFile => directive_matches_code(directive, diagnostic),
        OmenaDiagnosticDirectiveKindV0::IgnoreNextLine
        | OmenaDiagnosticDirectiveKindV0::ExpectError => {
            diagnostic_matches_line_directive(diagnostic, directive)
        }
    })
}

fn diagnostic_matches_line_directive(
    diagnostic: &OmenaQueryStyleDiagnosticV0,
    directive: &OmenaDiagnosticDirectiveV0,
) -> bool {
    directive.target_line == Some(diagnostic.range.start.line)
        && directive_matches_code(directive, diagnostic)
}

fn directive_matches_code(
    directive: &OmenaDiagnosticDirectiveV0,
    diagnostic: &OmenaQueryStyleDiagnosticV0,
) -> bool {
    directive.codes.is_empty() || directive.codes.contains(diagnostic.code)
}

fn file_directives_suppress_code(
    directives: &[OmenaDiagnosticDirectiveV0],
    diagnostic_code: &str,
) -> bool {
    directives.iter().any(|directive| {
        directive.kind == OmenaDiagnosticDirectiveKindV0::IgnoreFile
            && (directive.codes.is_empty() || directive.codes.contains(diagnostic_code))
    })
}

fn parse_omena_query_diagnostic_directives(source: &str) -> Vec<OmenaDiagnosticDirectiveV0> {
    let mut directives = Vec::new();
    let mut line_start_offset = 0usize;

    for (line_index, line) in source.split_inclusive('\n').enumerate() {
        let line_without_newline = line.trim_end_matches(['\r', '\n']);
        let line_context = OmenaDiagnosticDirectiveLineV0 {
            source,
            line: line_without_newline,
            line_start_offset,
        };
        collect_line_directive(
            line_context,
            OmenaDiagnosticDirectiveSpecV0 {
                name: OMENA_IGNORE_FILE,
                kind: OmenaDiagnosticDirectiveKindV0::IgnoreFile,
                target_line: None,
            },
            &mut directives,
        );
        collect_line_directive(
            line_context,
            OmenaDiagnosticDirectiveSpecV0 {
                name: OMENA_IGNORE_NEXT_LINE,
                kind: OmenaDiagnosticDirectiveKindV0::IgnoreNextLine,
                target_line: Some(line_index + 1),
            },
            &mut directives,
        );
        collect_line_directive(
            line_context,
            OmenaDiagnosticDirectiveSpecV0 {
                name: OMENA_EXPECT_ERROR,
                kind: OmenaDiagnosticDirectiveKindV0::ExpectError,
                target_line: Some(line_index + 1),
            },
            &mut directives,
        );
        line_start_offset += line.len();
    }

    directives
}

fn collect_line_directive(
    line_context: OmenaDiagnosticDirectiveLineV0<'_>,
    spec: OmenaDiagnosticDirectiveSpecV0,
    directives: &mut Vec<OmenaDiagnosticDirectiveV0>,
) {
    let Some(directive_offset) = line_context.line.find(spec.name) else {
        return;
    };
    let range = parser_range_for_byte_span(
        line_context.source,
        ParserByteSpanV0 {
            start: line_context.line_start_offset + directive_offset,
            end: line_context.line_start_offset + directive_offset + spec.name.len(),
        },
    );
    let code_tail = &line_context.line[directive_offset + spec.name.len()..];
    directives.push(OmenaDiagnosticDirectiveV0 {
        kind: spec.kind,
        target_line: spec.target_line,
        range,
        codes: parse_omena_query_diagnostic_directive_codes(code_tail),
    });
}

fn parse_omena_query_diagnostic_directive_codes(tail: &str) -> BTreeSet<String> {
    let mut code_text = tail
        .trim_start_matches(|ch: char| ch.is_whitespace() || matches!(ch, ':' | '=' | '[' | '('));
    if let Some((before_comment_end, _)) = code_text.split_once("*/") {
        code_text = before_comment_end;
    }
    if let Some((before_reason, _)) = code_text.split_once("--") {
        code_text = before_reason;
    }

    code_text
        .split(|ch: char| ch.is_whitespace() || matches!(ch, ',' | ']' | ')' | ';'))
        .filter_map(|token| {
            let code = token.trim_matches(|ch: char| matches!(ch, '"' | '\'' | '`'));
            if code.is_empty()
                || !code
                    .chars()
                    .all(|ch| ch.is_ascii_alphanumeric() || matches!(ch, '_' | '-'))
            {
                return None;
            }
            Some(code.to_string())
        })
        .collect()
}

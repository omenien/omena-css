use super::*;

const OMENA_IGNORE_FILE: &str = "omena-ignore-file";
const OMENA_IGNORE_NEXT_LINE: &str = "omena-ignore-next-line";
const OMENA_IGNORE_BLOCK: &str = "omena-ignore";
const OMENA_EXPECT_ERROR: &str = "omena-expect-error";
const OMENA_STRICT: &str = "@omena-strict";

/// File-scoped strictness sigil parsed from `// @omena-strict: <level>` (RFC 0004 #28 / #35).
///
/// The level is a monotone dial over the external-boundary lattice (#34). `Standard` is the
/// default the moment the sigil is absent, so a file *without* the directive is byte-for-byte
/// identical to today's behaviour. Higher levels escalate boundary diagnostics; `Relaxed`
/// suppresses them entirely.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum OmenaStrictnessLevelV0 {
    /// Suppress every external-boundary diagnostic; `TopAny` everywhere, never emit
    /// `missingExternalSif`/`staleExternalSif`/`partialExternalSif`.
    Relaxed,
    /// The default surface: `Missing`/`Stale` warn, `Partial` informs, and genuinely-unknown
    /// `TopAny` external symbols stay suppressed.
    Standard,
    /// Escalate `Missing`/`Stale`/`Partial` boundaries to `error`; still suppress genuinely
    /// unknown `TopAny` symbols.
    Strict,
    /// `TopOpaque` everywhere: unknown external symbols become errors too, and every
    /// non-`Resolved` boundary is escalated to `error`.
    Closed,
}

impl OmenaStrictnessLevelV0 {
    /// The level in force when no sigil is present — must preserve today's behaviour exactly.
    pub(super) const DEFAULT: Self = Self::Standard;

    fn from_token(token: &str) -> Option<Self> {
        match token {
            "relaxed" => Some(Self::Relaxed),
            "standard" => Some(Self::Standard),
            "strict" => Some(Self::Strict),
            "closed" => Some(Self::Closed),
            _ => None,
        }
    }

    /// Whether the `TopAny` external-symbol suppression should run at all. Only `Closed`
    /// flips the boundary to `TopOpaque`, exposing unknown external symbols as errors.
    pub(super) const fn suppresses_top_any_external_symbols(self) -> bool {
        !matches!(self, Self::Closed)
    }

    /// Whether external-boundary diagnostics (`missingExternalSif` etc.) are emitted at all.
    /// `Relaxed` drops them; every other level keeps them.
    pub(super) const fn emits_external_boundary_diagnostics(self) -> bool {
        !matches!(self, Self::Relaxed)
    }

    /// Multiply the level into a boundary diagnostic's default severity. `Strict`/`Closed`
    /// escalate non-`information` severities to `error`; lower levels pass the default through.
    pub(super) const fn boundary_severity(self, default_severity: &'static str) -> &'static str {
        match self {
            Self::Strict | Self::Closed => "error",
            Self::Standard | Self::Relaxed => default_severity,
        }
    }
}

/// Parse the file-scoped `@omena-strict: <level>` sigil. The directive is file-scoped like
/// `omena-ignore-file`, so the last well-formed occurrence wins; an absent or malformed sigil
/// yields the [`OmenaStrictnessLevelV0::DEFAULT`] level (today's behaviour).
pub(super) fn parse_omena_query_style_strictness_level(source: &str) -> OmenaStrictnessLevelV0 {
    let mut level = OmenaStrictnessLevelV0::DEFAULT;
    for line in source.lines() {
        let Some(directive_offset) = line.find(OMENA_STRICT) else {
            continue;
        };
        let tail = &line[directive_offset + OMENA_STRICT.len()..];
        let codes = parse_omena_query_diagnostic_directive_codes(tail);
        if let Some(parsed) = codes
            .iter()
            .find_map(|token| OmenaStrictnessLevelV0::from_token(token.as_str()))
        {
            level = parsed;
        }
    }
    level
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum OmenaDiagnosticDirectiveKindV0 {
    IgnoreFile,
    IgnoreBlock,
    IgnoreNextLine,
    ExpectError,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct OmenaDiagnosticDirectiveV0 {
    kind: OmenaDiagnosticDirectiveKindV0,
    target_line: Option<usize>,
    target_line_end: Option<usize>,
    range: ParserRangeV0,
    codes: BTreeSet<String>,
    reason: Option<String>,
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
    let original_diagnostic_count = summary.diagnostics.len();
    let directives = parse_omena_query_diagnostic_directives(source);
    push_omena_query_ready_surface(&mut summary.ready_surfaces, "diagnosticSuppressionSyntax");
    if directives.is_empty() {
        summary.diagnostic_count = summary.diagnostics.len();
        summary.suppression_summary = Some(OmenaQueryDiagnosticSuppressionSummaryV0 {
            original_diagnostic_count,
            emitted_diagnostic_count: summary.diagnostics.len(),
            suppressed_diagnostic_count: 0,
            unused_expect_error_count: 0,
            suppression_reasons: Vec::new(),
        });
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
    let suppressed_diagnostic_count =
        original_diagnostic_count.saturating_sub(summary.diagnostics.len());

    let mut unused_expect_error_count = 0usize;
    for (directive_index, directive) in directives.iter().enumerate() {
        if directive.kind != OmenaDiagnosticDirectiveKindV0::ExpectError
            || consumed_expect_directives.contains(&directive_index)
            || file_directives_suppress_code(directives.as_slice(), "unusedOmenaExpectError")
        {
            continue;
        }

        unused_expect_error_count += 1;
        summary.diagnostics.push(OmenaQueryStyleDiagnosticV0 {
            code: "unusedOmenaExpectError",
            severity: "warning",
            provenance: vec!["omena-query.diagnostic-suppression-syntax"],
            range: directive.range,
            message: "Unused omena-expect-error directive.".to_string(),
            tags: Vec::new(),
            create_custom_property: None,
            cascade_narrowing: None,
            cascade_confidence: None,
            polynomial_provenance: None,
            cross_file_scc: None,
        });
    }

    summary.diagnostic_count = summary.diagnostics.len();
    let suppression_reasons = summarize_omena_query_diagnostic_suppression_reasons(&directives);
    summary.suppression_summary = Some(OmenaQueryDiagnosticSuppressionSummaryV0 {
        original_diagnostic_count,
        emitted_diagnostic_count: summary.diagnostics.len(),
        suppressed_diagnostic_count,
        unused_expect_error_count,
        suppression_reasons,
    });
}

fn diagnostic_is_suppressed(
    diagnostic: &OmenaQueryStyleDiagnosticV0,
    directives: &[OmenaDiagnosticDirectiveV0],
) -> bool {
    directives.iter().any(|directive| match directive.kind {
        OmenaDiagnosticDirectiveKindV0::IgnoreFile => directive_matches_code(directive, diagnostic),
        OmenaDiagnosticDirectiveKindV0::IgnoreBlock => {
            diagnostic_matches_block_directive(diagnostic, directive)
        }
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

fn diagnostic_matches_block_directive(
    diagnostic: &OmenaQueryStyleDiagnosticV0,
    directive: &OmenaDiagnosticDirectiveV0,
) -> bool {
    let Some(start_line) = directive.target_line else {
        return false;
    };
    let Some(end_line) = directive.target_line_end else {
        return false;
    };
    (start_line..=end_line).contains(&diagnostic.range.start.line)
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
        collect_block_directive(line_context, &mut directives);
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
        target_line_end: None,
        range,
        codes: parse_omena_query_diagnostic_directive_codes(code_tail),
        reason: parse_omena_query_diagnostic_directive_reason(code_tail),
    });
}

fn collect_block_directive(
    line_context: OmenaDiagnosticDirectiveLineV0<'_>,
    directives: &mut Vec<OmenaDiagnosticDirectiveV0>,
) {
    let Some(directive_offset) = find_plain_omena_ignore_directive(line_context.line) else {
        return;
    };
    let directive_end =
        line_context.line_start_offset + directive_offset + OMENA_IGNORE_BLOCK.len();
    let Some((target_line, target_line_end)) =
        find_omena_ignore_block_line_range(line_context.source, directive_end)
    else {
        return;
    };
    let range = parser_range_for_byte_span(
        line_context.source,
        ParserByteSpanV0 {
            start: line_context.line_start_offset + directive_offset,
            end: directive_end,
        },
    );
    let code_tail = &line_context.line[directive_offset + OMENA_IGNORE_BLOCK.len()..];
    directives.push(OmenaDiagnosticDirectiveV0 {
        kind: OmenaDiagnosticDirectiveKindV0::IgnoreBlock,
        target_line: Some(target_line),
        target_line_end: Some(target_line_end),
        range,
        codes: parse_omena_query_diagnostic_directive_codes(code_tail),
        reason: parse_omena_query_diagnostic_directive_reason(code_tail),
    });
}

fn find_plain_omena_ignore_directive(line: &str) -> Option<usize> {
    line.match_indices(OMENA_IGNORE_BLOCK)
        .find_map(|(offset, _)| {
            let tail = &line[offset + OMENA_IGNORE_BLOCK.len()..];
            if tail.starts_with("-file") || tail.starts_with("-next-line") {
                return None;
            }
            Some(offset)
        })
}

fn find_omena_ignore_block_line_range(source: &str, after_offset: usize) -> Option<(usize, usize)> {
    let open_index = source[after_offset..]
        .find('{')
        .map(|relative| after_offset + relative)?;
    let open_range = parser_range_for_byte_span(
        source,
        ParserByteSpanV0 {
            start: open_index,
            end: open_index + 1,
        },
    );
    let mut depth = 0usize;
    let mut index = open_index;
    let mut quote: Option<char> = None;
    while index < source.len() {
        let Some(character) = source[index..].chars().next() else {
            break;
        };
        if let Some(quote_character) = quote {
            index += character.len_utf8();
            if character == '\\' {
                if let Some(escaped) = source[index..].chars().next() {
                    index += escaped.len_utf8();
                }
            } else if character == quote_character {
                quote = None;
            }
            continue;
        }
        match character {
            '"' | '\'' => quote = Some(character),
            '{' => depth += 1,
            '}' => {
                depth = depth.saturating_sub(1);
                if depth == 0 {
                    let close_range = parser_range_for_byte_span(
                        source,
                        ParserByteSpanV0 {
                            start: index,
                            end: index + character.len_utf8(),
                        },
                    );
                    return Some((open_range.start.line, close_range.end.line));
                }
            }
            _ => {}
        }
        index += character.len_utf8();
    }
    None
}

fn parse_omena_query_diagnostic_directive_codes(tail: &str) -> BTreeSet<String> {
    let mut code_text = tail
        .trim_start_matches(|ch: char| ch.is_whitespace() || matches!(ch, ':' | '=' | '[' | '('));
    if let Some((before_comment_end, _)) = code_text.split_once("*/") {
        code_text = before_comment_end;
    }
    if let Some((before_reason, _)) = code_text.split_once("[reason") {
        code_text = before_reason;
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

fn parse_omena_query_diagnostic_directive_reason(tail: &str) -> Option<String> {
    let (_, after_marker) = tail.split_once("[reason")?;
    let (_, after_colon) = after_marker.split_once(':')?;
    let before_end = after_colon
        .split_once(']')
        .map_or(after_colon, |(before, _)| before);
    let reason = before_end
        .trim()
        .trim_matches(|character| matches!(character, '"' | '\'' | '`'))
        .trim();
    (!reason.is_empty()).then(|| reason.to_string())
}

fn summarize_omena_query_diagnostic_suppression_reasons(
    directives: &[OmenaDiagnosticDirectiveV0],
) -> Vec<OmenaQueryDiagnosticSuppressionReasonV0> {
    directives
        .iter()
        .filter_map(|directive| {
            let reason = directive.reason.clone()?;
            Some(OmenaQueryDiagnosticSuppressionReasonV0 {
                directive_kind: directive_kind_label(directive.kind),
                codes: directive.codes.iter().cloned().collect(),
                reason,
                range: directive.range,
            })
        })
        .collect()
}

fn directive_kind_label(kind: OmenaDiagnosticDirectiveKindV0) -> &'static str {
    match kind {
        OmenaDiagnosticDirectiveKindV0::IgnoreFile => "ignoreFile",
        OmenaDiagnosticDirectiveKindV0::IgnoreBlock => "ignoreBlock",
        OmenaDiagnosticDirectiveKindV0::IgnoreNextLine => "ignoreNextLine",
        OmenaDiagnosticDirectiveKindV0::ExpectError => "expectError",
    }
}

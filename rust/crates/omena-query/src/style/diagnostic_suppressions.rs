use super::*;

const OMENA_IGNORE_FILE: &str = "omena-ignore-file";
const OMENA_IGNORE_NEXT_LINE: &str = "omena-ignore-next-line";
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
    summary.suppression_summary = Some(OmenaQueryDiagnosticSuppressionSummaryV0 {
        original_diagnostic_count,
        emitted_diagnostic_count: summary.diagnostics.len(),
        suppressed_diagnostic_count,
        unused_expect_error_count,
    });
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

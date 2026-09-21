use crate::{
    OmenaError, OmenaErrorClassV0, OmenaErrorContextV0, OmenaErrorRecoverabilityV0,
    OmenaErrorSeverityV0, OmenaQuerySourceSelectorReferenceFactV0 as SourceSelectorReferenceFact,
    OmenaQuerySourceSelectorReferenceMatchKindV0 as SourceSelectorReferenceMatchKind,
    OmenaQuerySourceSelectorReferenceSurfaceV0 as SourceSelectorReferenceSurface,
    OmenaQuerySourceTypeFactExpressionShapeV0 as SourceTypeFactExpressionShape,
    OmenaQuerySourceTypeFactLexicalAttemptV0 as SourceTypeFactLexicalAttempt,
    OmenaQuerySourceTypeFactLexicalDispositionV0 as SourceTypeFactLexicalDisposition,
    OmenaQuerySourceTypeFactTargetV0 as SourceTypeFactTarget, ParserByteSpanV0,
};
use omena_syntax::ident::{is_ascii_word_continue, is_css_name_continue, is_safe_css_identifier};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::collections::{BTreeMap, BTreeSet};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct OmenaWorkspaceProviderResolvedTypeV0 {
    pub kind: String,
    pub values: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct OmenaWorkspaceProviderResultV0 {
    pub file_path: String,
    pub expression_id: String,
    pub byte_span: ParserByteSpanV0,
    pub resolved_type: OmenaWorkspaceProviderResolvedTypeV0,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct OmenaWorkspaceProviderSpanResultV0 {
    pub file_path: String,
    pub expression_id: String,
    pub byte_span: ParserByteSpanV0,
    pub outcome: String,
    pub reason: String,
    pub span_exact: bool,
    pub non_nullish_member_count: usize,
    pub resolved_member_count: usize,
    pub resolved_type: OmenaWorkspaceProviderResolvedTypeV0,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct OmenaWorkspaceProviderUnavailableV0 {
    pub expression_id: String,
    pub byte_span: ParserByteSpanV0,
    pub reason: String,
}

/// Necessary provider inputs/results, not a serialized SourceSyntaxIndex.
/// Receivers rebuild target ids/spans from text, admit each result and recompute
/// the semantic projection. This is portable evidence, not provider authentication.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct OmenaWorkspaceSourceProviderInputsV0 {
    pub provider_id: String,
    pub source_digest: String,
    pub entries: Vec<OmenaWorkspaceProviderResultV0>,
    pub span_entries: Vec<OmenaWorkspaceProviderSpanResultV0>,
    pub unavailable: Vec<OmenaWorkspaceProviderUnavailableV0>,
}

pub fn source_provider_snapshot_digest_v0(source: &str) -> String {
    omena_sif::compute_omena_sif_leaf_hash_v1(source.as_bytes())
        .as_str()
        .to_string()
}

pub fn replay_omena_workspace_source_provider_v0(
    source_path: &str,
    source: &str,
    index: &mut crate::OmenaQuerySourceSyntaxIndexV0,
    attempts: &[SourceTypeFactLexicalAttempt],
    styles: &[crate::OmenaQueryStyleSourceInputV0],
    provider: &OmenaWorkspaceSourceProviderInputsV0,
) -> Result<(), OmenaError> {
    if provider.provider_id != "tsgo"
        || provider.source_digest != source_provider_snapshot_digest_v0(source)
    {
        return Err(provider_error(
            "provider or source digest does not match the admitted document",
        ));
    }
    let legacy = index.type_fact_targets.clone();
    let spans = source_provider_span_targets(source, attempts);
    let mut targets = legacy.clone();
    targets.extend(spans.clone());
    let mut seen = BTreeSet::new();
    for entry in &provider.entries {
        validate_provider_target(
            source_path,
            &entry.file_path,
            &entry.expression_id,
            entry.byte_span,
            &legacy,
        )?;
        if !seen.insert(entry.expression_id.as_str()) {
            return Err(provider_error("duplicate provider result"));
        }
        validate_resolved_type(&entry.resolved_type)?;
    }
    for entry in &provider.span_entries {
        validate_provider_target(
            source_path,
            &entry.file_path,
            &entry.expression_id,
            entry.byte_span,
            &spans,
        )?;
        if !seen.insert(entry.expression_id.as_str()) {
            return Err(provider_error("duplicate provider result"));
        }
        if !matches!(entry.outcome.as_str(), "resolved" | "refused") {
            return Err(provider_error("unknown span provider outcome"));
        }
        validate_resolved_type(&entry.resolved_type)?;
    }
    let mut entries = provider.entries.clone();
    for entry in &provider.span_entries {
        if let Some(target) = spans
            .iter()
            .find(|target| target.expression_id == entry.expression_id)
            && span_type_fact_entry_admissibility(entry, &target.prefix, &target.suffix).is_ok()
        {
            entries.push(OmenaWorkspaceProviderResultV0 {
                file_path: entry.file_path.clone(),
                expression_id: entry.expression_id.clone(),
                byte_span: entry.byte_span,
                resolved_type: entry.resolved_type.clone(),
            });
        }
    }
    let projections = project_provider_targets(source_path, source, &targets, &entries, styles)?;
    let complete = complete_tsgo_projection_expression_ids(&targets, &entries, &projections);
    for (target, selector_name) in projections {
        let byte_span = type_fact_template_spans(source, &target)
            .map(|spans| spans.selector_span)
            .unwrap_or(target.byte_span);
        index.selector_references.push(SourceSelectorReferenceFact {
            byte_span,
            selector_name: Some(selector_name),
            match_kind: SourceSelectorReferenceMatchKind::Exact,
            target_style_uri: target.target_style_uri.clone(),
            surface: SourceSelectorReferenceSurface::OmenaTsgoTypeFactProjection,
        });
    }
    for target in targets
        .iter()
        .filter(|target| complete.contains(&target.expression_id))
    {
        if let Some(spans) = type_fact_template_spans(source, target) {
            take_type_fact_prefix_references(
                &mut index.selector_references,
                spans.prefix_span,
                &target.prefix,
                target.target_style_uri.as_deref(),
            );
        }
    }
    crate::canonicalize_omena_query_source_selector_references(&mut index.selector_references);
    let mut unavailable_ids = BTreeSet::new();
    for fact in &provider.unavailable {
        let target = targets
            .iter()
            .find(|target| {
                target.expression_id == fact.expression_id && target.byte_span == fact.byte_span
            })
            .ok_or_else(|| {
                provider_error("provider unavailable target does not match parsed source")
            })?;
        if !unavailable_ids.insert(&fact.expression_id) || complete.contains(&fact.expression_id) {
            return Err(provider_error("inconsistent provider unavailable target"));
        }
        index.type_fact_provider_unavailable.push(
            crate::OmenaQuerySourceTypeFactProviderUnavailableFactV0 {
                expression_id: fact.expression_id.clone(),
                byte_span: fact.byte_span,
                target_style_uri: target.target_style_uri.clone(),
                provider_id: "tsgo",
                reason: admitted_unavailable_reason(&fact.reason)?,
            },
        );
    }
    Ok(())
}

fn validate_provider_target(
    path: &str,
    file_path: &str,
    expression_id: &str,
    span: ParserByteSpanV0,
    targets: &[SourceTypeFactTarget],
) -> Result<(), OmenaError> {
    if snapshot_source_native_path(path).as_deref() != Some(file_path)
        || !targets
            .iter()
            .any(|target| target.expression_id == expression_id && target.byte_span == span)
    {
        return Err(provider_error(
            "provider file, expression or exact span does not match parsed source",
        ));
    }
    Ok(())
}

fn validate_resolved_type(value: &OmenaWorkspaceProviderResolvedTypeV0) -> Result<(), OmenaError> {
    if !matches!(value.kind.as_str(), "union" | "unresolvable") {
        return Err(provider_error("unknown provider result kind"));
    }
    Ok(())
}

fn admitted_unavailable_reason(reason: &str) -> Result<&'static str, OmenaError> {
    match reason {
        "projectMiss" => Ok("projectMiss"),
        "noTransport" => Ok("noTransport"),
        "processUnavailable" => Ok("processUnavailable"),
        "requestFailed" => Ok("requestFailed"),
        "missingResult" => Ok("missingResult"),
        "unresolvable" => Ok("unresolvable"),
        _ => Err(provider_error("unknown provider unavailable reason")),
    }
}

/// URI spelling conversion for provider request matching only. Native paths are
/// never percent-decoded; this does not issue or canonicalize module identity.
fn snapshot_source_native_path(path: &str) -> Option<String> {
    let Some(uri_path) = path.strip_prefix("file://") else {
        return Some(path.to_string());
    };
    let bytes = uri_path.as_bytes();
    let mut decoded = Vec::with_capacity(bytes.len());
    let mut index = 0;
    while index < bytes.len() {
        if bytes[index] == b'%' {
            let high = char::from(*bytes.get(index + 1)?).to_digit(16)?;
            let low = char::from(*bytes.get(index + 2)?).to_digit(16)?;
            decoded.push((high * 16 + low) as u8);
            index += 3;
        } else {
            decoded.push(bytes[index]);
            index += 1;
        }
    }
    String::from_utf8(decoded).ok()
}

fn source_provider_span_targets(
    source: &str,
    attempts: &[SourceTypeFactLexicalAttempt],
) -> Vec<SourceTypeFactTarget> {
    attempts
        .iter()
        .filter(|attempt| {
            attempt.lexical_disposition == SourceTypeFactLexicalDisposition::Unresolved
                && span_type_fact_shape_is_supported(attempt.shape_class)
        })
        .filter_map(|attempt| {
            let (prefix, suffix) = span_type_fact_template_affixes(source, attempt.byte_span)?;
            Some(SourceTypeFactTarget {
                byte_span: attempt.byte_span,
                expression_id: attempt.expression_id.clone(),
                target_style_uri: attempt.target_style_uri.clone(),
                prefix,
                suffix,
            })
        })
        .collect()
}

fn project_provider_targets(
    source_path: &str,
    source: &str,
    targets: &[SourceTypeFactTarget],
    entries: &[OmenaWorkspaceProviderResultV0],
    styles: &[crate::OmenaQueryStyleSourceInputV0],
) -> Result<Vec<(SourceTypeFactTarget, String)>, OmenaError> {
    let line_index = omena_syntax::OmenaLineIndexV0::new(source);
    let range = |span: ParserByteSpanV0| {
        let (sl, sc) = line_index.position_for_byte_offset(source, span.start);
        let (el, ec) = line_index.position_for_byte_offset(source, span.end);
        json!({"start": {"line":sl,"character":sc}, "end":{"line":el,"character":ec}})
    };
    let class_expressions = targets.iter().filter_map(|target| Some(json!({
        "id":target.expression_id, "kind":"symbolRef", "scssModulePath":target.target_style_uri.as_ref()?,
        "range":range(target.byte_span), "className":null, "rootBindingDeclId":null, "accessPath":null,
    }))).collect::<Vec<_>>();
    let style_inputs = styles.iter().map(|style| {
        // Preserve the existing LSP candidate admission ceiling.
        let candidates = if style.style_source.len() > 64 * 1024 { Vec::new() } else {
            crate::summarize_omena_query_style_hover_candidates(&style.style_path, &style.style_source).map(|summary| summary.candidates).unwrap_or_default()
        };
        let selectors = candidates.iter().filter(|candidate| candidate.kind == "selector").map(|candidate| json!({
            "name":candidate.name,"viewKind":"canonical","canonicalName":candidate.name,"range":candidate.range,
            "nestedSafety":null,"composes":null,"bemSuffix":null,
        })).collect::<Vec<_>>();
        json!({"filePath":style.style_path,"source":style.style_source,"document":{"selectors":selectors}})
    }).collect::<Vec<_>>();
    let type_facts = targets.iter().filter_map(|target| {
        let entry = entries.iter().find(|entry| entry.expression_id == target.expression_id)?;
        if entry.resolved_type.kind != "union" { return None; }
        let mut values = entry.resolved_type.values.iter().filter(|value| value.chars().all(is_css_name_continue))
            .map(|value| format!("{}{}{}", target.prefix, value, target.suffix)).filter(|value| !value.is_empty()).collect::<Vec<_>>();
        values.sort(); values.dedup(); if values.is_empty() { return None; }
        Some(json!({"filePath":source_path,"expressionId":target.expression_id,"facts":{
            "kind":"finiteSet","constraintKind":null,"values":values,"prefix":null,"suffix":null,"minLen":null,"maxLen":null,
            "charMust":null,"charMay":null,"mayIncludeOtherChars":null,
        }}))
    }).collect::<Vec<_>>();
    if type_facts.is_empty() {
        return Ok(Vec::new());
    }
    let input: crate::EngineInputV2 = serde_json::from_value(json!({"version":"2",
        "workspace":{"root":"","classnameTransform":"asIs","settingsKey":"lsp-source-type-facts"},
        "sources":[{"filePath":source_path,"document":{"classExpressions":class_expressions}}],"styles":style_inputs,"typeFacts":type_facts,
    })).map_err(|_| provider_error("provider projection input was not admitted by the engine contract"))?;
    let projection = crate::summarize_omena_query_expression_domain_selector_projection(&input);
    let mut projected = Vec::new();
    for entry in projection.projections {
        if let Some(target) = targets
            .iter()
            .find(|target| target.expression_id == entry.node_id)
        {
            for selector in entry.selector_names {
                projected.push((target.clone(), selector));
            }
        }
    }
    projected.sort_by(|a, b| {
        (&a.0.expression_id, a.0.byte_span.start, &a.1).cmp(&(
            &b.0.expression_id,
            b.0.byte_span.start,
            &b.1,
        ))
    });
    projected.dedup_by(|a, b| {
        a.0.expression_id == b.0.expression_id && a.0.byte_span == b.0.byte_span && a.1 == b.1
    });
    Ok(projected)
}

fn provider_error(message: &str) -> OmenaError {
    OmenaError::new(
        OmenaErrorClassV0::Workspace,
        message,
        OmenaErrorContextV0 {
            code: "workspace.source-provider-admission".to_string(),
            severity: OmenaErrorSeverityV0::Error,
            recoverability: OmenaErrorRecoverabilityV0::Retry,
            evidence: Vec::new(),
        },
    )
}

const SOURCE_TYPE_FACT_OUTCOME_RESOLVED: &str = "resolved";
const SOURCE_TYPE_FACT_REASON_OUTCOME_NOT_RESOLVED: &str = "outcomeNotResolved";
const SOURCE_TYPE_FACT_REASON_SPAN_NOT_EXACT: &str = "spanNotExact";
const SOURCE_TYPE_FACT_REASON_EMPTY_EXACT_DOMAIN: &str = "emptyExactDomain";
const SOURCE_TYPE_FACT_REASON_MEMBER_COUNT_MISMATCH: &str = "memberCountMismatch";
const SOURCE_TYPE_FACT_REASON_NON_UNION_RESOLVED_TYPE: &str = "nonUnionResolvedType";
const SOURCE_TYPE_FACT_REASON_EMPTY_RESOLVED_VALUES: &str = "emptyResolvedValues";
const SOURCE_TYPE_FACT_REASON_INVALID_CSS_IDENTIFIER_CHARACTER: &str =
    "invalidCssIdentifierCharacter";
const SOURCE_TYPE_FACT_REASON_UNSAFE_CSS_IDENTIFIER: &str = "unsafeCssIdentifier";
fn span_type_fact_entry_admissibility(
    entry: &OmenaWorkspaceProviderSpanResultV0,
    prefix: &str,
    suffix: &str,
) -> Result<(), &'static str> {
    if entry.outcome != SOURCE_TYPE_FACT_OUTCOME_RESOLVED {
        return Err(SOURCE_TYPE_FACT_REASON_OUTCOME_NOT_RESOLVED);
    }
    if !entry.span_exact {
        return Err(SOURCE_TYPE_FACT_REASON_SPAN_NOT_EXACT);
    }
    if entry.non_nullish_member_count == 0 {
        return Err(SOURCE_TYPE_FACT_REASON_EMPTY_EXACT_DOMAIN);
    }
    if entry.non_nullish_member_count != entry.resolved_member_count {
        return Err(SOURCE_TYPE_FACT_REASON_MEMBER_COUNT_MISMATCH);
    }
    if entry.resolved_type.kind != "union" {
        return Err(SOURCE_TYPE_FACT_REASON_NON_UNION_RESOLVED_TYPE);
    }
    if entry.resolved_type.values.is_empty() {
        return Err(SOURCE_TYPE_FACT_REASON_EMPTY_RESOLVED_VALUES);
    }
    if !entry
        .resolved_type
        .values
        .iter()
        .all(|value| value.chars().all(is_css_name_continue))
    {
        return Err(SOURCE_TYPE_FACT_REASON_INVALID_CSS_IDENTIFIER_CHARACTER);
    }
    if !entry
        .resolved_type
        .values
        .iter()
        .all(|value| is_safe_css_identifier(format!("{prefix}{value}{suffix}").as_str()))
    {
        return Err(SOURCE_TYPE_FACT_REASON_UNSAFE_CSS_IDENTIFIER);
    }
    Ok(())
}

fn span_type_fact_shape_is_supported(shape: SourceTypeFactExpressionShape) -> bool {
    matches!(
        shape,
        SourceTypeFactExpressionShape::Call
            | SourceTypeFactExpressionShape::Arithmetic
            | SourceTypeFactExpressionShape::LogicalOperator
            | SourceTypeFactExpressionShape::ComputedNonLiteral
            | SourceTypeFactExpressionShape::NestedTemplate
    )
}

fn span_type_fact_template_affixes(
    source: &str,
    expression_span: ParserByteSpanV0,
) -> Option<(String, String)> {
    let before_expression = source.get(..expression_span.start)?;
    let Some(interpolation_start) = before_expression.rfind("${") else {
        return Some((String::new(), String::new()));
    };
    let before_in_interpolation = source.get(interpolation_start + 2..expression_span.start)?;
    if !before_in_interpolation.chars().all(char::is_whitespace) {
        // A completed earlier template is unrelated to this expression. An
        // active wrapper is ambiguous, so retain the lexical prefix instead.
        return if before_in_interpolation.contains(['}', '`']) {
            Some((String::new(), String::new()))
        } else {
            None
        };
    }

    let after_expression = source.get(expression_span.end..)?;
    let relative_interpolation_end = after_expression.find('}')?;
    let interpolation_end = expression_span.end + relative_interpolation_end;
    if !source
        .get(expression_span.end..interpolation_end)?
        .chars()
        .all(char::is_whitespace)
    {
        return None;
    }

    let prefix_start = source
        .get(..interpolation_start)?
        .char_indices()
        .rev()
        .take_while(|(_, character)| is_ascii_word_continue(*character))
        .last()
        .map(|(index, _)| index)
        .unwrap_or(interpolation_start);
    let suffix_start = interpolation_end + 1;
    let suffix_end = source
        .get(suffix_start..)?
        .char_indices()
        .take_while(|(_, character)| is_ascii_word_continue(*character))
        .last()
        .map(|(index, character)| suffix_start + index + character.len_utf8())
        .unwrap_or(suffix_start);
    Some((
        source.get(prefix_start..interpolation_start)?.to_string(),
        source.get(suffix_start..suffix_end)?.to_string(),
    ))
}

fn take_type_fact_prefix_references(
    references: &mut Vec<SourceSelectorReferenceFact>,
    prefix_span: ParserByteSpanV0,
    prefix: &str,
    target_style_uri: Option<&str>,
) -> Vec<SourceSelectorReferenceFact> {
    if prefix.is_empty() {
        return Vec::new();
    }
    let mut retired = Vec::new();
    references.retain(|reference| {
        let matches = reference.match_kind == SourceSelectorReferenceMatchKind::Prefix
            && reference.surface == SourceSelectorReferenceSurface::OmenaQuerySourceSyntaxIndex
            && reference.byte_span == prefix_span
            && reference.selector_name.as_deref() == Some(prefix)
            && reference.target_style_uri.as_deref() == target_style_uri;
        if matches {
            retired.push(reference.clone());
        }
        !matches
    });
    retired
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct TypeFactTemplateSpans {
    selector_span: ParserByteSpanV0,
    prefix_span: ParserByteSpanV0,
}

fn type_fact_template_spans(
    source: &str,
    target: &SourceTypeFactTarget,
) -> Option<TypeFactTemplateSpans> {
    let before_expression = source.get(..target.byte_span.start)?;
    let interpolation_start = before_expression.rfind("${")?;
    if !source
        .get(interpolation_start + 2..target.byte_span.start)?
        .chars()
        .all(char::is_whitespace)
    {
        return None;
    }
    let prefix_start = interpolation_start.checked_sub(target.prefix.len())?;
    if source.get(prefix_start..interpolation_start)? != target.prefix {
        return None;
    }
    let after_expression = source.get(target.byte_span.end..)?;
    let relative_interpolation_end = after_expression.find('}')?;
    let interpolation_end = target.byte_span.end + relative_interpolation_end;
    if !source
        .get(target.byte_span.end..interpolation_end)?
        .chars()
        .all(char::is_whitespace)
    {
        return None;
    }
    let suffix_start = interpolation_end + 1;
    let suffix_end = suffix_start.checked_add(target.suffix.len())?;
    if source.get(suffix_start..suffix_end)? != target.suffix {
        return None;
    }
    Some(TypeFactTemplateSpans {
        selector_span: ParserByteSpanV0 {
            start: prefix_start,
            end: suffix_end,
        },
        prefix_span: ParserByteSpanV0 {
            start: prefix_start,
            end: interpolation_start,
        },
    })
}

fn complete_tsgo_projection_expression_ids(
    targets: &[SourceTypeFactTarget],
    entries: &[OmenaWorkspaceProviderResultV0],
    projections: &[(SourceTypeFactTarget, String)],
) -> BTreeSet<String> {
    let entries_by_id = entries
        .iter()
        .map(|entry| (entry.expression_id.as_str(), entry))
        .collect::<BTreeMap<_, _>>();
    let mut projected_names_by_id = BTreeMap::<&str, Vec<&str>>::new();
    for (target, selector_name) in projections {
        projected_names_by_id
            .entry(target.expression_id.as_str())
            .or_default()
            .push(selector_name.as_str());
    }

    targets
        .iter()
        .filter_map(|target| {
            let entry = entries_by_id.get(target.expression_id.as_str())?;
            if entry.resolved_type.kind != "union" || entry.resolved_type.values.is_empty() {
                return None;
            }
            if entry
                .resolved_type
                .values
                .iter()
                .any(|value| !value.chars().all(is_css_name_continue))
            {
                return None;
            }
            let mut expected = entry
                .resolved_type
                .values
                .iter()
                .map(|value| format!("{}{}{}", target.prefix, value, target.suffix))
                .collect::<Vec<_>>();
            if expected.is_empty()
                || expected
                    .iter()
                    .any(|selector_name| !is_safe_css_identifier(selector_name))
            {
                return None;
            }
            expected.sort();
            expected.dedup();
            let mut projected = projected_names_by_id
                .get(target.expression_id.as_str())
                .cloned()
                .unwrap_or_default();
            projected.sort();
            projected.dedup();
            (projected.len() == expected.len()
                && projected
                    .iter()
                    .zip(expected.iter())
                    .all(|(projected, expected)| *projected == expected))
            .then(|| target.expression_id.clone())
        })
        .collect()
}

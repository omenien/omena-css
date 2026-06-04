use std::{
    collections::{BTreeMap, BTreeSet},
    path::{Component, Path, PathBuf},
};

use omena_parser::ParsedCssModuleComposesEdgeKind;

use super::*;

pub fn summarize_omena_query_style_extract_code_actions(
    style_uri: &str,
    source: &str,
    range: ParserRangeV0,
) -> OmenaQueryCodeActionPlanV0 {
    let mut actions = Vec::new();

    if let Some(value) = selected_extractable_css_value(source, range) {
        let property_name = next_custom_property_name(source, custom_property_stem(value));
        actions.push(OmenaQueryCodeActionV0 {
            title: format!("Extract CSS custom property '{property_name}'"),
            kind: "refactor.extract",
            edits: vec![
                OmenaQueryWorkspaceTextEditV0 {
                    uri: style_uri.to_string(),
                    range: start_of_source_range(),
                    new_text: format!(":root {{\n  {property_name}: {value};\n}}\n\n"),
                },
                OmenaQueryWorkspaceTextEditV0 {
                    uri: style_uri.to_string(),
                    range,
                    new_text: format!("var({property_name})"),
                },
            ],
            source: "omenaQueryStyleExtractCodeActions",
        });

        let value_name = next_value_name(source, value_name_stem(value));
        actions.push(OmenaQueryCodeActionV0 {
            title: format!("Extract @value '{value_name}'"),
            kind: "refactor.extract",
            edits: vec![
                OmenaQueryWorkspaceTextEditV0 {
                    uri: style_uri.to_string(),
                    range: start_of_source_range(),
                    new_text: format!("@value {value_name}: {value};\n\n"),
                },
                OmenaQueryWorkspaceTextEditV0 {
                    uri: style_uri.to_string(),
                    range,
                    new_text: value_name,
                },
            ],
            source: "omenaQueryStyleExtractCodeActions",
        });
    }

    OmenaQueryCodeActionPlanV0 {
        schema_version: "0",
        product: "omena-query.code-actions",
        file_uri: style_uri.to_string(),
        file_kind: "style",
        action_count: actions.len(),
        actions,
        ready_surfaces: vec!["styleExtractRefactorActions", "productFacingCodeActions"],
    }
}

pub fn summarize_omena_query_style_inline_code_actions(
    style_uri: &str,
    style_sources: &[OmenaQueryStyleSourceInputV0],
    range: ParserRangeV0,
    package_manifests: &[OmenaQueryStylePackageManifestV0],
) -> OmenaQueryCodeActionPlanV0 {
    let Some(target_source) = style_sources
        .iter()
        .find(|source| source.style_path == style_uri)
    else {
        return empty_style_code_action_plan(style_uri, "styleInlineRefactorActions");
    };
    let Some(range_start) =
        byte_offset_for_parser_position(&target_source.style_source, range.start)
    else {
        return empty_style_code_action_plan(style_uri, "styleInlineRefactorActions");
    };

    let style_source_by_path = style_sources
        .iter()
        .map(|source| (source.style_path.as_str(), source.style_source.as_str()))
        .collect::<BTreeMap<_, _>>();
    let available_style_paths = style_sources
        .iter()
        .map(|source| source.style_path.as_str())
        .collect::<BTreeSet<_>>();
    let dialect = omena_parser_dialect_for_style_path(style_uri);
    let facts = collect_omena_query_omena_parser_style_facts_raw(
        target_source.style_source.as_str(),
        dialect,
    );

    for edge in facts.css_module_composes_edges {
        let edge_start: u32 = edge.range.start().into();
        let edge_end: u32 = edge.range.end().into();
        let edge_span = ParserByteSpanV0 {
            start: edge_start as usize,
            end: edge_end as usize,
        };
        if range_start < edge_span.start || range_start > edge_span.end {
            continue;
        }
        if edge.kind == ParsedCssModuleComposesEdgeKind::Global {
            continue;
        }

        let mut context = InlineDeclarationContext {
            style_source_by_path: &style_source_by_path,
            available_style_paths: &available_style_paths,
            package_manifests,
            emitted: BTreeSet::new(),
            visiting: BTreeSet::new(),
        };
        let mut declarations = Vec::new();
        for target_name in &edge.target_names {
            let Some(target_style_path) = resolve_inline_target_style_path(
                style_uri,
                &edge,
                &available_style_paths,
                package_manifests,
            ) else {
                return empty_style_code_action_plan(style_uri, "styleInlineRefactorActions");
            };
            let Some(target_declarations) = collect_inline_declarations(
                target_style_path.as_str(),
                target_name.as_str(),
                &mut context,
            ) else {
                return empty_style_code_action_plan(style_uri, "styleInlineRefactorActions");
            };
            declarations.extend(target_declarations);
        }
        if declarations.is_empty() {
            continue;
        }

        let replacement_range =
            expand_inline_composes_statement_range(target_source.style_source.as_str(), edge_span);
        let replacement_text = format_inline_declarations(
            declarations.as_slice(),
            line_indent_at(
                target_source.style_source.as_str(),
                replacement_range.start.line,
            )
            .as_str(),
        );
        let action = OmenaQueryCodeActionV0 {
            title: format_inline_action_title(edge.target_names.as_slice()),
            kind: "refactor.inline",
            edits: vec![OmenaQueryWorkspaceTextEditV0 {
                uri: style_uri.to_string(),
                range: replacement_range,
                new_text: replacement_text,
            }],
            source: "omenaQueryStyleInlineCodeActions",
        };
        return OmenaQueryCodeActionPlanV0 {
            schema_version: "0",
            product: "omena-query.code-actions",
            file_uri: style_uri.to_string(),
            file_kind: "style",
            action_count: 1,
            actions: vec![action],
            ready_surfaces: vec!["styleInlineRefactorActions", "productFacingCodeActions"],
        };
    }

    empty_style_code_action_plan(style_uri, "styleInlineRefactorActions")
}

fn empty_style_code_action_plan(
    style_uri: &str,
    ready_surface: &'static str,
) -> OmenaQueryCodeActionPlanV0 {
    OmenaQueryCodeActionPlanV0 {
        schema_version: "0",
        product: "omena-query.code-actions",
        file_uri: style_uri.to_string(),
        file_kind: "style",
        action_count: 0,
        actions: Vec::new(),
        ready_surfaces: vec![ready_surface, "productFacingCodeActions"],
    }
}

pub fn summarize_omena_query_style_refactor_code_actions(
    style_uri: &str,
    style_sources: &[OmenaQueryStyleSourceInputV0],
    source: &str,
    range: ParserRangeV0,
    package_manifests: &[OmenaQueryStylePackageManifestV0],
) -> OmenaQueryCodeActionPlanV0 {
    let inline = summarize_omena_query_style_inline_code_actions(
        style_uri,
        style_sources,
        range,
        package_manifests,
    );
    if inline.action_count > 0 {
        return inline;
    }

    let insight = summarize_omena_query_style_insight_code_actions(style_uri, source, range);
    if insight.action_count > 0 {
        return insight;
    }

    summarize_omena_query_style_extract_code_actions(style_uri, source, range)
}

struct InlineDeclarationContext<'a> {
    style_source_by_path: &'a BTreeMap<&'a str, &'a str>,
    available_style_paths: &'a BTreeSet<&'a str>,
    package_manifests: &'a [OmenaQueryStylePackageManifestV0],
    emitted: BTreeSet<(String, String)>,
    visiting: BTreeSet<(String, String)>,
}

fn collect_inline_declarations(
    style_path: &str,
    selector_name: &str,
    context: &mut InlineDeclarationContext<'_>,
) -> Option<Vec<String>> {
    let key = (style_path.to_string(), selector_name.to_string());
    if context.emitted.contains(&key) {
        return Some(Vec::new());
    }
    if !context.visiting.insert(key.clone()) {
        return None;
    }

    let source = *context.style_source_by_path.get(style_path)?;
    let dialect = omena_parser_dialect_for_style_path(style_path);
    let facts = collect_omena_query_omena_parser_style_facts_raw(source, dialect);
    let mut declarations = Vec::new();

    for edge in facts.css_module_composes_edges.iter().filter(|edge| {
        edge.owner_selector_names
            .iter()
            .any(|owner| owner == selector_name)
    }) {
        if edge.kind == ParsedCssModuleComposesEdgeKind::Global {
            return None;
        }
        let target_style_path = resolve_inline_target_style_path(
            style_path,
            edge,
            context.available_style_paths,
            context.package_manifests,
        )?;
        for target_name in &edge.target_names {
            declarations.extend(collect_inline_declarations(
                target_style_path.as_str(),
                target_name.as_str(),
                context,
            )?);
        }
    }

    declarations.extend(selector_inline_declarations(source, selector_name)?);
    context.visiting.remove(&key);
    context.emitted.insert(key);
    Some(declarations)
}

fn resolve_inline_target_style_path(
    owner_style_path: &str,
    edge: &omena_parser::ParsedCssModuleComposesEdgeFact,
    available_style_paths: &BTreeSet<&str>,
    package_manifests: &[OmenaQueryStylePackageManifestV0],
) -> Option<String> {
    if edge.kind == ParsedCssModuleComposesEdgeKind::Local {
        return Some(owner_style_path.to_string());
    }
    let source = edge.import_source.as_deref()?;
    resolve_style_module_source(
        owner_style_path,
        source,
        available_style_paths,
        package_manifests,
    )
    .or_else(|| {
        resolve_file_uri_relative_style_module_source(
            owner_style_path,
            source,
            available_style_paths,
        )
    })
}

fn resolve_file_uri_relative_style_module_source(
    owner_style_uri: &str,
    source: &str,
    available_style_paths: &BTreeSet<&str>,
) -> Option<String> {
    if !source.starts_with('.') {
        return None;
    }
    let owner_path = owner_style_uri.strip_prefix("file://")?;
    let owner_dir = Path::new(owner_path).parent()?;
    let source_path = owner_dir.join(source);
    let mut candidates = Vec::new();
    push_file_uri_candidate_paths(
        &mut candidates,
        source_path.clone(),
        Path::new(source).extension().is_none(),
    );
    candidates.into_iter().find(|candidate| {
        available_style_paths
            .iter()
            .any(|available| normalize_file_uri(available).as_deref() == Some(candidate.as_str()))
    })
}

fn push_file_uri_candidate_paths(
    candidates: &mut Vec<String>,
    base_path: PathBuf,
    needs_extension: bool,
) {
    if needs_extension {
        for extension in ["css", "scss", "sass", "less"] {
            let mut candidate = base_path.clone();
            candidate.set_extension(extension);
            if let Some(uri) = file_uri_from_path(candidate.as_path()) {
                candidates.push(uri);
            }
        }
    } else if let Some(uri) = file_uri_from_path(base_path.as_path()) {
        candidates.push(uri);
    }
}

fn file_uri_from_path(path: &Path) -> Option<String> {
    let mut normalized = PathBuf::new();
    for component in path.components() {
        match component {
            Component::RootDir => normalized.push("/"),
            Component::CurDir => {}
            Component::ParentDir => {
                normalized.pop();
            }
            Component::Normal(value) => normalized.push(value),
            Component::Prefix(_) => return None,
        }
    }
    Some(format!(
        "file://{}",
        normalized.to_string_lossy().replace('\\', "/")
    ))
}

fn normalize_file_uri(uri: &str) -> Option<String> {
    file_uri_from_path(Path::new(uri.strip_prefix("file://")?))
}

fn selector_inline_declarations(source: &str, selector_name: &str) -> Option<Vec<String>> {
    let body = selector_rule_body(source, selector_name)?;
    let declarations = body
        .split(';')
        .map(str::trim)
        .filter(|declaration| !declaration.is_empty())
        .filter(|declaration| !declaration.starts_with("composes:"))
        .map(ToString::to_string)
        .collect::<Vec<_>>();
    Some(declarations)
}

fn selector_rule_body<'a>(source: &'a str, selector_name: &str) -> Option<&'a str> {
    let needle = format!(".{selector_name}");
    let mut search_start = 0usize;
    while let Some(relative_index) = source.get(search_start..)?.find(needle.as_str()) {
        let selector_start = search_start + relative_index;
        let before_is_identifier = source
            .get(..selector_start)
            .and_then(|prefix| prefix.chars().next_back())
            .is_some_and(is_css_selector_identifier_char);
        let selector_end = selector_start + needle.len();
        let after_is_identifier = source
            .get(selector_end..)
            .and_then(|suffix| suffix.chars().next())
            .is_some_and(is_css_selector_identifier_char);
        if before_is_identifier || after_is_identifier {
            search_start = selector_end;
            continue;
        }
        let open_brace = selector_end + source.get(selector_end..)?.find('{')?;
        let close_brace = matching_close_brace(source, open_brace)?;
        return source.get(open_brace + 1..close_brace);
    }
    None
}

fn matching_close_brace(source: &str, open_brace: usize) -> Option<usize> {
    let mut depth = 0usize;
    for (offset, ch) in source.get(open_brace..)?.char_indices() {
        if ch == '{' {
            depth += 1;
        } else if ch == '}' {
            depth = depth.checked_sub(1)?;
            if depth == 0 {
                return Some(open_brace + offset);
            }
        }
    }
    None
}

fn expand_inline_composes_statement_range(source: &str, span: ParserByteSpanV0) -> ParserRangeV0 {
    let mut end = span.end;
    while source
        .as_bytes()
        .get(end)
        .is_some_and(u8::is_ascii_whitespace)
    {
        end += 1;
    }
    if source.as_bytes().get(end) == Some(&b';') {
        end += 1;
    }
    parser_range_for_byte_span(
        source,
        ParserByteSpanV0 {
            start: span.start,
            end,
        },
    )
}

fn line_indent_at(source: &str, line: usize) -> String {
    source
        .lines()
        .nth(line)
        .map(|line| {
            line.chars()
                .take_while(|ch| *ch == ' ' || *ch == '\t')
                .collect()
        })
        .unwrap_or_default()
}

fn format_inline_declarations(declarations: &[String], continuation_indent: &str) -> String {
    declarations
        .iter()
        .enumerate()
        .map(|(index, declaration)| {
            if index == 0 {
                format!("{declaration};")
            } else {
                format!("{continuation_indent}{declaration};")
            }
        })
        .collect::<Vec<_>>()
        .join("\n")
}

fn format_inline_action_title(target_names: &[String]) -> String {
    if target_names.len() == 1 {
        return format!("Inline composed class '{}'", target_names[0]);
    }
    format!("Inline composed classes '{}'", target_names.join(", "))
}

fn is_css_selector_identifier_char(ch: char) -> bool {
    ch.is_ascii_alphanumeric() || matches!(ch, '-' | '_' | '\\')
}

fn selected_extractable_css_value(source: &str, range: ParserRangeV0) -> Option<&str> {
    let start = byte_offset_for_parser_position(source, range.start)?;
    let end = byte_offset_for_parser_position(source, range.end)?;
    if end <= start {
        return None;
    }
    let value = source.get(start..end)?.trim();
    is_extractable_css_value(value).then_some(value)
}

fn start_of_source_range() -> ParserRangeV0 {
    ParserRangeV0 {
        start: ParserPositionV0 {
            line: 0,
            character: 0,
        },
        end: ParserPositionV0 {
            line: 0,
            character: 0,
        },
    }
}

fn is_extractable_css_value(value: &str) -> bool {
    if value.is_empty() || value.contains('\n') || value.starts_with("var(") {
        return false;
    }
    is_hex_color(value)
        || numeric_value_unit_kind(value).is_some()
        || is_color_function(value)
        || is_ascii_css_keyword(value)
}

fn is_hex_color(value: &str) -> bool {
    value.strip_prefix('#').is_some_and(|hex| {
        (3..=8).contains(&hex.len()) && hex.chars().all(|ch| ch.is_ascii_hexdigit())
    })
}

fn numeric_value_unit_kind(value: &str) -> Option<&'static str> {
    let mut chars = value.char_indices().peekable();
    if matches!(chars.peek(), Some((_, '-'))) {
        chars.next();
    }

    let mut digit_count = 0usize;
    let mut last_number_end = 0usize;
    while let Some((index, ch)) = chars.peek().copied() {
        if !ch.is_ascii_digit() {
            break;
        }
        digit_count += 1;
        last_number_end = index + ch.len_utf8();
        chars.next();
    }
    if digit_count == 0 {
        return None;
    }

    if matches!(chars.peek(), Some((_, '.'))) {
        chars.next();
        let mut fractional_digit_count = 0usize;
        while let Some((index, ch)) = chars.peek().copied() {
            if !ch.is_ascii_digit() {
                break;
            }
            fractional_digit_count += 1;
            last_number_end = index + ch.len_utf8();
            chars.next();
        }
        if fractional_digit_count == 0 {
            return None;
        }
    }

    let unit = value.get(last_number_end..)?;
    match unit {
        "s" | "ms" => Some("duration"),
        "deg" => Some("angle"),
        "" | "px" | "rem" | "em" | "%" | "vh" | "vw" | "vmin" | "vmax" | "ch" | "ex" => {
            Some("size")
        }
        _ => None,
    }
}

fn is_color_function(value: &str) -> bool {
    ["rgb(", "rgba(", "hsl(", "hsla("]
        .iter()
        .any(|prefix| value.starts_with(prefix))
        && value.ends_with(')')
}

fn is_ascii_css_keyword(value: &str) -> bool {
    let mut chars = value.chars();
    chars.next().is_some_and(|ch| ch.is_ascii_alphabetic())
        && chars.all(|ch| ch.is_ascii_alphabetic() || ch == '-')
}

fn custom_property_stem(value: &str) -> &'static str {
    if is_hex_color(value) || is_color_function(value) {
        return "extracted-color";
    }
    match numeric_value_unit_kind(value) {
        Some("duration") => "extracted-duration",
        Some("angle") => "extracted-angle",
        Some("size") => "extracted-size",
        _ => "extracted-token",
    }
}

fn next_custom_property_name(source: &str, stem: &str) -> String {
    let mut candidate = format!("--{stem}");
    let mut suffix = 2usize;
    while source.contains(candidate.as_str()) {
        candidate = format!("--{stem}-{suffix}");
        suffix += 1;
    }
    candidate
}

fn value_name_stem(value: &str) -> &'static str {
    if is_hex_color(value) || is_color_function(value) {
        return "extractedColor";
    }
    match numeric_value_unit_kind(value) {
        Some("duration") => "extractedDuration",
        Some("angle") => "extractedAngle",
        Some("size") => "extractedSize",
        _ => "extractedToken",
    }
}

fn next_value_name(source: &str, stem: &str) -> String {
    let mut candidate = stem.to_string();
    let mut suffix = 2usize;
    while contains_identifier(source, candidate.as_str()) {
        candidate = format!("{stem}{suffix}");
        suffix += 1;
    }
    candidate
}

fn contains_identifier(source: &str, candidate: &str) -> bool {
    source.match_indices(candidate).any(|(start, _)| {
        let end = start + candidate.len();
        let before_is_identifier = source
            .get(..start)
            .and_then(|prefix| prefix.chars().next_back())
            .is_some_and(is_identifier_char);
        let after_is_identifier = source
            .get(end..)
            .and_then(|suffix| suffix.chars().next())
            .is_some_and(is_identifier_char);
        !before_is_identifier && !after_is_identifier
    })
}

fn is_identifier_char(ch: char) -> bool {
    ch.is_ascii_alphanumeric() || ch == '_'
}

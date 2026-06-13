use cstree::text::TextRange;
use omena_syntax::SyntaxKind;
use std::collections::{BTreeMap, BTreeSet};

use crate::{
    Token, find_block_after_header, matching_right_brace, next_non_trivia_token_index_until,
    previous_non_trivia_token_index, skip_statement, skip_trivia_tokens, style_wrapper_at_rule,
    top_level_token_kind_index, top_level_token_text_index,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParsedCssModuleValueFact {
    pub kind: ParsedCssModuleValueFactKind,
    pub name: String,
    pub range: TextRange,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum ParsedCssModuleValueFactKind {
    Definition,
    Reference,
    ImportSource,
}

pub(crate) fn collect_css_module_value_facts_from_tokens(
    tokens: &[Token<'_>],
) -> Vec<ParsedCssModuleValueFact> {
    let mut values = Vec::new();
    let mut seen = BTreeSet::new();
    let value_path_aliases = collect_css_module_value_path_aliases_from_tokens(tokens);
    for (index, token) in tokens.iter().enumerate() {
        if token.kind != SyntaxKind::AtKeyword || !token.text.eq_ignore_ascii_case("@value") {
            continue;
        }

        let start = skip_trivia_tokens(tokens, index + 1, tokens.len());
        let end = css_module_value_statement_end(tokens, start);
        let colon_index = top_level_token_kind_index(tokens, start, end, SyntaxKind::Colon);
        let from_index = top_level_token_text_index(tokens, start, end, "from");

        if let Some(from_index) = from_index
            && match colon_index {
                Some(colon_index) => from_index < colon_index,
                None => true,
            }
        {
            collect_css_module_value_import_facts(
                tokens,
                start,
                from_index,
                end,
                &value_path_aliases,
                &mut values,
                &mut seen,
            );
            continue;
        }

        if let Some(colon_index) = colon_index {
            if css_module_value_path_alias_from_tokens(tokens, start, colon_index, end).is_some() {
                continue;
            }
            collect_css_module_value_definition_facts(
                tokens,
                start,
                colon_index,
                &mut values,
                &mut seen,
            );
            collect_css_module_value_reference_facts(
                tokens,
                colon_index + 1,
                end,
                &mut values,
                &mut seen,
            );
        } else {
            collect_css_module_value_definition_facts(tokens, start, end, &mut values, &mut seen);
        }
    }
    let local_value_names = values
        .iter()
        .filter(|value| value.kind == ParsedCssModuleValueFactKind::Definition)
        .map(|value| value.name.clone())
        .collect::<BTreeSet<_>>();
    collect_css_module_value_declaration_reference_facts(
        tokens,
        0,
        tokens.len(),
        &local_value_names,
        &mut values,
        &mut seen,
    );
    values
}

fn collect_css_module_value_path_aliases_from_tokens(
    tokens: &[Token<'_>],
) -> BTreeMap<String, String> {
    let mut aliases = BTreeMap::new();
    for (index, token) in tokens.iter().enumerate() {
        if token.kind != SyntaxKind::AtKeyword || !token.text.eq_ignore_ascii_case("@value") {
            continue;
        }

        let start = skip_trivia_tokens(tokens, index + 1, tokens.len());
        let end = css_module_value_statement_end(tokens, start);
        let Some(colon_index) = top_level_token_kind_index(tokens, start, end, SyntaxKind::Colon)
        else {
            continue;
        };
        if top_level_token_text_index(tokens, start, end, "from").is_some() {
            continue;
        }
        if let Some((name, target)) =
            css_module_value_path_alias_from_tokens(tokens, start, colon_index, end)
        {
            aliases.insert(name, target);
        }
    }
    aliases
}

fn css_module_value_path_alias_from_tokens(
    tokens: &[Token<'_>],
    start: usize,
    colon_index: usize,
    end: usize,
) -> Option<(String, String)> {
    let name_index = next_non_trivia_token_index_until(tokens, start, colon_index)?;
    let name_token = tokens[name_index];
    if !css_module_value_name_token_can_define(name_token) {
        return None;
    }
    let source_index = next_non_trivia_token_index_until(tokens, colon_index + 1, end)?;
    let source_token = tokens[source_index];
    if !matches!(source_token.kind, SyntaxKind::String | SyntaxKind::Url) {
        return None;
    }
    let source = css_module_value_source_name(source_token);
    css_module_value_source_looks_like_style_request(&source)
        .then(|| (name_token.text.to_string(), source))
}

pub(crate) fn css_module_value_statement_end(tokens: &[Token<'_>], start: usize) -> usize {
    let mut index = start;
    let mut paren_depth = 0usize;
    let mut bracket_depth = 0usize;
    while index < tokens.len() {
        match tokens[index].kind {
            SyntaxKind::LeftParen => paren_depth += 1,
            SyntaxKind::RightParen => paren_depth = paren_depth.saturating_sub(1),
            SyntaxKind::LeftBracket => bracket_depth += 1,
            SyntaxKind::RightBracket => bracket_depth = bracket_depth.saturating_sub(1),
            SyntaxKind::Semicolon
            | SyntaxKind::SassOptionalSemicolon
            | SyntaxKind::LeftBrace
            | SyntaxKind::RightBrace
            | SyntaxKind::SassIndent
            | SyntaxKind::SassDedent
                if paren_depth == 0 && bracket_depth == 0 =>
            {
                return index;
            }
            _ => {}
        }
        index += 1;
    }
    index
}

fn collect_css_module_value_import_facts(
    tokens: &[Token<'_>],
    start: usize,
    from_index: usize,
    end: usize,
    value_path_aliases: &BTreeMap<String, String>,
    values: &mut Vec<ParsedCssModuleValueFact>,
    seen: &mut BTreeSet<(ParsedCssModuleValueFactKind, String, u32, u32)>,
) {
    collect_css_module_value_import_names(tokens, start, from_index, values, seen);
    if let Some((source_name, source_range)) =
        css_module_value_import_edge_source(tokens, from_index + 1, end, value_path_aliases)
    {
        push_css_module_value_fact(
            values,
            seen,
            ParsedCssModuleValueFactKind::ImportSource,
            source_name,
            source_range,
        );
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParsedCssModuleValueImportEdgeFact {
    pub remote_name: String,
    pub local_name: String,
    pub import_source: String,
    pub local_range: TextRange,
    pub remote_range: TextRange,
    pub range: TextRange,
}

pub(crate) fn collect_css_module_value_import_edge_facts_from_tokens(
    tokens: &[Token<'_>],
) -> Vec<ParsedCssModuleValueImportEdgeFact> {
    let mut edges = Vec::new();
    let value_path_aliases = collect_css_module_value_path_aliases_from_tokens(tokens);
    for (index, token) in tokens.iter().enumerate() {
        if token.kind != SyntaxKind::AtKeyword || !token.text.eq_ignore_ascii_case("@value") {
            continue;
        }

        let start = skip_trivia_tokens(tokens, index + 1, tokens.len());
        let end = css_module_value_statement_end(tokens, start);
        let colon_index = top_level_token_kind_index(tokens, start, end, SyntaxKind::Colon);
        let from_index = top_level_token_text_index(tokens, start, end, "from");
        let Some(from_index) = from_index else {
            continue;
        };
        if colon_index.is_some_and(|colon_index| from_index > colon_index) {
            continue;
        }
        let Some((import_source, _source_range)) =
            css_module_value_import_edge_source(tokens, from_index + 1, end, &value_path_aliases)
        else {
            continue;
        };

        collect_css_module_value_import_edges(tokens, start, from_index, import_source, &mut edges);
    }
    edges
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParsedCssModuleValueDefinitionEdgeFact {
    pub definition_name: String,
    pub reference_names: Vec<String>,
    pub range: TextRange,
}

pub(crate) fn collect_css_module_value_definition_edge_facts_from_tokens(
    tokens: &[Token<'_>],
) -> Vec<ParsedCssModuleValueDefinitionEdgeFact> {
    let mut edges = Vec::new();
    for (index, token) in tokens.iter().enumerate() {
        if token.kind != SyntaxKind::AtKeyword || !token.text.eq_ignore_ascii_case("@value") {
            continue;
        }

        let start = skip_trivia_tokens(tokens, index + 1, tokens.len());
        let end = css_module_value_statement_end(tokens, start);
        let colon_index = top_level_token_kind_index(tokens, start, end, SyntaxKind::Colon);
        let from_index = top_level_token_text_index(tokens, start, end, "from");
        let Some(colon_index) = colon_index else {
            continue;
        };
        if from_index.is_some_and(|from_index| from_index < colon_index) {
            continue;
        }

        let definition_names = collect_css_module_value_definition_edge_names(
            tokens,
            start,
            colon_index,
            |tokens, index| css_module_value_name_token_can_define(tokens[index]),
        );
        let reference_names = collect_css_module_value_definition_edge_names(
            tokens,
            colon_index + 1,
            end,
            css_module_value_reference_token_can_be_name,
        );
        if reference_names.is_empty() {
            continue;
        }
        let range_end = end
            .checked_sub(1)
            .and_then(|end| tokens.get(end))
            .map(|token| token.range.end())
            .unwrap_or_else(|| tokens[index].range.end());

        for definition_name in definition_names {
            edges.push(ParsedCssModuleValueDefinitionEdgeFact {
                definition_name,
                reference_names: reference_names.clone(),
                range: TextRange::new(tokens[index].range.start(), range_end),
            });
        }
    }
    edges
}

pub(crate) fn collect_css_module_value_definition_edge_names(
    tokens: &[Token<'_>],
    start: usize,
    end: usize,
    predicate: impl Fn(&[Token<'_>], usize) -> bool,
) -> Vec<String> {
    let mut names = Vec::new();
    let mut index = start;
    while index < end {
        if predicate(tokens, index) && !names.iter().any(|name| name == tokens[index].text) {
            names.push(tokens[index].text.to_string());
        }
        index += 1;
    }
    names
}

fn css_module_value_import_edge_source(
    tokens: &[Token<'_>],
    start: usize,
    end: usize,
    value_path_aliases: &BTreeMap<String, String>,
) -> Option<(String, TextRange)> {
    let source_index = next_non_trivia_token_index_until(tokens, start, end)?;
    let token = tokens[source_index];
    if matches!(token.kind, SyntaxKind::String | SyntaxKind::Url) {
        return Some((css_module_value_source_name(token), token.range));
    }
    if css_module_value_name_token_can_define(token) {
        return css_module_value_source_alias_target(token.text, token.range, value_path_aliases);
    }
    None
}

fn css_module_value_source_alias_target(
    name: &str,
    range: TextRange,
    value_path_aliases: &BTreeMap<String, String>,
) -> Option<(String, TextRange)> {
    value_path_aliases
        .get(name)
        .map(|source| (source.clone(), range))
}

fn collect_css_module_value_import_edges(
    tokens: &[Token<'_>],
    start: usize,
    end: usize,
    import_source: String,
    edges: &mut Vec<ParsedCssModuleValueImportEdgeFact>,
) {
    let mut index = start;
    while index < end {
        let token = tokens[index];
        if !css_module_value_name_token_can_define(token) {
            index += 1;
            continue;
        }
        if previous_non_trivia_token_index(tokens, index, start)
            .is_some_and(|previous| tokens[previous].text == "as")
        {
            index += 1;
            continue;
        }
        let remote_name = token.text.to_string();
        let mut local_name = remote_name.clone();
        let mut local_range = token.range;
        if let Some(as_index) = next_non_trivia_token_index_until(tokens, index + 1, end)
            && tokens[as_index].text == "as"
            && let Some(local_index) = next_non_trivia_token_index_until(tokens, as_index + 1, end)
            && css_module_value_name_token_can_define(tokens[local_index])
        {
            local_name = tokens[local_index].text.to_string();
            local_range = tokens[local_index].range;
            index = local_index + 1;
        } else {
            index += 1;
        }
        edges.push(ParsedCssModuleValueImportEdgeFact {
            remote_name,
            local_name,
            import_source: import_source.clone(),
            local_range,
            remote_range: token.range,
            range: token.range,
        });
    }
}

fn collect_css_module_value_import_names(
    tokens: &[Token<'_>],
    start: usize,
    end: usize,
    values: &mut Vec<ParsedCssModuleValueFact>,
    seen: &mut BTreeSet<(ParsedCssModuleValueFactKind, String, u32, u32)>,
) {
    let mut index = start;
    while index < end {
        let token = tokens[index];
        if css_module_value_name_token_can_define(token) {
            let previous = previous_non_trivia_token_index(tokens, index, start);
            let next = next_non_trivia_token_index_until(tokens, index + 1, end);
            let kind = if previous.is_some_and(|previous| tokens[previous].text == "as") {
                Some(ParsedCssModuleValueFactKind::Definition)
            } else if next.is_some_and(|next| tokens[next].text == "as") {
                Some(ParsedCssModuleValueFactKind::Reference)
            } else {
                Some(ParsedCssModuleValueFactKind::Definition)
            };
            if let Some(kind) = kind {
                push_css_module_value_fact(values, seen, kind, token.text.to_string(), token.range);
            }
        }
        index += 1;
    }
}

fn collect_css_module_value_definition_facts(
    tokens: &[Token<'_>],
    start: usize,
    end: usize,
    values: &mut Vec<ParsedCssModuleValueFact>,
    seen: &mut BTreeSet<(ParsedCssModuleValueFactKind, String, u32, u32)>,
) {
    let mut index = start;
    while index < end {
        let token = tokens[index];
        if css_module_value_name_token_can_define(token) {
            push_css_module_value_fact(
                values,
                seen,
                ParsedCssModuleValueFactKind::Definition,
                token.text.to_string(),
                token.range,
            );
        }
        index += 1;
    }
}

fn collect_css_module_value_reference_facts(
    tokens: &[Token<'_>],
    start: usize,
    end: usize,
    values: &mut Vec<ParsedCssModuleValueFact>,
    seen: &mut BTreeSet<(ParsedCssModuleValueFactKind, String, u32, u32)>,
) {
    let mut index = start;
    let mut paren_depth = 0usize;
    let mut bracket_depth = 0usize;
    while index < end {
        match tokens[index].kind {
            SyntaxKind::LeftParen => paren_depth += 1,
            SyntaxKind::RightParen => paren_depth = paren_depth.saturating_sub(1),
            SyntaxKind::LeftBracket => bracket_depth += 1,
            SyntaxKind::RightBracket => bracket_depth = bracket_depth.saturating_sub(1),
            _ => {}
        }
        if paren_depth == 0
            && bracket_depth == 0
            && css_module_value_reference_token_can_be_name(tokens, index)
        {
            push_css_module_value_fact(
                values,
                seen,
                ParsedCssModuleValueFactKind::Reference,
                tokens[index].text.to_string(),
                tokens[index].range,
            );
        }
        index += 1;
    }
}

fn collect_css_module_value_declaration_reference_facts(
    tokens: &[Token<'_>],
    start: usize,
    end: usize,
    local_value_names: &BTreeSet<String>,
    values: &mut Vec<ParsedCssModuleValueFact>,
    seen: &mut BTreeSet<(ParsedCssModuleValueFactKind, String, u32, u32)>,
) {
    if local_value_names.is_empty() {
        return;
    }

    let mut index = start;
    while index < end {
        index = skip_trivia_tokens(tokens, index, end);
        if index >= end {
            break;
        }

        if tokens[index].kind == SyntaxKind::AtKeyword {
            let block = find_block_after_header(tokens, index, end);
            if let Some((open, close)) = block {
                if style_wrapper_at_rule(tokens[index].text) {
                    collect_css_module_value_declaration_reference_facts(
                        tokens,
                        open + 1,
                        close,
                        local_value_names,
                        values,
                        seen,
                    );
                }
                index = close + 1;
            } else {
                index = skip_statement(tokens, index, end);
            }
            continue;
        }

        let statement_end = css_module_value_statement_end(tokens, index);
        if statement_end < end && tokens[statement_end].kind == SyntaxKind::LeftBrace {
            if let Some(close) = matching_right_brace(tokens, statement_end, end) {
                collect_css_module_value_declaration_reference_facts(
                    tokens,
                    statement_end + 1,
                    close,
                    local_value_names,
                    values,
                    seen,
                );
                index = close + 1;
            } else {
                index = statement_end + 1;
            }
            continue;
        }

        if let Some(colon_index) = declaration_colon_index(tokens, index, statement_end.min(end)) {
            collect_known_css_module_value_reference_facts(
                tokens,
                colon_index + 1,
                statement_end.min(end),
                local_value_names,
                values,
                seen,
            );
        }

        if statement_end >= end || tokens[statement_end].kind == SyntaxKind::RightBrace {
            break;
        }
        index = statement_end + 1;
    }
}

pub(crate) fn declaration_colon_index(
    tokens: &[Token<'_>],
    start: usize,
    end: usize,
) -> Option<usize> {
    let colon_index = top_level_token_kind_index(tokens, start, end, SyntaxKind::Colon)?;
    let property_index = previous_non_trivia_token_index(tokens, colon_index, start)?;
    if !matches!(
        tokens[property_index].kind,
        SyntaxKind::Ident
            | SyntaxKind::CustomPropertyName
            | SyntaxKind::ScssVariable
            | SyntaxKind::LessVariable
            | SyntaxKind::LessPropertyVariableToken
    ) {
        return None;
    }
    let value_index = next_non_trivia_token_index_until(tokens, colon_index + 1, end)?;
    if matches!(
        tokens[value_index].kind,
        SyntaxKind::LeftBrace | SyntaxKind::LeftParen | SyntaxKind::LeftBracket
    ) {
        return None;
    }
    Some(colon_index)
}

fn collect_known_css_module_value_reference_facts(
    tokens: &[Token<'_>],
    start: usize,
    end: usize,
    local_value_names: &BTreeSet<String>,
    values: &mut Vec<ParsedCssModuleValueFact>,
    seen: &mut BTreeSet<(ParsedCssModuleValueFactKind, String, u32, u32)>,
) {
    let mut index = start;
    let mut paren_depth = 0usize;
    let mut bracket_depth = 0usize;
    while index < end {
        match tokens[index].kind {
            SyntaxKind::LeftParen => paren_depth += 1,
            SyntaxKind::RightParen => paren_depth = paren_depth.saturating_sub(1),
            SyntaxKind::LeftBracket => bracket_depth += 1,
            SyntaxKind::RightBracket => bracket_depth = bracket_depth.saturating_sub(1),
            _ => {}
        }
        if paren_depth == 0
            && bracket_depth == 0
            && css_module_value_reference_token_can_be_name(tokens, index)
            && local_value_names.contains(tokens[index].text)
        {
            push_css_module_value_fact(
                values,
                seen,
                ParsedCssModuleValueFactKind::Reference,
                tokens[index].text.to_string(),
                tokens[index].range,
            );
        }
        index += 1;
    }
}

fn push_css_module_value_fact(
    values: &mut Vec<ParsedCssModuleValueFact>,
    seen: &mut BTreeSet<(ParsedCssModuleValueFactKind, String, u32, u32)>,
    kind: ParsedCssModuleValueFactKind,
    name: String,
    range: TextRange,
) {
    if seen.insert((
        kind,
        name.clone(),
        u32::from(range.start()),
        u32::from(range.end()),
    )) {
        values.push(ParsedCssModuleValueFact { kind, name, range });
    }
}

fn css_module_value_name_token_can_define(token: Token<'_>) -> bool {
    matches!(
        token.kind,
        SyntaxKind::Ident | SyntaxKind::CustomPropertyName
    ) && !matches!(token.text, "as" | "from")
}

pub(crate) fn css_module_value_reference_token_can_be_name(
    tokens: &[Token<'_>],
    index: usize,
) -> bool {
    let token = tokens[index];
    if !matches!(
        token.kind,
        SyntaxKind::Ident | SyntaxKind::CustomPropertyName
    ) {
        return false;
    }
    if let Some(next_index) = next_non_trivia_token_index_until(tokens, index + 1, tokens.len())
        && tokens[next_index].kind == SyntaxKind::LeftParen
    {
        return false;
    }
    !css_module_value_literal_ident_is_not_reference(token.text)
}

fn css_module_value_literal_ident_is_not_reference(name: &str) -> bool {
    matches!(
        name.to_ascii_lowercase().as_str(),
        "initial"
            | "inherit"
            | "unset"
            | "revert"
            | "revert-layer"
            | "none"
            | "auto"
            | "normal"
            | "transparent"
            | "currentcolor"
            | "black"
            | "white"
            | "red"
            | "green"
            | "blue"
            | "yellow"
            | "magenta"
            | "cyan"
            | "solid"
            | "dashed"
            | "block"
            | "inline"
            | "flex"
            | "grid"
    )
}

pub(crate) fn css_module_value_source_name(token: Token<'_>) -> String {
    token
        .text
        .trim_matches(|character| character == '"' || character == '\'')
        .to_string()
}

fn css_module_value_source_looks_like_style_request(source: &str) -> bool {
    let lower = source.to_ascii_lowercase();
    (lower.starts_with('/') || lower.starts_with("./") || lower.starts_with("../"))
        && (lower.ends_with(".css")
            || lower.ends_with(".scss")
            || lower.ends_with(".sass")
            || lower.ends_with(".less"))
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParsedCssModuleComposesFact {
    pub kind: ParsedCssModuleComposesFactKind,
    pub name: String,
    pub range: TextRange,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum ParsedCssModuleComposesFactKind {
    Target,
    ImportSource,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParsedCssModuleComposesEdgeFact {
    pub kind: ParsedCssModuleComposesEdgeKind,
    pub owner_selector_names: Vec<String>,
    pub target_names: Vec<String>,
    pub import_source: Option<String>,
    pub range: TextRange,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum ParsedCssModuleComposesEdgeKind {
    Local,
    Global,
    External,
}

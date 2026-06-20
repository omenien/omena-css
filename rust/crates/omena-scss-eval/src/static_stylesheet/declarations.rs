use std::collections::BTreeMap;

use omena_parser::{LexedToken, ParsedVariableFact, ParsedVariableFactKind, StyleDialect, lex};
use omena_syntax::SyntaxKind;

use super::{
    model::{
        StaticLessMixinDeclaration, StaticScssFunctionDeclaration, StaticScssMixinDeclaration,
        StaticStylesheetPropertyDeclaration, StaticStylesheetScope,
        StaticStylesheetScopedVariableDeclaration, StaticStylesheetVariableDeclaration,
        StaticStylesheetVariableKind,
    },
    safety::{
        static_stylesheet_less_declaration_value_is_removal_safe,
        static_stylesheet_property_name_is_safe, static_stylesheet_property_value_is_removal_safe,
        static_stylesheet_scss_declaration_value_is_removal_safe,
    },
    static_stylesheet_position_is_inside_ranges, static_stylesheet_scope_for_position,
    tokens::{
        parser_text_size_to_usize, static_stylesheet_declaration_value_end_token,
        static_stylesheet_matching_token_index, static_stylesheet_next_token_kind_index,
        static_stylesheet_previous_token_is_body_start,
        static_stylesheet_scss_module_rule_semicolon, static_stylesheet_skip_trivia_tokens,
        static_stylesheet_token_end, static_stylesheet_token_is_trivia,
        static_stylesheet_token_start,
    },
    variable_references::static_stylesheet_variable_reference_is_named_argument_label,
};

fn static_stylesheet_previous_token_starts_declaration(
    tokens: &[LexedToken],
    index: usize,
) -> bool {
    tokens[..index]
        .iter()
        .rev()
        .find(|token| !static_stylesheet_token_is_trivia(token.kind))
        .is_some_and(|token| {
            matches!(
                token.kind,
                SyntaxKind::LeftBrace
                    | SyntaxKind::SassIndent
                    | SyntaxKind::Semicolon
                    | SyntaxKind::SassOptionalSemicolon
            )
        })
}

pub(super) fn extract_static_stylesheet_variable_declaration(
    source: &str,
    variable_start: usize,
    variable_end: usize,
    dialect: StyleDialect,
    variable_kind: StaticStylesheetVariableKind,
) -> Option<StaticStylesheetVariableDeclaration> {
    let after_name = source.get(variable_end..)?;
    let colon_offset = after_name.find(':')?;
    let value_start = variable_end + colon_offset + 1;
    let (value_end, span_end) =
        static_stylesheet_variable_value_and_span_end(source, value_start, dialect)?;
    let (value, is_default, is_global) = parse_static_stylesheet_declaration_value(
        source.get(value_start..value_end)?,
        variable_kind,
    );
    Some(StaticStylesheetVariableDeclaration {
        value,
        span_start: variable_start,
        span_end,
        removal_spans: vec![(variable_start, span_end)],
        is_default,
        is_global,
    })
}

fn static_stylesheet_variable_value_and_span_end(
    source: &str,
    value_start: usize,
    dialect: StyleDialect,
) -> Option<(usize, usize)> {
    let rest = source.get(value_start..)?;
    if dialect == StyleDialect::Sass {
        let value_end = rest
            .find('\n')
            .map(|offset| value_start + offset)
            .unwrap_or(source.len());
        let span_end = if value_end < source.len() {
            value_end + 1
        } else {
            value_end
        };
        return Some((value_end, span_end));
    }

    let terminator_offset = rest.find(';')?;
    let value_end = value_start + terminator_offset;
    Some((value_end, value_end + 1))
}

fn parse_static_stylesheet_declaration_value(
    value: &str,
    variable_kind: StaticStylesheetVariableKind,
) -> (String, bool, bool) {
    let mut value = value.trim();
    let mut is_default = false;
    let mut is_global = false;
    if variable_kind == StaticStylesheetVariableKind::Scss {
        loop {
            if let Some(before_flag) = value.strip_suffix("!default")
                && before_flag
                    .chars()
                    .next_back()
                    .is_some_and(char::is_whitespace)
            {
                is_default = true;
                value = before_flag.trim_end();
                continue;
            }
            if let Some(before_flag) = value.strip_suffix("!global")
                && before_flag
                    .chars()
                    .next_back()
                    .is_some_and(char::is_whitespace)
            {
                is_global = true;
                value = before_flag.trim_end();
                continue;
            }
            break;
        }
    }
    (value.to_string(), is_default, is_global)
}

fn merge_static_stylesheet_duplicate_declaration(
    previous: &mut StaticStylesheetVariableDeclaration,
    declaration: StaticStylesheetVariableDeclaration,
    variable_kind: StaticStylesheetVariableKind,
) -> Option<()> {
    match variable_kind {
        StaticStylesheetVariableKind::Less => {
            let mut removal_spans = previous.removal_spans.clone();
            removal_spans.extend(declaration.removal_spans.iter().copied());
            *previous = StaticStylesheetVariableDeclaration {
                removal_spans,
                ..declaration
            };
            Some(())
        }
        StaticStylesheetVariableKind::Scss if declaration.is_default => {
            previous
                .removal_spans
                .extend(declaration.removal_spans.iter().copied());
            Some(())
        }
        StaticStylesheetVariableKind::Scss if previous.is_default => {
            let mut removal_spans = previous.removal_spans.clone();
            removal_spans.extend(declaration.removal_spans.iter().copied());
            *previous = StaticStylesheetVariableDeclaration {
                removal_spans,
                ..declaration
            };
            Some(())
        }
        StaticStylesheetVariableKind::Scss => None,
    }
}

pub(super) fn collect_static_scss_variable_declarations(
    source: &str,
    dialect: StyleDialect,
    variable_facts: &[ParsedVariableFact],
    scopes: &[StaticStylesheetScope],
) -> Option<Vec<StaticStylesheetScopedVariableDeclaration>> {
    let mut declarations = Vec::new();
    let module_rule_ranges = collect_static_scss_module_rule_ranges(source, dialect);
    let function_declaration_ranges =
        collect_static_scss_function_declaration_ranges(source, dialect);
    let mixin_declaration_ranges = collect_static_scss_mixin_declaration_ranges(source, dialect);
    for fact in variable_facts {
        if fact.kind != ParsedVariableFactKind::ScssDeclaration {
            continue;
        }
        let start = parser_text_size_to_usize(fact.range.start().into());
        let end = parser_text_size_to_usize(fact.range.end().into());
        if static_stylesheet_variable_reference_is_named_argument_label(source, start, end) {
            continue;
        }
        if static_stylesheet_position_is_inside_ranges(start, &module_rule_ranges)
            || static_stylesheet_position_is_inside_ranges(start, &function_declaration_ranges)
            || static_stylesheet_position_is_inside_ranges(start, &mixin_declaration_ranges)
        {
            continue;
        }
        let scope_id = static_stylesheet_scope_for_position(scopes, start)?;
        let declaration = extract_static_stylesheet_variable_declaration(
            source,
            start,
            end,
            dialect,
            StaticStylesheetVariableKind::Scss,
        )?;
        if !static_stylesheet_scss_declaration_value_is_removal_safe(&declaration.value) {
            return None;
        }
        declarations.push(StaticStylesheetScopedVariableDeclaration {
            name: fact.name.clone(),
            scope_id: if declaration.is_global { 0 } else { scope_id },
            removal_spans: declaration.removal_spans.clone(),
            declaration,
        });
    }
    declarations.sort_by_key(|declaration| declaration.declaration.span_start);
    Some(declarations)
}

fn collect_static_scss_function_declaration_ranges(
    source: &str,
    dialect: StyleDialect,
) -> Vec<(usize, usize)> {
    collect_static_scss_block_at_rule_ranges(source, dialect, "@function")
}

fn collect_static_scss_mixin_declaration_ranges(
    source: &str,
    dialect: StyleDialect,
) -> Vec<(usize, usize)> {
    collect_static_scss_block_at_rule_ranges(source, dialect, "@mixin")
}

fn collect_static_scss_block_at_rule_ranges(
    source: &str,
    dialect: StyleDialect,
    at_rule_name: &str,
) -> Vec<(usize, usize)> {
    let lexed = lex(source, dialect);
    let tokens = lexed.tokens();
    let (body_open_kind, body_close_kind) = match dialect {
        StyleDialect::Sass => (SyntaxKind::SassIndent, SyntaxKind::SassDedent),
        _ => (SyntaxKind::LeftBrace, SyntaxKind::RightBrace),
    };
    let mut ranges = Vec::new();
    let mut index = 0usize;
    while index < tokens.len() {
        if tokens[index].kind != SyntaxKind::AtKeyword
            || !tokens[index].text.eq_ignore_ascii_case(at_rule_name)
        {
            index += 1;
            continue;
        }
        let Some(body_open_index) =
            static_stylesheet_next_token_kind_index(tokens, index + 1, body_open_kind)
        else {
            index += 1;
            continue;
        };
        let Some(body_close_index) = static_stylesheet_matching_token_index(
            tokens,
            body_open_index,
            body_open_kind,
            body_close_kind,
        ) else {
            index += 1;
            continue;
        };
        ranges.push((
            static_stylesheet_token_start(&tokens[index]),
            static_stylesheet_token_end(&tokens[body_close_index]),
        ));
        index = body_close_index + 1;
    }
    ranges
}

pub(super) fn static_scss_function_declaration_ranges_from_declarations(
    declarations: &[StaticScssFunctionDeclaration],
) -> Vec<(usize, usize)> {
    declarations
        .iter()
        .map(|declaration| (declaration.span_start, declaration.span_end))
        .collect()
}

pub(super) fn static_scss_mixin_declaration_ranges_from_declarations(
    declarations: &[StaticScssMixinDeclaration],
) -> Vec<(usize, usize)> {
    declarations
        .iter()
        .map(|declaration| (declaration.span_start, declaration.span_end))
        .collect()
}

pub(super) fn static_less_mixin_declaration_ranges_from_declarations(
    declarations: &[StaticLessMixinDeclaration],
) -> Vec<(usize, usize)> {
    declarations
        .iter()
        .map(|declaration| (declaration.span_start, declaration.span_end))
        .collect()
}

fn collect_static_scss_module_rule_ranges(
    source: &str,
    dialect: StyleDialect,
) -> Vec<(usize, usize)> {
    let lexed = lex(source, dialect);
    let tokens = lexed.tokens();
    let mut ranges = Vec::new();
    let mut depth = 0usize;
    let mut index = 0usize;

    while index < tokens.len() {
        match tokens[index].kind {
            SyntaxKind::LeftBrace | SyntaxKind::SassIndent => depth += 1,
            SyntaxKind::RightBrace | SyntaxKind::SassDedent => depth = depth.saturating_sub(1),
            SyntaxKind::AtKeyword
                if depth == 0
                    && matches!(
                        tokens[index].text.to_ascii_lowercase().as_str(),
                        "@use" | "@forward"
                    ) =>
            {
                let Some(end_index) = static_stylesheet_scss_module_rule_semicolon(tokens, index)
                else {
                    index += 1;
                    continue;
                };
                ranges.push((
                    static_stylesheet_token_start(&tokens[index]),
                    static_stylesheet_token_end(&tokens[end_index]),
                ));
                index = end_index + 1;
                continue;
            }
            _ => {}
        }
        index += 1;
    }

    ranges
}

pub(super) fn collect_static_less_variable_declarations(
    source: &str,
    variable_facts: &[ParsedVariableFact],
    scopes: &[StaticStylesheetScope],
    excluded_ranges: &[(usize, usize)],
) -> Option<BTreeMap<(usize, String), StaticStylesheetVariableDeclaration>> {
    let mut declarations = BTreeMap::<(usize, String), StaticStylesheetVariableDeclaration>::new();
    for fact in variable_facts {
        if fact.kind != ParsedVariableFactKind::LessDeclaration {
            continue;
        }
        let start = parser_text_size_to_usize(fact.range.start().into());
        let end = parser_text_size_to_usize(fact.range.end().into());
        if static_stylesheet_variable_reference_is_named_argument_label(source, start, end) {
            continue;
        }
        if static_stylesheet_position_is_inside_ranges(start, excluded_ranges) {
            continue;
        }
        let scope_id = static_stylesheet_scope_for_position(scopes, start)?;
        let declaration = extract_static_stylesheet_variable_declaration(
            source,
            start,
            end,
            StyleDialect::Less,
            StaticStylesheetVariableKind::Less,
        )?;
        if !static_stylesheet_less_declaration_value_is_removal_safe(&declaration.value) {
            return None;
        }
        let key = (scope_id, fact.name.clone());
        if let Some(previous) = declarations.get_mut(&key) {
            merge_static_stylesheet_duplicate_declaration(
                previous,
                declaration,
                StaticStylesheetVariableKind::Less,
            )?;
            continue;
        }
        declarations.insert(key, declaration);
    }
    Some(declarations)
}

pub(super) fn collect_static_less_property_declarations(
    source: &str,
    tokens: &[LexedToken],
    scopes: &[StaticStylesheetScope],
) -> Option<BTreeMap<(usize, String), StaticStylesheetPropertyDeclaration>> {
    collect_static_less_property_declarations_with_body_start(source, tokens, scopes, false)
}

pub(super) fn collect_static_less_body_property_declarations(
    source: &str,
    tokens: &[LexedToken],
    scopes: &[StaticStylesheetScope],
) -> Option<BTreeMap<(usize, String), StaticStylesheetPropertyDeclaration>> {
    collect_static_less_property_declarations_with_body_start(source, tokens, scopes, true)
}

fn collect_static_less_property_declarations_with_body_start(
    source: &str,
    tokens: &[LexedToken],
    scopes: &[StaticStylesheetScope],
    allow_body_start: bool,
) -> Option<BTreeMap<(usize, String), StaticStylesheetPropertyDeclaration>> {
    let mut declarations = BTreeMap::<(usize, String), StaticStylesheetPropertyDeclaration>::new();
    let mut index = 0usize;
    while index < tokens.len() {
        if !matches!(
            tokens[index].kind,
            SyntaxKind::Ident | SyntaxKind::CustomPropertyName
        ) || !static_stylesheet_property_name_is_safe(tokens[index].text.as_str())
            || !(static_stylesheet_previous_token_starts_declaration(tokens, index)
                || (allow_body_start
                    && static_stylesheet_previous_token_is_body_start(tokens, index)))
        {
            index += 1;
            continue;
        }

        let colon_index = static_stylesheet_skip_trivia_tokens(tokens, index + 1);
        if tokens
            .get(colon_index)
            .is_none_or(|token| token.kind != SyntaxKind::Colon)
        {
            index += 1;
            continue;
        }

        let value_start_index = colon_index + 1;
        let value_end_index =
            static_stylesheet_declaration_value_end_token(tokens, value_start_index)?;
        let value_start = static_stylesheet_token_end(&tokens[colon_index]);
        let value_end = static_stylesheet_token_start(&tokens[value_end_index]);
        let value = source.get(value_start..value_end)?.trim().to_string();
        if value.is_empty() || !static_stylesheet_property_value_is_removal_safe(&value) {
            return None;
        }

        let scope_id = static_stylesheet_scope_for_position(
            scopes,
            static_stylesheet_token_start(&tokens[index]),
        )?;
        declarations.insert(
            (scope_id, format!("${}", tokens[index].text)),
            StaticStylesheetPropertyDeclaration { value },
        );
        index = value_end_index + 1;
    }
    Some(declarations)
}

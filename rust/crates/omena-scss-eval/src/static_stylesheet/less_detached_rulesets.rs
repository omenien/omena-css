use omena_parser::LexedToken;
use omena_syntax::SyntaxKind;

use super::{
    less_mixins::static_less_mixin_call_context_is_plain,
    model::{
        StaticLessDetachedRulesetAccessor, StaticLessDetachedRulesetCall,
        StaticLessDetachedRulesetDeclaration, StaticStylesheetScope,
    },
    static_less_mixin_body_is_static_declaration_subset, static_less_variable_name_is_safe,
    static_stylesheet_matching_token_index, static_stylesheet_property_name_is_safe,
    static_stylesheet_scope_for_position, static_stylesheet_skip_trivia_tokens,
    static_stylesheet_token_end, static_stylesheet_token_start,
};

pub(super) fn collect_static_less_detached_ruleset_declarations(
    source: &str,
    tokens: &[LexedToken],
    scopes: &[StaticStylesheetScope],
) -> Option<Vec<StaticLessDetachedRulesetDeclaration>> {
    let mut declarations = Vec::new();
    let mut index = 0usize;
    while index < tokens.len() {
        let token = tokens.get(index)?;
        if token.kind != SyntaxKind::LessVariable {
            index += 1;
            continue;
        }
        if !static_less_variable_name_is_safe(token.text.as_str()) {
            return None;
        }
        let colon_index = static_stylesheet_skip_trivia_tokens(tokens, index + 1);
        if tokens
            .get(colon_index)
            .is_none_or(|candidate| candidate.kind != SyntaxKind::Colon)
        {
            index += 1;
            continue;
        }
        let body_open_index = static_stylesheet_skip_trivia_tokens(tokens, colon_index + 1);
        if tokens
            .get(body_open_index)
            .is_none_or(|candidate| candidate.kind != SyntaxKind::LeftBrace)
        {
            index += 1;
            continue;
        }
        let body_close_index = static_stylesheet_matching_token_index(
            tokens,
            body_open_index,
            SyntaxKind::LeftBrace,
            SyntaxKind::RightBrace,
        )?;
        let semicolon_index = static_stylesheet_skip_trivia_tokens(tokens, body_close_index + 1);
        if tokens
            .get(semicolon_index)
            .is_none_or(|candidate| candidate.kind != SyntaxKind::Semicolon)
        {
            return None;
        }
        let body_start = static_stylesheet_token_end(&tokens[body_open_index]);
        let body_end = static_stylesheet_token_start(&tokens[body_close_index]);
        let body = source.get(body_start..body_end)?;
        if !static_less_mixin_body_is_static_declaration_subset(body) {
            return None;
        }
        let span_start = static_stylesheet_token_start(token);
        declarations.push(StaticLessDetachedRulesetDeclaration {
            name: token.text.clone(),
            scope_id: static_stylesheet_scope_for_position(scopes, span_start)?,
            span_start,
            span_end: static_stylesheet_token_end(&tokens[semicolon_index]),
            body_start,
            body_end,
        });
        index = semicolon_index + 1;
    }
    Some(declarations)
}

pub(super) fn collect_static_less_detached_ruleset_calls(
    source: &str,
    tokens: &[LexedToken],
) -> Option<Vec<StaticLessDetachedRulesetCall>> {
    let mut calls = Vec::new();
    let mut index = 0usize;
    while index < tokens.len() {
        let Some((call, semicolon_index)) =
            static_less_detached_ruleset_call_at(source, tokens, index)
        else {
            index += 1;
            continue;
        };
        calls.push(call);
        index = semicolon_index + 1;
    }
    calls.sort_by_key(|call| (call.start, call.end));
    Some(calls)
}

pub(super) fn collect_static_less_detached_ruleset_accessors(
    source: &str,
    tokens: &[LexedToken],
) -> Option<Vec<StaticLessDetachedRulesetAccessor>> {
    let mut accessors = Vec::new();
    let mut index = 0usize;
    while index < tokens.len() {
        let token = &tokens[index];
        if token.kind != SyntaxKind::LessVariable
            || !static_less_variable_name_is_safe(token.text.as_str())
        {
            index += 1;
            continue;
        }
        let bracket_open_index = static_stylesheet_skip_trivia_tokens(tokens, index + 1);
        if tokens
            .get(bracket_open_index)
            .is_none_or(|candidate| candidate.kind != SyntaxKind::LeftBracket)
        {
            index += 1;
            continue;
        }
        let bracket_close_index = static_stylesheet_matching_token_index(
            tokens,
            bracket_open_index,
            SyntaxKind::LeftBracket,
            SyntaxKind::RightBracket,
        )?;
        let member = static_less_detached_ruleset_accessor_member(source.get(
            static_stylesheet_token_end(&tokens[bracket_open_index])
                ..static_stylesheet_token_start(&tokens[bracket_close_index]),
        )?)?;
        accessors.push(StaticLessDetachedRulesetAccessor {
            name: token.text.clone(),
            member,
            start: static_stylesheet_token_start(token),
            end: static_stylesheet_token_end(&tokens[bracket_close_index]),
        });
        index = bracket_close_index + 1;
    }
    accessors.sort_by_key(|accessor| (accessor.start, accessor.end));
    Some(accessors)
}

pub(super) fn find_static_less_detached_ruleset_declaration<'a>(
    name: &str,
    mut scope_id: usize,
    scopes: &[StaticStylesheetScope],
    declarations: &'a [StaticLessDetachedRulesetDeclaration],
) -> Option<&'a StaticLessDetachedRulesetDeclaration> {
    loop {
        if let Some(declaration) = declarations
            .iter()
            .rev()
            .find(|declaration| declaration.name == name && declaration.scope_id == scope_id)
        {
            return Some(declaration);
        }
        scope_id = scopes.get(scope_id)?.parent_id?;
    }
}

pub(super) fn static_less_detached_ruleset_ranges_from_declarations(
    declarations: &[StaticLessDetachedRulesetDeclaration],
) -> Vec<(usize, usize)> {
    declarations
        .iter()
        .map(|declaration| (declaration.span_start, declaration.span_end))
        .collect()
}

pub(super) fn static_less_detached_ruleset_ranges_from_calls(
    calls: &[StaticLessDetachedRulesetCall],
) -> Vec<(usize, usize)> {
    calls.iter().map(|call| (call.start, call.end)).collect()
}

pub(super) fn static_less_detached_ruleset_ranges_from_accessors(
    accessors: &[StaticLessDetachedRulesetAccessor],
) -> Vec<(usize, usize)> {
    accessors
        .iter()
        .map(|accessor| (accessor.start, accessor.end))
        .collect()
}

fn static_less_detached_ruleset_call_at(
    source: &str,
    tokens: &[LexedToken],
    index: usize,
) -> Option<(StaticLessDetachedRulesetCall, usize)> {
    let token = tokens.get(index)?;
    if token.kind != SyntaxKind::LessVariable
        || !static_less_variable_name_is_safe(token.text.as_str())
        || !static_less_mixin_call_context_is_plain(tokens, index)
    {
        return None;
    }
    let open_index = static_stylesheet_skip_trivia_tokens(tokens, index + 1);
    if tokens
        .get(open_index)
        .is_none_or(|candidate| candidate.kind != SyntaxKind::LeftParen)
    {
        return None;
    }
    let close_index = static_stylesheet_matching_token_index(
        tokens,
        open_index,
        SyntaxKind::LeftParen,
        SyntaxKind::RightParen,
    )?;
    let argument_text = source.get(
        static_stylesheet_token_end(&tokens[open_index])
            ..static_stylesheet_token_start(&tokens[close_index]),
    )?;
    if !argument_text.trim().is_empty() {
        return None;
    }
    let semicolon_index = static_stylesheet_skip_trivia_tokens(tokens, close_index + 1);
    if tokens
        .get(semicolon_index)
        .is_none_or(|candidate| candidate.kind != SyntaxKind::Semicolon)
    {
        return None;
    }
    Some((
        StaticLessDetachedRulesetCall {
            name: token.text.clone(),
            start: static_stylesheet_token_start(token),
            end: static_stylesheet_token_end(&tokens[semicolon_index]),
        },
        semicolon_index,
    ))
}

fn static_less_detached_ruleset_accessor_member(member: &str) -> Option<String> {
    let member = member.trim();
    if static_less_variable_name_is_safe(member) || static_stylesheet_property_name_is_safe(member)
    {
        return Some(member.to_string());
    }
    None
}

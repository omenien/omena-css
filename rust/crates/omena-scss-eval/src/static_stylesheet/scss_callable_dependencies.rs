use std::collections::BTreeSet;

use omena_parser::{StyleDialect, lex};
use omena_syntax::SyntaxKind;

use super::{
    StaticScssFunctionDeclaration, canonical_static_scss_function_name,
    static_stylesheet_skip_trivia_tokens,
};

pub(super) fn extend_static_scss_used_function_dependencies(
    used_declaration_names: &mut BTreeSet<String>,
    declarations: &[StaticScssFunctionDeclaration],
) {
    let declaration_names = declarations
        .iter()
        .map(|declaration| canonical_static_scss_function_name(declaration.name.as_str()))
        .collect::<BTreeSet<_>>();
    let mut changed = true;
    while changed {
        changed = false;
        for declaration in declarations {
            let declaration_name = canonical_static_scss_function_name(declaration.name.as_str());
            if !used_declaration_names.contains(&declaration_name) {
                continue;
            }
            for dependency_name in
                static_scss_function_dependency_names(declaration, &declaration_names)
            {
                if used_declaration_names.insert(dependency_name) {
                    changed = true;
                }
            }
        }
    }
}

fn static_scss_function_dependency_names(
    declaration: &StaticScssFunctionDeclaration,
    declaration_names: &BTreeSet<String>,
) -> Vec<String> {
    declaration
        .return_clauses
        .iter()
        .flat_map(|clause| {
            std::iter::once(clause.value.as_str()).chain(clause.condition.as_deref())
        })
        .chain(
            declaration
                .local_variables
                .iter()
                .map(|local_variable| local_variable.value.as_str()),
        )
        .flat_map(|value| static_scss_callable_names_in_value(value, declaration_names))
        .collect()
}

fn static_scss_callable_names_in_value(
    value: &str,
    declaration_names: &BTreeSet<String>,
) -> Vec<String> {
    let lexed = lex(value, StyleDialect::Scss);
    let tokens = lexed.tokens();
    tokens
        .iter()
        .enumerate()
        .filter_map(|(index, token)| {
            if token.kind != SyntaxKind::Ident || token.text.eq_ignore_ascii_case("if") {
                return None;
            }
            let canonical_name = canonical_static_scss_function_name(token.text.as_str());
            (declaration_names.contains(&canonical_name)
                && tokens
                    .get(static_stylesheet_skip_trivia_tokens(tokens, index + 1))
                    .is_some_and(|candidate| candidate.kind == SyntaxKind::LeftParen))
            .then_some(canonical_name)
        })
        .collect()
}

pub(super) fn static_scss_function_value_contains_any_callable(value: &str) -> bool {
    let lexed = lex(value, StyleDialect::Scss);
    let tokens = lexed.tokens();
    tokens.iter().enumerate().any(|(index, token)| {
        token.kind == SyntaxKind::Ident
            && !token.text.eq_ignore_ascii_case("if")
            && tokens
                .get(static_stylesheet_skip_trivia_tokens(tokens, index + 1))
                .is_some_and(|candidate| candidate.kind == SyntaxKind::LeftParen)
    })
}

pub(super) fn static_scss_function_value_contains_callable_to(value: &str, name: &str) -> bool {
    let canonical_name = canonical_static_scss_function_name(name);
    let lexed = lex(value, StyleDialect::Scss);
    let tokens = lexed.tokens();
    tokens.iter().enumerate().any(|(index, token)| {
        token.kind == SyntaxKind::Ident
            && canonical_static_scss_function_name(token.text.as_str()) == canonical_name
            && tokens
                .get(static_stylesheet_skip_trivia_tokens(tokens, index + 1))
                .is_some_and(|candidate| candidate.kind == SyntaxKind::LeftParen)
    })
}

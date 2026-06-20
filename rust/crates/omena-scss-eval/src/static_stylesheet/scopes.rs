use std::collections::BTreeMap;

use super::model::{
    StaticStylesheetScope, StaticStylesheetScopedVariableDeclaration,
    StaticStylesheetVariableDeclaration,
};

pub(super) fn collect_static_stylesheet_scopes(source: &str) -> Option<Vec<StaticStylesheetScope>> {
    let mut scopes = vec![StaticStylesheetScope {
        parent_id: None,
        body_start: 0,
        end: source.len(),
    }];
    let mut stack = vec![0usize];
    let mut index = 0usize;
    let mut quote: Option<char> = None;
    let bytes = source.as_bytes();

    while index < source.len() {
        let ch = source[index..].chars().next()?;
        if let Some(quote_ch) = quote {
            index += ch.len_utf8();
            if ch == '\\' {
                if let Some(escaped) = source[index..].chars().next() {
                    index += escaped.len_utf8();
                }
            } else if ch == quote_ch {
                quote = None;
            }
            continue;
        }

        if matches!(ch, '"' | '\'') {
            quote = Some(ch);
            index += ch.len_utf8();
            continue;
        }
        if bytes.get(index..index + 2) == Some(b"/*") {
            let end = source.get(index + 2..)?.find("*/")?;
            index += end + 4;
            continue;
        }
        if bytes.get(index..index + 2) == Some(b"//") {
            let line_end = source
                .get(index + 2..)?
                .find('\n')
                .map(|offset| index + 2 + offset)
                .unwrap_or(source.len());
            index = line_end;
            continue;
        }

        match ch {
            '{' => {
                let parent_id = *stack.last()?;
                let scope_id = scopes.len();
                scopes.push(StaticStylesheetScope {
                    parent_id: Some(parent_id),
                    body_start: index + ch.len_utf8(),
                    end: source.len(),
                });
                stack.push(scope_id);
            }
            '}' => {
                let scope_id = stack.pop()?;
                if scope_id == 0 {
                    return None;
                }
                scopes.get_mut(scope_id)?.end = index;
            }
            _ => {}
        }
        index += ch.len_utf8();
    }

    (stack.len() == 1).then_some(scopes)
}

pub(super) fn static_stylesheet_scope_for_position(
    scopes: &[StaticStylesheetScope],
    position: usize,
) -> Option<usize> {
    scopes
        .iter()
        .enumerate()
        .rev()
        .find_map(|(scope_id, scope)| {
            (position >= scope.body_start && position < scope.end).then_some(scope_id)
        })
}

pub(super) fn static_stylesheet_position_is_inside_scoped_declaration(
    declarations: &BTreeMap<(usize, String), StaticStylesheetVariableDeclaration>,
    position: usize,
) -> bool {
    declarations.values().any(|declaration| {
        declaration
            .removal_spans
            .iter()
            .any(|(start, end)| position >= *start && position < *end)
    })
}

pub(super) fn static_stylesheet_position_is_inside_scss_declaration(
    declarations: &[StaticStylesheetScopedVariableDeclaration],
    position: usize,
) -> bool {
    declarations.iter().any(|declaration| {
        declaration
            .removal_spans
            .iter()
            .any(|(start, end)| position >= *start && position < *end)
    })
}

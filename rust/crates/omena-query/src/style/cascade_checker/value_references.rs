use std::collections::BTreeSet;

use omena_query_core::split_top_level_value_arguments;

pub(super) fn collect_query_var_references_in_value(value: &str) -> Vec<String> {
    let mut references = BTreeSet::new();
    let mut index = 0usize;
    let mut quote: Option<char> = None;
    while index < value.len() {
        let Some(ch) = value[index..].chars().next() else {
            break;
        };
        if let Some(quote_ch) = quote {
            index += ch.len_utf8();
            if ch == '\\' {
                if let Some(escaped) = value[index..].chars().next() {
                    index += escaped.len_utf8();
                }
            } else if ch == quote_ch {
                quote = None;
            }
            continue;
        }

        match ch {
            '"' | '\'' => {
                quote = Some(ch);
                index += ch.len_utf8();
            }
            _ if function_name_starts_at(value, index, "var") => {
                let open_index = index + "var".len();
                let Some(close_index) = matching_paren_end(value, open_index) else {
                    index += ch.len_utf8();
                    continue;
                };
                collect_var_references_from_arguments(
                    &value[open_index + 1..close_index],
                    &mut references,
                );
                index = close_index + 1;
            }
            _ => index += ch.len_utf8(),
        }
    }
    references.into_iter().collect()
}

fn collect_var_references_from_arguments(arguments: &str, references: &mut BTreeSet<String>) {
    let parts = split_top_level_value_arguments(arguments, 0)
        .map(|segments| segments.into_iter().map(|segment| segment.text).collect())
        .unwrap_or_else(|| vec![arguments]);
    let Some(first_argument) = parts.first().map(|part| part.trim()) else {
        return;
    };
    if first_argument.starts_with("--") {
        references.insert(first_argument.to_string());
    }
    for fallback in parts.iter().skip(1) {
        for reference in collect_query_var_references_in_value(fallback) {
            references.insert(reference);
        }
    }
}

fn function_name_starts_at(value: &str, index: usize, function_name: &str) -> bool {
    value
        .get(index..index + function_name.len())
        .is_some_and(|name| name.eq_ignore_ascii_case(function_name))
        && value[index + function_name.len()..].starts_with('(')
}

fn matching_paren_end(source: &str, open_index: usize) -> Option<usize> {
    if source.as_bytes().get(open_index).copied()? != b'(' {
        return None;
    }
    let mut index = open_index + 1;
    let mut depth = 1usize;
    let mut quote: Option<char> = None;
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
        match ch {
            '"' | '\'' => quote = Some(ch),
            '(' => depth += 1,
            ')' => {
                depth -= 1;
                if depth == 0 {
                    return Some(index);
                }
            }
            _ => {}
        }
        index += ch.len_utf8();
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn nested_var_fallbacks_remain_value_facts_not_declaration_scanning() {
        assert_eq!(
            collect_query_var_references_in_value("linear-gradient(var(--a), var(--b, var(--c)))"),
            ["--a", "--b", "--c"]
        );
        assert!(collect_query_var_references_in_value("'var(--quoted)'").is_empty());
    }
}

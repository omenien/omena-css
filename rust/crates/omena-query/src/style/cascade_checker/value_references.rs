use std::collections::BTreeMap;

use omena_query_core::split_top_level_value_arguments;
use omena_syntax::ident::{AuthoredPropertyTextV0, CanonicalCustomPropertyNameV0, PropertyNameV0};

#[derive(Debug, Clone)]
pub(super) struct QueryCustomPropertyReferenceV0 {
    pub(super) authored: AuthoredPropertyTextV0,
    pub(super) key: CanonicalCustomPropertyNameV0,
}

impl PartialEq for QueryCustomPropertyReferenceV0 {
    fn eq(&self, other: &Self) -> bool {
        self.key == other.key
    }
}

impl Eq for QueryCustomPropertyReferenceV0 {}

pub(super) fn collect_query_var_references_in_value(value: &str) -> Vec<AuthoredPropertyTextV0> {
    collect_query_var_reference_facts_in_value(value)
        .into_iter()
        .map(|reference| reference.authored)
        .collect()
}

pub(super) fn collect_query_var_reference_facts_in_value(
    value: &str,
) -> Vec<QueryCustomPropertyReferenceV0> {
    let mut references = BTreeMap::new();
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
    references
        .into_iter()
        .map(|(key, authored)| QueryCustomPropertyReferenceV0 { authored, key })
        .collect()
}

fn collect_var_references_from_arguments(
    arguments: &str,
    references: &mut BTreeMap<CanonicalCustomPropertyNameV0, AuthoredPropertyTextV0>,
) {
    let parts = split_top_level_value_arguments(arguments, 0)
        .map(|segments| segments.into_iter().map(|segment| segment.text).collect())
        .unwrap_or_else(|| vec![arguments]);
    let Some(first_argument) = parts.first().map(|part| part.trim()) else {
        return;
    };
    let property = PropertyNameV0::from_authored(first_argument);
    if let Some(property_key) = property.as_custom_key() {
        references
            .entry(property_key)
            .or_insert_with(|| property.authored_text());
    }
    for fallback in parts.iter().skip(1) {
        for reference in collect_query_var_reference_facts_in_value(fallback) {
            references
                .entry(reference.key)
                .or_insert(reference.authored);
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

    fn authored_text(property: &AuthoredPropertyTextV0) -> String {
        let mut text = String::new();
        let _ = omena_syntax::ident::render_authored(property, &mut text);
        text
    }

    #[test]
    fn nested_var_fallbacks_remain_value_facts_not_declaration_scanning() {
        assert_eq!(
            collect_query_var_references_in_value("linear-gradient(var(--a), var(--b, var(--c)))")
                .iter()
                .map(authored_text)
                .collect::<Vec<_>>(),
            ["--a", "--b", "--c"]
        );
        assert!(collect_query_var_references_in_value("'var(--quoted)'").is_empty());
    }

    #[test]
    fn var_reference_facts_share_custom_property_escape_identity() {
        let references = collect_query_var_reference_facts_in_value(
            r"linear-gradient(var(--foo), var(--f\6f o))",
        );

        assert_eq!(references.len(), 1);
        assert_eq!(authored_text(&references[0].authored), "--foo");
        assert_eq!(references[0].key.as_str(), "--foo");
    }

    #[test]
    fn var_reference_kind_is_classified_after_escape_decoding() {
        let references = collect_query_var_reference_facts_in_value(r"var(\2d\2d FOO)");

        assert_eq!(references.len(), 1);
        assert_eq!(authored_text(&references[0].authored), r"\2d\2d FOO");
        assert_eq!(references[0].key.as_str(), "--FOO");
    }

    #[test]
    fn query_custom_property_reference_identity_uses_custom_property_keys() {
        let plain = collect_query_var_reference_facts_in_value("var(--foo)");
        let escaped = collect_query_var_reference_facts_in_value(r"var(--f\6f o)");
        let different_case = collect_query_var_reference_facts_in_value("var(--FOO)");

        assert_eq!(plain[0], escaped[0]);
        assert_ne!(plain[0], different_case[0]);
    }
}

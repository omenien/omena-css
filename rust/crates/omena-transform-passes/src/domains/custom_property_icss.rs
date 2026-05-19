use omena_parser::LexedToken;

use crate::{
    domains::custom_property::collect_custom_property_references_in_value,
    helpers::{
        blocks::rule_block_token_indexes, declarations::collect_simple_declarations_in_block,
        rules::collect_declaration_ordinary_rule_slices,
    },
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct CustomPropertyIcssExportRule {
    pub(crate) start: usize,
    pub(crate) end: usize,
    pub(crate) declarations: Vec<CustomPropertyIcssExportDeclaration>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct CustomPropertyIcssExportDeclaration {
    pub(crate) export_name: String,
    pub(crate) value: String,
    pub(crate) start: usize,
    pub(crate) end: usize,
}

pub(crate) fn collect_static_custom_property_icss_export_rules(
    source: &str,
    tokens: &[LexedToken],
) -> Vec<CustomPropertyIcssExportRule> {
    collect_declaration_ordinary_rule_slices(source, tokens)
        .into_iter()
        .filter(|rule| rule.selector.trim().eq_ignore_ascii_case(":export"))
        .filter_map(|rule| {
            let (block_start_index, block_end_index) =
                rule_block_token_indexes(tokens, rule.block_start, rule.block_end)?;
            let declarations =
                collect_simple_declarations_in_block(tokens, block_start_index, block_end_index)
                    .into_iter()
                    .filter(|declaration| {
                        !collect_custom_property_references_in_value(&declaration.value).is_empty()
                    })
                    .map(|declaration| CustomPropertyIcssExportDeclaration {
                        export_name: declaration.property,
                        value: declaration.value,
                        start: declaration.start,
                        end: declaration.end,
                    })
                    .collect::<Vec<_>>();
            (!declarations.is_empty()).then_some(CustomPropertyIcssExportRule {
                start: rule.start,
                end: rule.end,
                declarations,
            })
        })
        .collect()
}

pub(crate) fn custom_property_icss_export_is_reachable(
    export_name: &str,
    roots: &[String],
) -> bool {
    roots.iter().any(|root| {
        root == export_name
            || custom_property_icss_export_alias(root)
                == custom_property_icss_export_alias(export_name)
    })
}

fn custom_property_icss_export_alias(name: &str) -> &str {
    name.trim().strip_prefix("--").unwrap_or(name.trim())
}

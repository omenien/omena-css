use omena_parser::LexedToken;
use omena_syntax::ident::{AuthoredPropertyTextV0, PropertyNameV0, render_authored};

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
                    .map(|declaration| {
                        let mut export_name = String::new();
                        let _ = render_authored(&declaration.property, &mut export_name);
                        CustomPropertyIcssExportDeclaration {
                            export_name,
                            value: declaration.value,
                            start: declaration.start,
                            end: declaration.end,
                        }
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
    roots: &[AuthoredPropertyTextV0],
) -> bool {
    let export_key = PropertyNameV0::canonical_custom_key(export_name);
    roots.iter().any(|root| {
        let root_key = root.to_custom_key();
        root_key == export_key
            || custom_property_icss_export_alias(root_key.as_str())
                == custom_property_icss_export_alias(export_key.as_str())
    })
}

fn custom_property_icss_export_alias(name: &str) -> &str {
    name.trim().strip_prefix("--").unwrap_or(name.trim())
}

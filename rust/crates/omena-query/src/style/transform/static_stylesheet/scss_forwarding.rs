use super::super::parser_facade::lex_omena_query_omena_parser_style_source;
use super::super::{
    apply_transform_source_replacements, transform_token_end, transform_token_start,
};
use super::scss_module_rules;
use crate::OmenaParserStyleDialect;
use omena_syntax::SyntaxKind;
use std::collections::{BTreeMap, BTreeSet};

#[derive(Debug, Clone)]
pub(super) struct StaticScssModuleForwardEvaluation {
    pub(super) source: String,
    pub(super) forward_rule_ordinal: usize,
    pub(super) module_identity_key: String,
    pub(super) module_output_css: String,
    pub(super) variable_exports: BTreeMap<String, String>,
    pub(super) configurable_variable_names: BTreeSet<String>,
}

pub(super) fn inline_static_scss_forward_rules(
    source: &str,
    dialect: OmenaParserStyleDialect,
    forward_evaluations: &[StaticScssModuleForwardEvaluation],
    emitted_module_identity_keys: &mut BTreeSet<String>,
) -> (String, usize) {
    let lexed = lex_omena_query_omena_parser_style_source(source, dialect);
    let tokens = lexed.tokens();
    let mut replacements = Vec::new();
    let mut depth = 0usize;
    let mut forward_rule_ordinal = 0usize;
    let mut index = 0usize;

    while index < tokens.len() {
        match tokens[index].kind {
            SyntaxKind::LeftBrace => depth += 1,
            SyntaxKind::RightBrace => depth = depth.saturating_sub(1),
            SyntaxKind::AtKeyword
                if depth == 0 && tokens[index].text.eq_ignore_ascii_case("@forward") =>
            {
                let Some(end_index) =
                    scss_module_rules::static_scss_use_rule_semicolon(tokens, index)
                else {
                    index += 1;
                    continue;
                };
                let start = transform_token_start(&tokens[index]);
                let end = transform_token_end(&tokens[end_index]);
                if let Some(source_name) = scss_module_rules::static_scss_module_rule_source_name(
                    tokens,
                    index + 1,
                    end_index,
                ) {
                    let matching_forward = forward_evaluations.iter().find(|forward| {
                        forward.forward_rule_ordinal == forward_rule_ordinal
                            && forward.source == source_name
                    });
                    forward_rule_ordinal += 1;
                    if let Some(forward) = matching_forward {
                        let replacement = if emitted_module_identity_keys
                            .insert(forward.module_identity_key.clone())
                        {
                            forward.module_output_css.clone()
                        } else {
                            String::new()
                        };
                        replacements.push((start, end, replacement));
                    }
                }
                index = end_index + 1;
                continue;
            }
            _ => {}
        }
        index += 1;
    }

    apply_transform_source_replacements(source, replacements)
}

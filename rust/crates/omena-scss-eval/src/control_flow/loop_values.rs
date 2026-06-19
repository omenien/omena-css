use omena_parser::{StyleDialect, lex};
use omena_syntax::SyntaxKind;

use super::{
    model::OmenaScssEvalControlFlowBlockV0, tokens::next_non_trivia_token_index,
    variables::canonical_scss_variable_name,
};

pub(super) fn while_loop_body_assignment_names(
    source: &str,
    block: &OmenaScssEvalControlFlowBlockV0,
) -> Vec<String> {
    let Some(body) = control_flow_block_body_text(source, block) else {
        return Vec::new();
    };
    let lexed = lex(body, StyleDialect::Scss);
    let tokens = lexed.tokens();
    let mut names: Vec<String> = Vec::new();
    for (index, token) in tokens.iter().enumerate() {
        if token.kind != SyntaxKind::ScssVariable {
            continue;
        }
        let Some(colon_index) = next_non_trivia_token_index(tokens, index + 1) else {
            continue;
        };
        if tokens[colon_index].kind != SyntaxKind::Colon {
            continue;
        }
        let name = token.text.to_string();
        if !names.iter().any(|existing| {
            canonical_scss_variable_name(existing.as_str())
                == canonical_scss_variable_name(name.as_str())
        }) {
            names.push(name);
        }
    }
    names
}

fn control_flow_block_body_text<'a>(
    source: &'a str,
    block: &OmenaScssEvalControlFlowBlockV0,
) -> Option<&'a str> {
    let block_text = source.get(block.source_span_start..block.source_span_end)?;
    let open = block_text.find('{')?;
    let close = block_text.rfind('}')?;
    (open < close)
        .then(|| block_text.get(open + '{'.len_utf8()..close))
        .flatten()
}

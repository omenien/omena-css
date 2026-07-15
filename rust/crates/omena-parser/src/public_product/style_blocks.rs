use std::collections::BTreeMap;

use cstree::syntax::SyntaxNode;
use omena_syntax::SyntaxKind;

use crate::ParseResult;

use super::{
    ParserIndexAtRuleContextV0, SelectorBranch, SourceLineIndex, StyleBlock, WrapperContext,
    byte_span_for_range, classify_nested_safety, parser_range_for_byte_span,
    resolve_selector_header_text,
};

pub(super) fn collect_style_blocks_from_cst(
    source: &str,
    line_index: &SourceLineIndex,
    parsed: &ParseResult,
) -> Vec<StyleBlock> {
    let syntax = parsed.syntax();
    let mut blocks = Vec::new();
    let mut branches_by_node = BTreeMap::<(usize, usize), Vec<SelectorBranch>>::new();

    for node in syntax.descendants() {
        if is_style_wrapper_node(node.kind()) {
            if has_excluded_style_owner_ancestor(node) {
                continue;
            }
            if let Some(bounds) = block_bounds(parsed, node) {
                let wrapper = wrapper_context_for_node(source, line_index, parsed, node);
                let header = source
                    .get(bounds.header_start..bounds.open)
                    .unwrap_or_default()
                    .trim();
                blocks.push(StyleBlock {
                    names: Vec::new(),
                    context_text: None,
                    start: bounds.body_start,
                    end: bounds.close,
                    rule_start: bounds.header_start,
                    rule_end: bounds.rule_end,
                    body_start: bounds.body_start,
                    body_end: bounds.close,
                    header_text: Some(header.to_string()),
                    under_media: wrapper.under_media,
                    under_supports: wrapper.under_supports,
                    under_layer: wrapper.under_layer,
                    wrapper_at_rules: wrapper.wrapper_at_rules,
                });
            }
            continue;
        }

        if !matches!(node.kind(), SyntaxKind::Rule | SyntaxKind::NestRule)
            || has_excluded_style_owner_ancestor(node)
        {
            continue;
        }
        let Some(bounds) = block_bounds(parsed, node) else {
            continue;
        };
        let parent_branches = nearest_parent_branches(node, &branches_by_node);
        let selector_start =
            selector_header_start(node, bounds.open).unwrap_or(bounds.header_start);
        let (selector_header, branches) =
            resolve_header_from_cst_span(source, selector_start, bounds.open, &parent_branches);
        let full_header = source
            .get(bounds.header_start..bounds.open)
            .unwrap_or_default()
            .trim();
        let header = if node.kind() == SyntaxKind::NestRule {
            selector_header.as_str()
        } else {
            full_header
        };
        branches_by_node.insert(node_key(node), branches.clone());

        let wrapper = wrapper_context_for_node(source, line_index, parsed, node);
        push_style_block_from_cst(
            source,
            header,
            bounds,
            &branches,
            &parent_branches,
            parent_branches.len() > 1,
            wrapper,
            &mut blocks,
        );
    }

    blocks
}

#[derive(Debug, Clone, Copy)]
struct CstBlockBounds {
    header_start: usize,
    open: usize,
    body_start: usize,
    close: usize,
    rule_end: usize,
}

fn block_bounds(parsed: &ParseResult, node: &SyntaxNode<SyntaxKind>) -> Option<CstBlockBounds> {
    let open = node
        .descendants_with_tokens()
        .filter_map(|element| element.into_token())
        .find(|token| token.kind() == SyntaxKind::LeftBrace)
        .map(|token| byte_span_for_range(token.text_range()))?;
    let close = node
        .descendants_with_tokens()
        .filter_map(|element| element.into_token())
        .filter(|token| token.kind() == SyntaxKind::RightBrace)
        .last()
        .map(|token| byte_span_for_range(token.text_range()))?;
    (open.end <= close.start).then(|| CstBlockBounds {
        header_start: header_start_from_tokens(parsed, node),
        open: open.start,
        body_start: open.end,
        close: close.start,
        rule_end: close.end,
    })
}

fn header_start_from_tokens(parsed: &ParseResult, node: &SyntaxNode<SyntaxKind>) -> usize {
    let node_start = byte_span_for_range(node.text_range()).start;
    parsed
        .syntax_token_views()
        .iter()
        .rev()
        .find(|token| {
            byte_span_for_range(token.range).end <= node_start
                && matches!(
                    token.kind,
                    SyntaxKind::LeftBrace
                        | SyntaxKind::RightBrace
                        | SyntaxKind::Semicolon
                        | SyntaxKind::SassOptionalSemicolon
                )
        })
        .map_or(0, |token| byte_span_for_range(token.range).end)
}

fn selector_header_start(node: &SyntaxNode<SyntaxKind>, open: usize) -> Option<usize> {
    let node_start = byte_span_for_range(node.text_range()).start;
    if node.kind() != SyntaxKind::NestRule {
        return Some(node_start);
    }
    node.descendants_with_tokens()
        .filter_map(|element| element.into_token())
        .find(|token| {
            token.kind() == SyntaxKind::AtKeyword
                && byte_span_for_range(token.text_range()).start < open
        })
        .map(|token| byte_span_for_range(token.text_range()).end)
}

fn resolve_header_from_cst_span(
    source: &str,
    start: usize,
    end: usize,
    parent_branches: &[SelectorBranch],
) -> (String, Vec<SelectorBranch>) {
    let raw = source.get(start..end).unwrap_or_default();
    let trimmed = raw.trim();
    let leading = raw.len().saturating_sub(raw.trim_start().len());
    let absolute_start = start + leading;
    let mut branches = resolve_selector_header_text(
        source.get(absolute_start..).unwrap_or_default(),
        trimmed,
        parent_branches,
    );
    for branch in &mut branches {
        branch.name_span.start += absolute_start;
        branch.name_span.end += absolute_start;
    }
    (trimmed.to_string(), branches)
}

#[allow(clippy::too_many_arguments)]
fn push_style_block_from_cst(
    source: &str,
    header: &str,
    bounds: CstBlockBounds,
    branches: &[SelectorBranch],
    parent_branches: &[SelectorBranch],
    parent_is_grouped: bool,
    wrapper: WrapperContext,
    blocks: &mut Vec<StyleBlock>,
) {
    let context_text = if branches.is_empty() {
        (!header.is_empty()).then(|| header.to_string())
    } else {
        None
    };
    blocks.push(StyleBlock {
        names: branches.iter().map(|branch| branch.name.clone()).collect(),
        context_text,
        start: bounds.body_start,
        end: bounds.close,
        rule_start: bounds.header_start,
        rule_end: bounds.rule_end,
        body_start: bounds.body_start,
        body_end: bounds.close,
        header_text: Some(header.to_string()),
        under_media: wrapper.under_media,
        under_supports: wrapper.under_supports,
        under_layer: wrapper.under_layer,
        wrapper_at_rules: wrapper.wrapper_at_rules.clone(),
    });
    let nested_safety =
        classify_nested_safety(header, branches, parent_branches, parent_is_grouped);
    for branch in branches {
        blocks.push(StyleBlock {
            names: vec![format!("__selector_meta:{}:{nested_safety}", branch.name)],
            context_text: source
                .get(branch.name_span.start..branch.name_span.end)
                .map(ToString::to_string),
            start: branch.name_span.start,
            end: branch.name_span.end,
            rule_start: bounds.header_start,
            rule_end: bounds.rule_end,
            body_start: bounds.body_start,
            body_end: bounds.close,
            header_text: Some(header.to_string()),
            under_media: wrapper.under_media,
            under_supports: wrapper.under_supports,
            under_layer: wrapper.under_layer,
            wrapper_at_rules: wrapper.wrapper_at_rules.clone(),
        });
    }
}

fn nearest_parent_branches(
    node: &SyntaxNode<SyntaxKind>,
    branches_by_node: &BTreeMap<(usize, usize), Vec<SelectorBranch>>,
) -> Vec<SelectorBranch> {
    node.ancestors()
        .skip(1)
        .find(|ancestor| matches!(ancestor.kind(), SyntaxKind::Rule | SyntaxKind::NestRule))
        .and_then(|ancestor| branches_by_node.get(&node_key(ancestor)))
        .cloned()
        .unwrap_or_default()
}

fn wrapper_context_for_node(
    source: &str,
    line_index: &SourceLineIndex,
    parsed: &ParseResult,
    node: &SyntaxNode<SyntaxKind>,
) -> WrapperContext {
    let mut ancestors = node.ancestors().collect::<Vec<_>>();
    ancestors.reverse();
    let mut wrapper = WrapperContext::default();
    for ancestor in ancestors {
        if !is_style_wrapper_node(ancestor.kind()) {
            continue;
        }
        let Some(bounds) = block_bounds(parsed, ancestor) else {
            continue;
        };
        let header = source
            .get(bounds.header_start..bounds.open)
            .unwrap_or_default()
            .trim();
        wrapper.under_media |= ancestor.kind() == SyntaxKind::MediaRule;
        wrapper.under_supports |= ancestor.kind() == SyntaxKind::SupportsRule;
        wrapper.under_layer |= ancestor.kind() == SyntaxKind::LayerRule;
        wrapper.wrapper_at_rules.push(at_rule_context_for_block(
            source,
            line_index,
            header,
            bounds.header_start,
            bounds.rule_end,
        ));
    }
    wrapper
}

fn has_excluded_style_owner_ancestor(node: &SyntaxNode<SyntaxKind>) -> bool {
    for ancestor in node.ancestors().skip(1) {
        if crate::is_at_rule_node_kind(ancestor.kind())
            && ancestor.kind() != SyntaxKind::AtRule
            && !is_style_wrapper_node(ancestor.kind())
        {
            return true;
        }
        if matches!(
            ancestor.kind(),
            SyntaxKind::ScssMixinDeclaration
                | SyntaxKind::ScssFunctionDeclaration
                | SyntaxKind::LessMixinDeclaration
                | SyntaxKind::LessDetachedRulesetNode
        ) {
            return true;
        }
    }
    false
}

fn is_style_wrapper_node(kind: SyntaxKind) -> bool {
    matches!(
        kind,
        SyntaxKind::MediaRule
            | SyntaxKind::SupportsRule
            | SyntaxKind::ContainerRule
            | SyntaxKind::LayerRule
            | SyntaxKind::ScopeRule
            | SyntaxKind::StartingStyleRule
            | SyntaxKind::WhenRule
            | SyntaxKind::ElseRule
            | SyntaxKind::IfRule
            | SyntaxKind::ScssControlIf
            | SyntaxKind::ScssControlElse
            | SyntaxKind::ScssControlEach
            | SyntaxKind::ScssControlFor
            | SyntaxKind::ScssControlWhile
            | SyntaxKind::ScssAtRootRule
    )
}

fn node_key(node: &SyntaxNode<SyntaxKind>) -> (usize, usize) {
    let span = byte_span_for_range(node.text_range());
    (span.start, span.end)
}

fn at_rule_context_for_block(
    source: &str,
    line_index: &SourceLineIndex,
    header: &str,
    start: usize,
    end: usize,
) -> ParserIndexAtRuleContextV0 {
    let trimmed = header.trim();
    let without_at = trimmed.strip_prefix('@').unwrap_or(trimmed);
    let split_index = without_at
        .find(|character: char| character.is_whitespace())
        .unwrap_or(without_at.len());
    let name = without_at[..split_index].to_string();
    let params = without_at[split_index..].trim().to_string();
    let byte_span = crate::ParserByteSpanV0 { start, end };
    ParserIndexAtRuleContextV0 {
        name,
        params,
        byte_span,
        range: parser_range_for_byte_span(source, line_index, byte_span),
    }
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeSet;

    use serde_json::Value;

    use super::*;
    use crate::{StyleDialect, parse};

    #[test]
    fn conditional_groups_surface_nested_selectors_without_descriptor_selectors() {
        let source = "@container card (width > 1px) { .inside { color: red; } } @keyframes fade { from { opacity: 0; } to { opacity: 1; } } @font-face { font-family: demo; }";
        let blocks = collect(source, StyleDialect::Css);
        let names = block_names(&blocks);

        assert!(names.contains(&"inside"));
        assert!(!names.contains(&"from"));
        assert!(!names.contains(&"to"));
        let inside = blocks
            .iter()
            .find(|block| block.names.iter().any(|name| name == "inside"));
        assert!(inside.is_some(), "container descendant must be indexed");
        assert!(
            inside.is_some_and(|block| {
                block
                    .wrapper_at_rules
                    .iter()
                    .any(|wrapper| wrapper.name == "container")
            }),
            "container descendant must retain wrapper provenance"
        );
    }

    #[test]
    fn scss_control_rules_keep_typed_wrapper_context() {
        let source = "$enabled: false; @if $enabled { .on { color: green; } } @else { .off { color: red; } }";
        let blocks = collect(source, StyleDialect::Scss);

        for (name, wrapper_name) in [("on", "if"), ("off", "else")] {
            let block = blocks
                .iter()
                .find(|block| block.names.iter().any(|candidate| candidate == name));
            assert!(block.is_some(), "{name} must be indexed");
            assert!(
                block.is_some_and(|block| {
                    block
                        .wrapper_at_rules
                        .iter()
                        .any(|wrapper| wrapper.name == wrapper_name)
                }),
                "{name} must retain {wrapper_name} provenance"
            );
        }
    }

    #[test]
    fn interpolation_braces_do_not_split_rule_boundaries() {
        let cases = [
            (
                StyleDialect::Less,
                "@name: card;\n.card-@{name} { color: red; }",
                "card-",
            ),
            (
                StyleDialect::Scss,
                "$color: red;\n.-text-#{$color}- { color: $color; }",
                "-text-",
            ),
        ];

        for (dialect, source, expected_name) in cases {
            let blocks = collect(source, dialect);
            let named = regular_named_blocks(&blocks);
            assert_eq!(named.len(), 1, "interpolation split a style rule");
            assert_eq!(named[0].names, [expected_name]);
            assert_eq!(named[0].rule_end, source.len());
        }
    }

    #[test]
    fn value_braces_do_not_create_style_blocks() {
        let cases = [
            (
                StyleDialect::Css,
                ".card { background-image: url(\"http://host/{token}\"); }",
            ),
            (StyleDialect::Scss, "$gap: 1px; .card { margin: #{$gap}; }"),
            (
                StyleDialect::Less,
                "@path: \"../img\"; .card { background: url(\"@{path}/icon.png\"); }",
            ),
        ];

        for (dialect, source) in cases {
            let blocks = collect(source, dialect);
            let named = regular_named_blocks(&blocks);
            assert_eq!(named.len(), 1, "value syntax created a style block");
            assert_eq!(named[0].names, ["card"]);
            assert!(
                blocks
                    .iter()
                    .filter(|block| !is_selector_meta_block(block))
                    .all(|block| !block.names.is_empty()),
                "value syntax created an empty style block"
            );
        }
    }

    #[test]
    fn less_mixins_are_not_selector_definitions() {
        let source = ".rounded(@r) { border-radius: @r; }\n.card { .rounded(4px); }";
        let blocks = collect(source, StyleDialect::Less);
        assert_eq!(block_names(&blocks), ["card"]);
    }

    #[test]
    fn repeated_selector_metadata_uses_each_rule_span() {
        let source = ".same { color: red; }\n.same { color: blue; }";
        let blocks = collect(source, StyleDialect::Css);
        let metadata = blocks
            .iter()
            .filter(|block| {
                block
                    .names
                    .first()
                    .is_some_and(|name| name == "__selector_meta:same:flat")
            })
            .collect::<Vec<_>>();

        assert_eq!(metadata.len(), 2);
        assert_ne!(metadata[0].start, metadata[1].start);
        for block in metadata {
            assert_eq!(source.get(block.start..block.end), Some("same"));
            assert!(block.rule_start <= block.start);
            assert!(block.end <= block.body_start);
        }
    }

    #[test]
    fn divergence_ledger_records_each_adjudicated_correction() {
        let parsed: Result<Value, _> =
            serde_json::from_str(include_str!("../../style-block-divergences.json"));
        assert!(parsed.is_ok(), "committed divergence ledger must parse");
        let value = parsed.unwrap_or_default();
        let kinds = value
            .get("adjudications")
            .and_then(Value::as_array)
            .into_iter()
            .flatten()
            .filter_map(|entry| entry.get("kind").and_then(Value::as_str))
            .collect::<BTreeSet<_>>();
        let expected = [
            "conditionalGroupDescendantRecovery",
            "interpolationBoundaryCorrection",
            "lessMixinFalsePositiveRemoval",
            "rawValueBraceFalsePositiveRemoval",
            "scssControlDescendantRecovery",
            "selectorMetaSpanCorrection",
        ]
        .into_iter()
        .collect::<BTreeSet<_>>();

        assert_eq!(kinds, expected);
        assert!(
            value
                .get("corpusInputCount")
                .and_then(Value::as_u64)
                .is_some_and(|count| count >= 150)
        );
        assert!(
            value
                .get("entries")
                .and_then(Value::as_array)
                .is_some_and(|entries| {
                    entries.iter().any(|entry| {
                        entry.get("label").and_then(Value::as_str)
                            == Some("parser-fixture:conditional-group-descendant")
                    })
                })
        );
    }

    fn collect(source: &str, dialect: StyleDialect) -> Vec<StyleBlock> {
        let line_index = SourceLineIndex::new(source);
        let parsed = parse(source, dialect);
        collect_style_blocks_from_cst(source, &line_index, &parsed)
    }

    fn regular_named_blocks(blocks: &[StyleBlock]) -> Vec<&StyleBlock> {
        blocks
            .iter()
            .filter(|block| !is_selector_meta_block(block) && !block.names.is_empty())
            .collect()
    }

    fn block_names(blocks: &[StyleBlock]) -> Vec<&str> {
        blocks
            .iter()
            .flat_map(|block| block.names.iter())
            .filter_map(|name| (!name.starts_with("__selector_meta:")).then_some(name.as_str()))
            .collect()
    }

    fn is_selector_meta_block(block: &StyleBlock) -> bool {
        block.names.len() == 1 && block.names[0].starts_with("__selector_meta:")
    }
}

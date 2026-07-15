use std::collections::BTreeMap;

use cstree::syntax::SyntaxNode;

use crate::{ParseResult, SyntaxKind};

use super::{
    SelectorBranch, SourceLineIndex, StyleBlock, WrapperContext, at_rule_context_for_block,
    byte_span_for_range, classify_nested_safety, collect_style_blocks,
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

#[cfg(test)]
mod tests {
    use std::{
        collections::BTreeMap,
        fs,
        path::{Path, PathBuf},
    };

    use serde::Serialize;
    use serde_json::Value;

    use super::*;
    use crate::{StyleDialect, parse};

    #[derive(Debug)]
    struct CorpusInput {
        label: String,
        dialect: StyleDialect,
        source: String,
    }

    #[derive(Debug, Serialize)]
    #[serde(rename_all = "camelCase")]
    struct DivergenceReport {
        schema_version: &'static str,
        product: &'static str,
        corpus_input_count: usize,
        adjudications: Vec<DivergenceAdjudication>,
        entries: Vec<DivergenceEntry>,
    }

    #[derive(Debug, Serialize)]
    #[serde(rename_all = "camelCase")]
    struct DivergenceAdjudication {
        kind: &'static str,
        disposition: &'static str,
        reason: &'static str,
    }

    #[derive(Debug, Serialize)]
    #[serde(rename_all = "camelCase")]
    struct DivergenceEntry {
        label: String,
        kind: &'static str,
        old_only_count: usize,
        cst_only_count: usize,
    }

    #[test]
    fn cst_blocks_preserve_the_checked_in_corpus() {
        let corpus = checked_in_corpus();
        assert!(
            corpus.len() >= 50,
            "style-block corpus unexpectedly shrank: {}",
            corpus.len()
        );

        let corpus_input_count = corpus.len();
        let mut entries = Vec::new();
        let mut unclassified = Vec::new();
        for input in corpus {
            let line_index = SourceLineIndex::new(&input.source);
            let parsed = parse(&input.source, input.dialect);
            let old = collect_style_blocks(&input.source, &line_index);
            let new = collect_style_blocks_from_cst(&input.source, &line_index, &parsed);
            let exact_old_only = unmatched_blocks_exact(&old, &new);
            let exact_cst_only = unmatched_blocks_exact(&new, &old);
            if exact_old_only.is_empty() && exact_cst_only.is_empty() {
                continue;
            }

            let old_only = unmatched_blocks(&old, &new);
            let cst_only = unmatched_blocks(&new, &old);
            let Some(kind) = classify_divergence(
                &input,
                &parsed,
                &exact_old_only,
                &exact_cst_only,
                &old_only,
                &cst_only,
            ) else {
                unclassified.push(format!(
                    "unclassified style-block divergence for {}\n  old-only: {}\n  cst-only: {}",
                    input.label,
                    summarize_blocks(&exact_old_only),
                    summarize_blocks(&exact_cst_only)
                ));
                continue;
            };
            entries.push(DivergenceEntry {
                label: input.label,
                kind,
                old_only_count: exact_old_only.len(),
                cst_only_count: exact_cst_only.len(),
            });
        }
        assert!(unclassified.is_empty(), "{}", unclassified.join("\n"));

        let report = DivergenceReport {
            schema_version: "0",
            product: "omena.parser.style-block-divergences",
            corpus_input_count,
            adjudications: divergence_adjudications(),
            entries,
        };
        let actual_result = serde_json::to_value(report);
        assert!(actual_result.is_ok(), "divergence report must serialize");
        let actual = actual_result.unwrap_or_default();
        let expected_result: Result<Value, _> =
            serde_json::from_str(include_str!("../../style-block-divergences.json"));
        assert!(
            expected_result.is_ok(),
            "committed divergence report must parse"
        );
        let expected = expected_result.unwrap_or_default();
        assert_eq!(actual, expected, "style-block divergence ledger drifted");
    }

    #[test]
    fn cst_blocks_preserve_existing_blocks_and_expose_group_rule_descendants() {
        let cases = [
            (
                StyleDialect::Css,
                ".root { color: red; @media (width > 1px) { .child { color: blue; } } }",
            ),
            (
                StyleDialect::Scss,
                ".card { &__icon { color: red; } @supports (display: grid) { &--wide { display: grid; } } }",
            ),
            (
                StyleDialect::Css,
                "@container card (width > 1px) { .inside { color: red; } }",
            ),
            (
                StyleDialect::Css,
                "@keyframes fade { from { opacity: 0; } to { opacity: 1; } } @font-face { font-family: demo; }",
            ),
        ];

        for (dialect, source) in cases {
            let line_index = SourceLineIndex::new(source);
            let parsed = parse(source, dialect);
            let old = collect_style_blocks(source, &line_index);
            let new = collect_style_blocks_from_cst(source, &line_index, &parsed);
            let missing = unmatched_blocks(&old, &new);
            assert!(
                missing.is_empty(),
                "CST path lost existing blocks: {missing:#?}"
            );
            for extra in unmatched_blocks(&new, &old) {
                assert!(
                    has_new_conditional_group_context(&extra),
                    "unexpected CST block divergence: {extra:#?}"
                );
            }
        }
    }

    #[test]
    fn container_rules_surface_nested_selectors_without_keyframe_selectors() {
        let source = "@container card (width > 1px) { .inside { color: red; } } @keyframes fade { from { opacity: 0; } to { opacity: 1; } }";
        let line_index = SourceLineIndex::new(source);
        let parsed = parse(source, StyleDialect::Css);
        let old = collect_style_blocks(source, &line_index);
        let new = collect_style_blocks_from_cst(source, &line_index, &parsed);

        assert!(!block_names(&old).contains(&"inside"));
        assert!(block_names(&new).contains(&"inside"));
        assert!(!block_names(&new).contains(&"from"));
        assert!(!block_names(&new).contains(&"to"));
    }

    fn unmatched_blocks(left: &[StyleBlock], right: &[StyleBlock]) -> Vec<StyleBlock> {
        unmatched_blocks_by(left, right, equivalent_block)
    }

    fn unmatched_blocks_exact(left: &[StyleBlock], right: &[StyleBlock]) -> Vec<StyleBlock> {
        unmatched_blocks_by(left, right, |left, right| left == right)
    }

    fn unmatched_blocks_by(
        left: &[StyleBlock],
        right: &[StyleBlock],
        equivalent: impl Fn(&StyleBlock, &StyleBlock) -> bool,
    ) -> Vec<StyleBlock> {
        let mut remaining = right.to_vec();
        let mut unmatched = Vec::new();
        for block in left {
            if let Some(index) = remaining
                .iter()
                .position(|candidate| equivalent(block, candidate))
            {
                remaining.remove(index);
            } else {
                unmatched.push(block.clone());
            }
        }
        unmatched
    }

    fn classify_divergence(
        input: &CorpusInput,
        parsed: &ParseResult,
        exact_old_only: &[StyleBlock],
        exact_cst_only: &[StyleBlock],
        old_only: &[StyleBlock],
        cst_only: &[StyleBlock],
    ) -> Option<&'static str> {
        if old_only.is_empty()
            && cst_only.is_empty()
            && exact_old_only.iter().all(is_selector_meta_block)
            && exact_cst_only.iter().all(is_selector_meta_block)
        {
            return Some("selectorMetaSpanCorrection");
        }
        if cst_only.is_empty()
            && !old_only.is_empty()
            && old_only.iter().all(|block| block.names.is_empty())
            && (input.source.contains("url(\"")
                || input.source.contains("#{")
                || input.source.contains("@{"))
        {
            return Some("rawValueBraceFalsePositiveRemoval");
        }
        if cst_only.is_empty()
            && old_only.iter().any(|block| !block.names.is_empty())
            && parsed
                .syntax()
                .descendants()
                .any(|node| node.kind() == SyntaxKind::LessMixinDeclaration)
        {
            return Some("lessMixinFalsePositiveRemoval");
        }
        if !old_only.is_empty()
            && !cst_only.is_empty()
            && (input.source.contains("#{") || input.source.contains("@{"))
            && sorted_block_names(old_only) == sorted_block_names(cst_only)
        {
            return Some("interpolationBoundaryCorrection");
        }
        if old_only.is_empty()
            && !cst_only.is_empty()
            && cst_only.iter().all(has_scss_control_context)
        {
            return Some("scssControlDescendantRecovery");
        }
        if old_only.is_empty()
            && !cst_only.is_empty()
            && cst_only.iter().all(has_new_conditional_group_context)
        {
            return Some("conditionalGroupDescendantRecovery");
        }
        None
    }

    fn divergence_adjudications() -> Vec<DivergenceAdjudication> {
        vec![
            DivergenceAdjudication {
                kind: "conditionalGroupDescendantRecovery",
                disposition: "fix",
                reason: "Typed group-rule descendants are style rules; keyframe and descriptor bodies remain excluded.",
            },
            DivergenceAdjudication {
                kind: "scssControlDescendantRecovery",
                disposition: "fix",
                reason: "Selectors in top-level Sass control blocks remain source selectors and retain typed wrapper provenance.",
            },
            DivergenceAdjudication {
                kind: "selectorMetaSpanCorrection",
                disposition: "fix",
                reason: "Selector metadata uses the current CST rule span instead of the first matching selector spelling in the file.",
            },
            DivergenceAdjudication {
                kind: "interpolationBoundaryCorrection",
                disposition: "fix",
                reason: "Interpolation braces are selector syntax, not style-block boundaries.",
            },
            DivergenceAdjudication {
                kind: "rawValueBraceFalsePositiveRemoval",
                disposition: "fix",
                reason: "Braces inside strings, URLs, and interpolated values do not create style blocks.",
            },
            DivergenceAdjudication {
                kind: "lessMixinFalsePositiveRemoval",
                disposition: "fix",
                reason: "A Less mixin declaration is not a CSS Modules class selector definition.",
            },
        ]
    }

    fn equivalent_block(left: &StyleBlock, right: &StyleBlock) -> bool {
        if left == right {
            return true;
        }
        is_selector_meta_block(left)
            && is_selector_meta_block(right)
            && left.names == right.names
            && left.context_text == right.context_text
            && left.rule_start == right.rule_start
            && left.rule_end == right.rule_end
            && left.body_start == right.body_start
            && left.body_end == right.body_end
            && left.header_text == right.header_text
            && left.under_media == right.under_media
            && left.under_supports == right.under_supports
            && left.under_layer == right.under_layer
            && left.wrapper_at_rules == right.wrapper_at_rules
    }

    fn is_selector_meta_block(block: &StyleBlock) -> bool {
        block.names.len() == 1 && block.names[0].starts_with("__selector_meta:")
    }

    fn block_names(blocks: &[StyleBlock]) -> Vec<&str> {
        blocks
            .iter()
            .flat_map(|block| block.names.iter())
            .filter_map(|name| (!name.starts_with("__selector_meta:")).then_some(name.as_str()))
            .collect()
    }

    fn sorted_block_names(blocks: &[StyleBlock]) -> Vec<String> {
        let mut names = block_names(blocks)
            .into_iter()
            .map(ToString::to_string)
            .collect::<Vec<_>>();
        names.sort();
        names.dedup();
        names
    }

    fn summarize_blocks(blocks: &[StyleBlock]) -> String {
        blocks
            .iter()
            .take(8)
            .map(|block| {
                format!(
                    "{:?}@{}..{} body={}..{} selection={}..{} context={:?} wrappers={:?}",
                    block.names,
                    block.rule_start,
                    block.rule_end,
                    block.body_start,
                    block.body_end,
                    block.start,
                    block.end,
                    block.context_text,
                    block
                        .wrapper_at_rules
                        .iter()
                        .map(|wrapper| wrapper.name.as_str())
                        .collect::<Vec<_>>()
                )
            })
            .chain((blocks.len() > 8).then(|| format!("... +{}", blocks.len() - 8)))
            .collect::<Vec<_>>()
            .join(", ")
    }

    fn has_new_conditional_group_context(block: &StyleBlock) -> bool {
        block.wrapper_at_rules.iter().any(|wrapper| {
            matches!(
                wrapper.name.as_str(),
                "container" | "scope" | "starting-style" | "when" | "else" | "if"
            )
        })
    }

    fn has_scss_control_context(block: &StyleBlock) -> bool {
        block.wrapper_at_rules.iter().any(|wrapper| {
            matches!(
                wrapper.name.as_str(),
                "if" | "else" | "for" | "each" | "while" | "at-root"
            )
        })
    }

    fn checked_in_corpus() -> Vec<CorpusInput> {
        let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        let repo_root = manifest_dir
            .ancestors()
            .nth(3)
            .map(Path::to_path_buf)
            .unwrap_or_default();
        let diff_test_root = manifest_dir
            .parent()
            .map(|path| path.join("omena-diff-test"))
            .unwrap_or_default();
        let mut corpus = BTreeMap::<(String, String), CorpusInput>::new();

        for root in [
            repo_root.join("src"),
            repo_root.join("test"),
            repo_root.join("examples"),
            diff_test_root.clone(),
        ] {
            collect_corpus_files(&root, &diff_test_root, &mut corpus);
        }
        insert_corpus_input(
            "parser-fixture:conditional-group-descendant".to_string(),
            StyleDialect::Css,
            "@container card (width > 1px) { .inside { color: red; } } @keyframes fade { from { opacity: 0; } to { opacity: 1; } }".to_string(),
            &mut corpus,
        );

        let repo_prefix = format!("{}/", repo_root.display());
        corpus
            .into_values()
            .map(|mut input| {
                if let Some(relative) = input.label.strip_prefix(&repo_prefix) {
                    input.label = relative.to_string();
                }
                input
            })
            .collect()
    }

    fn collect_corpus_files(
        root: &Path,
        diff_test_root: &Path,
        corpus: &mut BTreeMap<(String, String), CorpusInput>,
    ) {
        if is_generated_or_dependency_directory(root) {
            return;
        }
        let Ok(entries) = fs::read_dir(root) else {
            return;
        };
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                collect_corpus_files(&path, diff_test_root, corpus);
                continue;
            }

            let extension = path
                .extension()
                .and_then(|value| value.to_str())
                .unwrap_or_default();
            if let Some(dialect) = dialect_from_name(extension) {
                if let Ok(source) = fs::read_to_string(&path)
                    && !source.trim().is_empty()
                {
                    insert_corpus_input(path.display().to_string(), dialect, source, corpus);
                }
            } else if extension == "json"
                && path.starts_with(diff_test_root)
                && let Ok(source) = fs::read_to_string(&path)
                && let Ok(value) = serde_json::from_str::<Value>(&source)
            {
                collect_json_sources(&value, &path.display().to_string(), corpus);
            }
        }
    }

    fn is_generated_or_dependency_directory(path: &Path) -> bool {
        path.file_name().is_some_and(|name| {
            matches!(
                name.to_str(),
                Some("node_modules" | "target" | "dist" | ".git")
            )
        })
    }

    fn collect_json_sources(
        value: &Value,
        origin: &str,
        corpus: &mut BTreeMap<(String, String), CorpusInput>,
    ) {
        match value {
            Value::Array(values) => {
                for value in values {
                    collect_json_sources(value, origin, corpus);
                }
            }
            Value::Object(object) => {
                if let Some(source) = object.get("source").and_then(Value::as_str) {
                    let dialect = object
                        .get("dialect")
                        .and_then(Value::as_str)
                        .and_then(dialect_from_name)
                        .unwrap_or(StyleDialect::Css);
                    let label = object
                        .get("id")
                        .or_else(|| object.get("label"))
                        .and_then(Value::as_str)
                        .map_or_else(|| origin.to_string(), |name| format!("{origin}:{name}"));
                    insert_corpus_input(label, dialect, source.to_string(), corpus);
                }
                for value in object.values() {
                    collect_json_sources(value, origin, corpus);
                }
            }
            _ => {}
        }
    }

    fn insert_corpus_input(
        label: String,
        dialect: StyleDialect,
        source: String,
        corpus: &mut BTreeMap<(String, String), CorpusInput>,
    ) {
        let key = (format!("{dialect:?}"), source.clone());
        corpus.entry(key).or_insert(CorpusInput {
            label,
            dialect,
            source,
        });
    }

    fn dialect_from_name(name: &str) -> Option<StyleDialect> {
        match name {
            "css" => Some(StyleDialect::Css),
            "scss" => Some(StyleDialect::Scss),
            "sass" => Some(StyleDialect::Sass),
            "less" => Some(StyleDialect::Less),
            _ => None,
        }
    }
}

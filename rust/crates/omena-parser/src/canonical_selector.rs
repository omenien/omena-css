//! Parser-CST producer for the shared canonical selector authority.

use omena_syntax::{CanonicalSelectorAst, StyleDialect, SyntaxKind, SyntaxNode};

use crate::{ParsedCst, parse_only};

pub fn canonical_selector_asts_from_cst(cst: &ParsedCst) -> Vec<CanonicalSelectorAst> {
    cst.root()
        .descendants()
        .filter(|node| {
            matches!(
                node.kind(),
                SyntaxKind::SelectorList
                    | SyntaxKind::RelativeSelectorList
                    | SyntaxKind::BogusSelectorList
            ) && !node.ancestors().skip(1).any(|ancestor| {
                matches!(
                    ancestor.kind(),
                    SyntaxKind::SelectorList
                        | SyntaxKind::RelativeSelectorList
                        | SyntaxKind::BogusSelectorList
                )
            })
        })
        .filter_map(CanonicalSelectorAst::from_cst)
        .collect()
}

pub fn canonical_selector_ast_from_source(
    selector: &str,
    dialect: StyleDialect,
) -> Option<CanonicalSelectorAst> {
    let wrapper = format!("{selector}{{}} ");
    let parsed = parse_only(wrapper.as_str(), dialect);
    let root = parsed.syntax();
    let selector_list = first_top_level_selector_list(&root)?;
    CanonicalSelectorAst::from_cst(&selector_list)
}

pub fn expand_nested_selector_from_cst(
    parent_selector: &str,
    nested_selector: &str,
    dialect: StyleDialect,
) -> Option<String> {
    let parent = canonical_selector_ast_from_source(parent_selector, dialect)?;
    let nested = canonical_selector_ast_from_source(nested_selector, dialect)?;
    nested.expand_with_parent(&parent)
}

fn first_top_level_selector_list(root: &SyntaxNode) -> Option<SyntaxNode> {
    root.descendants()
        .find(|node| {
            matches!(
                node.kind(),
                SyntaxKind::SelectorList
                    | SyntaxKind::RelativeSelectorList
                    | SyntaxKind::BogusSelectorList
            ) && !node.ancestors().skip(1).any(|ancestor| {
                matches!(
                    ancestor.kind(),
                    SyntaxKind::SelectorList
                        | SyntaxKind::RelativeSelectorList
                        | SyntaxKind::BogusSelectorList
                )
            })
        })
        .cloned()
}

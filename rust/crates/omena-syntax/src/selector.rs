//! Canonical selector structure shared by parser, transforms, queries, and matching.
//!
//! The parser supplies a selector CST node. This module projects that CST into
//! one source-preserving authority: nesting tokens retain exact byte spans,
//! while identity-bearing selector names are represented by sealed decoded keys.

use std::ops::Range;

use crate::{
    SyntaxKind, SyntaxNode,
    ident::{CanonicalClassKeyV0, CanonicalIdKeyV0, CanonicalTypeSelectorKeyV0, ClassNameV0},
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CanonicalSelectorCombinatorV0 {
    Descendant,
    Child,
    NextSibling,
    SubsequentSibling,
    Column,
    Other,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CanonicalSelectorSpecificityWitnessV0 {
    pub ids: u32,
    pub classes: u32,
    pub types: u32,
    pub exact: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NestingTokenV0 {
    byte_range: Range<usize>,
}

impl NestingTokenV0 {
    pub fn byte_range(&self) -> Range<usize> {
        self.byte_range.clone()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CanonicalCompoundSelectorV0 {
    byte_range: Range<usize>,
    required_tag: Option<CanonicalTypeSelectorKeyV0>,
    required_id: Option<CanonicalIdKeyV0>,
    required_classes: Vec<CanonicalClassKeyV0>,
    nesting_tokens: Vec<NestingTokenV0>,
}

impl CanonicalCompoundSelectorV0 {
    pub fn byte_range(&self) -> Range<usize> {
        self.byte_range.clone()
    }

    pub fn required_tag(&self) -> Option<&CanonicalTypeSelectorKeyV0> {
        self.required_tag.as_ref()
    }

    pub fn required_id(&self) -> Option<&CanonicalIdKeyV0> {
        self.required_id.as_ref()
    }

    pub fn required_classes(&self) -> &[CanonicalClassKeyV0] {
        self.required_classes.as_slice()
    }

    pub fn nesting_tokens(&self) -> &[NestingTokenV0] {
        self.nesting_tokens.as_slice()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CanonicalSelectorBranchV0 {
    authored: String,
    byte_range: Range<usize>,
    compounds: Vec<CanonicalCompoundSelectorV0>,
    combinators: Vec<CanonicalSelectorCombinatorV0>,
    nesting_tokens: Vec<NestingTokenV0>,
    specificity: CanonicalSelectorSpecificityWitnessV0,
}

impl CanonicalSelectorBranchV0 {
    pub fn authored(&self) -> &str {
        self.authored.as_str()
    }

    pub fn byte_range(&self) -> Range<usize> {
        self.byte_range.clone()
    }

    pub fn compounds(&self) -> &[CanonicalCompoundSelectorV0] {
        self.compounds.as_slice()
    }

    pub fn combinators(&self) -> &[CanonicalSelectorCombinatorV0] {
        self.combinators.as_slice()
    }

    pub fn nesting_tokens(&self) -> &[NestingTokenV0] {
        self.nesting_tokens.as_slice()
    }

    pub fn specificity(&self) -> CanonicalSelectorSpecificityWitnessV0 {
        self.specificity
    }

    fn substitute_nesting(&self, parent: &str) -> String {
        if self.nesting_tokens.is_empty() {
            return format!("{parent} {}", self.authored.trim());
        }
        let branch_start = self.byte_range.start;
        let mut output = String::with_capacity(self.authored.len().saturating_add(parent.len()));
        let mut cursor = 0usize;
        for token in &self.nesting_tokens {
            let start = token.byte_range.start.saturating_sub(branch_start);
            let end = token.byte_range.end.saturating_sub(branch_start);
            if start < cursor || end > self.authored.len() {
                continue;
            }
            output.push_str(&self.authored[cursor..start]);
            output.push_str(parent);
            cursor = end;
        }
        output.push_str(&self.authored[cursor..]);
        output
    }
}

/// The sole selector canonicalization authority.
///
/// Construction accepts parser CST nodes, so quoted, escaped, and attribute
/// interiors never need to be rediscovered by a string replacement pass.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CanonicalSelectorAst {
    authored: String,
    branches: Vec<CanonicalSelectorBranchV0>,
}

impl CanonicalSelectorAst {
    pub fn from_cst(selector_node: &SyntaxNode) -> Option<Self> {
        if !matches!(
            selector_node.kind(),
            SyntaxKind::SelectorList
                | SyntaxKind::RelativeSelectorList
                | SyntaxKind::BogusSelectorList
                | SyntaxKind::Selector
                | SyntaxKind::RelativeSelector
                | SyntaxKind::BogusSelector
        ) {
            return None;
        }
        let authored = syntax_node_text(selector_node)?;
        let authority_start = byte_start(selector_node);
        let branch_nodes = if matches!(
            selector_node.kind(),
            SyntaxKind::Selector | SyntaxKind::RelativeSelector | SyntaxKind::BogusSelector
        ) {
            vec![selector_node.clone()]
        } else {
            selector_node
                .children()
                .filter(|child| {
                    matches!(
                        child.kind(),
                        SyntaxKind::Selector
                            | SyntaxKind::RelativeSelector
                            | SyntaxKind::BogusSelector
                    )
                })
                .cloned()
                .collect::<Vec<_>>()
        };
        let branches = branch_nodes
            .iter()
            .filter_map(|branch| build_branch(branch, authority_start, authored.as_str()))
            .collect::<Vec<_>>();
        (!branches.is_empty()).then_some(Self { authored, branches })
    }

    pub fn authored(&self) -> &str {
        self.authored.as_str()
    }

    pub fn branches(&self) -> &[CanonicalSelectorBranchV0] {
        self.branches.as_slice()
    }

    pub fn nesting_token_count(&self) -> usize {
        self.branches
            .iter()
            .map(|branch| branch.nesting_tokens.len())
            .sum()
    }

    pub fn canonical_class_keys(&self) -> impl Iterator<Item = &CanonicalClassKeyV0> {
        self.branches
            .iter()
            .flat_map(|branch| branch.compounds.iter())
            .flat_map(|compound| compound.required_classes.iter())
    }

    pub fn expand_with_parent(&self, parent: &Self) -> Option<String> {
        let mut expanded = Vec::new();
        for parent_branch in &parent.branches {
            let parent_text = parent_branch.authored.trim();
            if parent_text.is_empty() {
                continue;
            }
            for nested_branch in &self.branches {
                expanded.push(nested_branch.substitute_nesting(parent_text));
            }
        }
        (!expanded.is_empty()).then(|| expanded.join(", "))
    }
}

fn build_branch(
    branch: &SyntaxNode,
    authority_start: usize,
    authority_text: &str,
) -> Option<CanonicalSelectorBranchV0> {
    let raw_branch_range = relative_range(branch, authority_start)?;
    let raw_authored = authority_text.get(raw_branch_range.clone())?;
    let leading_trivia_bytes = raw_authored
        .len()
        .saturating_sub(raw_authored.trim_start().len());
    let trailing_trivia_bytes = raw_authored
        .len()
        .saturating_sub(raw_authored.trim_end().len());
    let branch_range = raw_branch_range.start.saturating_add(leading_trivia_bytes)
        ..raw_branch_range.end.saturating_sub(trailing_trivia_bytes);
    let authored = authority_text.get(branch_range.clone())?.to_string();
    let complex = branch
        .children()
        .find(|child| child.kind() == SyntaxKind::ComplexSelector)
        .unwrap_or(branch);
    let compounds = complex
        .children()
        .filter(|child| child.kind() == SyntaxKind::CompoundSelector)
        .filter_map(|compound| build_compound(&compound, authority_start))
        .collect::<Vec<_>>();
    let combinators = complex
        .children()
        .filter(|child| child.kind() == SyntaxKind::Combinator)
        .map(|node| classify_combinator(syntax_node_text(&node).as_deref().unwrap_or_default()))
        .collect::<Vec<_>>();
    let nesting_tokens = branch
        .descendants()
        .filter(|node| node.kind() == SyntaxKind::NestingSelectorNode)
        .filter_map(|node| {
            relative_range(&node, authority_start).map(|byte_range| NestingTokenV0 { byte_range })
        })
        .collect::<Vec<_>>();
    let specificity = specificity_witness(branch);
    Some(CanonicalSelectorBranchV0 {
        authored,
        byte_range: branch_range,
        compounds,
        combinators,
        nesting_tokens,
        specificity,
    })
}

fn build_compound(
    compound: &SyntaxNode,
    authority_start: usize,
) -> Option<CanonicalCompoundSelectorV0> {
    let mut required_tag = None;
    let mut required_id = None;
    let mut required_classes = Vec::new();
    let mut nesting_tokens = Vec::new();
    for node in compound.children() {
        match node.kind() {
            SyntaxKind::ClassSelector => {
                if let Some(text) = syntax_node_text(&node)
                    && let Some(raw) = text.strip_prefix('.')
                {
                    required_classes.push(ClassNameV0::new(raw).canonical_key());
                }
            }
            SyntaxKind::IdSelector => {
                if let Some(text) = syntax_node_text(&node)
                    && let Some(raw) = text.strip_prefix('#')
                {
                    required_id = Some(CanonicalIdKeyV0::from_authored(raw));
                }
            }
            SyntaxKind::TypeSelector => {
                if let Some(text) = syntax_node_text(&node) {
                    let raw = text.rsplit('|').next().unwrap_or(text.as_str()).trim();
                    if raw != "*" && !raw.is_empty() {
                        required_tag = Some(CanonicalTypeSelectorKeyV0::from_authored(raw));
                    }
                }
            }
            SyntaxKind::NestingSelectorNode => {
                if let Some(byte_range) = relative_range(&node, authority_start) {
                    nesting_tokens.push(NestingTokenV0 { byte_range });
                }
            }
            _ => {}
        }
    }
    Some(CanonicalCompoundSelectorV0 {
        byte_range: relative_range(compound, authority_start)?,
        required_tag,
        required_id,
        required_classes,
        nesting_tokens,
    })
}

fn specificity_witness(branch: &SyntaxNode) -> CanonicalSelectorSpecificityWitnessV0 {
    let mut witness = CanonicalSelectorSpecificityWitnessV0 {
        ids: 0,
        classes: 0,
        types: 0,
        exact: true,
    };
    for node in branch.descendants() {
        match node.kind() {
            SyntaxKind::IdSelector => witness.ids = witness.ids.saturating_add(1),
            SyntaxKind::ClassSelector
            | SyntaxKind::AttributeSelector
            | SyntaxKind::PseudoClassSelector => {
                witness.classes = witness.classes.saturating_add(1)
            }
            SyntaxKind::TypeSelector | SyntaxKind::PseudoElementSelector => {
                witness.types = witness.types.saturating_add(1)
            }
            SyntaxKind::PseudoSelectorArgument
            | SyntaxKind::NthSelectorArgument
            | SyntaxKind::BogusSelector
            | SyntaxKind::BogusCompoundSelector => witness.exact = false,
            _ => {}
        }
    }
    witness
}

fn classify_combinator(text: &str) -> CanonicalSelectorCombinatorV0 {
    match text.trim() {
        "" => CanonicalSelectorCombinatorV0::Descendant,
        ">" => CanonicalSelectorCombinatorV0::Child,
        "+" => CanonicalSelectorCombinatorV0::NextSibling,
        "~" => CanonicalSelectorCombinatorV0::SubsequentSibling,
        "||" => CanonicalSelectorCombinatorV0::Column,
        _ => CanonicalSelectorCombinatorV0::Other,
    }
}

fn relative_range(node: &SyntaxNode, authority_start: usize) -> Option<Range<usize>> {
    let start = byte_start(node).checked_sub(authority_start)?;
    let end = byte_end(node).checked_sub(authority_start)?;
    (start <= end).then_some(start..end)
}

fn byte_start(node: &SyntaxNode) -> usize {
    u32::from(node.text_range().start()) as usize
}

fn byte_end(node: &SyntaxNode) -> usize {
    u32::from(node.text_range().end()) as usize
}

fn syntax_node_text(node: &SyntaxNode) -> Option<String> {
    node.try_resolved()
        .map(|resolved| resolved.text().to_string())
}

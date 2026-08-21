//! Parser facts for selectors that are absent from the legacy named-selector surface.

use cstree::text::TextRange;
use omena_syntax::SyntaxKind;

use crate::ParseResult;

use super::StyleFactSink;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
#[non_exhaustive]
pub enum ParsedEmissionSelectorFactKindV0 {
    Element,
    Attribute,
    Universal,
    PseudoClass,
    PseudoElement,
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub struct ParsedEmissionSelectorFactV0 {
    pub kind: ParsedEmissionSelectorFactKindV0,
    pub name: String,
    pub range: TextRange,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
#[non_exhaustive]
pub struct ParsedEmissionSelectorFactsV0 {
    pub selectors: Vec<ParsedEmissionSelectorFactV0>,
}

pub fn collect_emission_selector_facts_from_cst(
    text: &str,
    parsed: &ParseResult,
) -> ParsedEmissionSelectorFactsV0 {
    let sink = StyleFactSink::from_cst(text, parsed);
    collect_emission_selector_facts_from_sink(&sink)
}

pub(crate) fn collect_emission_selector_facts_from_sink(
    sink: &StyleFactSink<'_>,
) -> ParsedEmissionSelectorFactsV0 {
    let mut selectors = sink
        .nodes()
        .filter_map(|node| {
            let kind = match node.kind {
                SyntaxKind::TypeSelector => ParsedEmissionSelectorFactKindV0::Element,
                SyntaxKind::AttributeSelector => ParsedEmissionSelectorFactKindV0::Attribute,
                SyntaxKind::UniversalSelector => ParsedEmissionSelectorFactKindV0::Universal,
                SyntaxKind::PseudoClassSelector => ParsedEmissionSelectorFactKindV0::PseudoClass,
                SyntaxKind::PseudoElementSelector => {
                    ParsedEmissionSelectorFactKindV0::PseudoElement
                }
                _ => return None,
            };
            let range = node.range;
            let start = u32::from(range.start()) as usize;
            let end = u32::from(range.end()) as usize;
            Some(ParsedEmissionSelectorFactV0 {
                kind,
                name: sink.text().get(start..end).unwrap_or_default().to_string(),
                range,
            })
        })
        .collect::<Vec<_>>();
    selectors.sort_by_key(|selector| {
        (
            u32::from(selector.range.start()),
            u32::from(selector.range.end()),
            selector.kind,
            selector.name.clone(),
        )
    });
    ParsedEmissionSelectorFactsV0 { selectors }
}

#[cfg(test)]
mod tests {
    use super::{ParsedEmissionSelectorFactKindV0, collect_emission_selector_facts_from_cst};
    use crate::{
        StyleDialect, collect_style_fact_collection, parse, with_omena_parser_parse_instrumentation,
    };

    #[test]
    fn collects_structural_selectors_absent_from_the_named_selector_surface() {
        let source = "div, [data-state='open'], svg|*, :root, ::before { color: red; }";
        let parsed = parse(source, StyleDialect::Css);
        let facts = collect_emission_selector_facts_from_cst(source, &parsed);

        assert_eq!(
            facts
                .selectors
                .iter()
                .map(|selector| (selector.kind, selector.name.as_str()))
                .collect::<Vec<_>>(),
            vec![
                (ParsedEmissionSelectorFactKindV0::Element, "div"),
                (
                    ParsedEmissionSelectorFactKindV0::Attribute,
                    "[data-state='open']"
                ),
                (ParsedEmissionSelectorFactKindV0::Universal, "svg|*"),
                (ParsedEmissionSelectorFactKindV0::PseudoClass, ":root"),
                (ParsedEmissionSelectorFactKindV0::PseudoElement, "::before"),
            ]
        );
    }

    #[test]
    fn combined_fact_collection_parses_once() {
        let (collection, snapshot) = with_omena_parser_parse_instrumentation(|| {
            collect_style_fact_collection("div { color: red; }", StyleDialect::Css)
        });

        assert_eq!(snapshot.parse_invocation_count, 1);
        assert!(collection.facts.selectors.is_empty());
        assert_eq!(collection.emission_selectors.selectors.len(), 1);
    }
}

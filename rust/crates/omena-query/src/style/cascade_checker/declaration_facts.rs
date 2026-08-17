use std::collections::BTreeSet;

use omena_cascade::{CascadeOriginV0, LayerOrdinal};
use omena_parser::{
    ParserByteSpanV0, ParserDeclarationSyntaxFactV0, StyleDialect,
    collect_parser_declaration_syntax_facts,
};
use omena_query_checker_orchestrator::{CanonicalSelector, OmenaCheckerCascadeDeclarationInputV0};
use omena_query_transform_runner::expand_css_nested_selector;
use omena_semantic::{
    LayerBindingResolutionV0, StyleContextIndexV0, layer_ordinal_for_byte_span,
    summarize_omena_parser_style_semantic_boundary_from_source,
};

use super::runtime_state::query_selector_class_names;
use super::source_scanner::collect_query_var_references_in_value;

/// The query-owned join of one parser declaration with its semantic wrapper
/// context. This is deliberately internal: public query payloads continue to
/// carry their existing declaration-id strings and checker inputs.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(in crate::style) struct ParsedDeclarationFactV0 {
    pub(in crate::style) declaration_id: String,
    pub(in crate::style) byte_span: ParserByteSpanV0,
    pub(in crate::style) selector: String,
    pub(in crate::style) property_name: String,
    pub(in crate::style) value: String,
    pub(in crate::style) important: bool,
    pub(in crate::style) condition_context: Vec<String>,
    pub(in crate::style) semantic_context_ids: Vec<String>,
    pub(in crate::style) layer_name: Option<String>,
    pub(in crate::style) layer_order: Option<i32>,
    pub(in crate::style) source_order: u32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(in crate::style) struct ParsedDeclarationFactCollectionV0 {
    pub(in crate::style) facts: Vec<ParsedDeclarationFactV0>,
    pub(in crate::style) topology_incomplete_unresolved_count: Option<usize>,
}

impl ParsedDeclarationFactV0 {
    #[allow(dead_code)]
    pub(in crate::style) fn checker_input(&self) -> OmenaCheckerCascadeDeclarationInputV0 {
        OmenaCheckerCascadeDeclarationInputV0 {
            declaration_id: self.declaration_id.clone(),
            selector: CanonicalSelector::from_canonical(self.selector.as_str()),
            property: self.property_name.clone(),
            value: self.value.clone(),
            source_order: self.source_order,
            condition_context: self.condition_context.clone(),
            layer_name: self.layer_name.clone(),
            layer_order: self.layer_order,
            origin: CascadeOriginV0::Author,
            important: self.important,
            var_references: collect_query_var_references_in_value(self.value.as_str()),
        }
    }
}

#[allow(dead_code)]
pub(in crate::style) fn collect_parsed_declaration_facts(
    style_path: &str,
    source: &str,
    dialect: StyleDialect,
) -> Vec<ParsedDeclarationFactV0> {
    collect_parsed_declaration_fact_collection(style_path, source, dialect).facts
}

pub(in crate::style) fn collect_parsed_declaration_fact_collection(
    style_path: &str,
    source: &str,
    dialect: StyleDialect,
) -> ParsedDeclarationFactCollectionV0 {
    let syntax_facts = collect_parser_declaration_syntax_facts(source, dialect);
    let semantic = summarize_omena_parser_style_semantic_boundary_from_source(style_path, source);
    let topology_incomplete_unresolved_count = (!semantic
        .semantic_facts
        .context_index
        .layer_index
        .topology_complete)
        .then_some(
            semantic
                .semantic_facts
                .context_index
                .layer_index
                .unresolved_topology_count,
        );
    let facts = join_declaration_syntax_and_context(
        source,
        syntax_facts,
        &semantic.semantic_facts.context_index,
    );
    ParsedDeclarationFactCollectionV0 {
        facts,
        topology_incomplete_unresolved_count,
    }
}

fn join_declaration_syntax_and_context(
    source: &str,
    syntax_facts: Vec<ParserDeclarationSyntaxFactV0>,
    context_index: &StyleContextIndexV0,
) -> Vec<ParsedDeclarationFactV0> {
    let mut facts = Vec::new();
    for syntax_fact in syntax_facts {
        let selectors = canonical_selectors_for_syntax_fact(&syntax_fact);
        for selector in selectors {
            let Some(value) = source
                .get(syntax_fact.value_span.start..syntax_fact.value_span.end)
                .map(str::trim)
                .map(ToString::to_string)
            else {
                continue;
            };
            let source_order = facts.len().min(u32::MAX as usize) as u32;
            let (layer_name, layer_order) =
                layer_binding_for_span(context_index, syntax_fact.byte_span);
            facts.push(ParsedDeclarationFactV0 {
                declaration_id: declaration_id_for_source_order(source_order),
                byte_span: syntax_fact.byte_span,
                semantic_context_ids: semantic_context_ids_for_selector(
                    context_index,
                    selector.as_str(),
                ),
                selector,
                property_name: syntax_fact.property_name.clone(),
                value,
                important: syntax_fact.important,
                condition_context: syntax_fact.condition_contexts.clone(),
                layer_name,
                layer_order,
                source_order,
            });
        }
    }
    facts
}

fn canonical_selectors_for_syntax_fact(fact: &ParserDeclarationSyntaxFactV0) -> Vec<String> {
    let mut current = Vec::<String>::new();
    for context in &fact.selector_contexts {
        let parents = if context.reset_to_root {
            Vec::new()
        } else {
            current.clone()
        };
        let mut next = Vec::new();
        for member in &context.selector_members {
            if parents.is_empty() {
                push_unique(&mut next, member.trim().to_string());
                continue;
            }
            for parent in &parents {
                let expanded = expand_css_nested_selector(parent, member).unwrap_or_else(|| {
                    if member.contains('&') {
                        member.replace('&', parent)
                    } else {
                        format!("{parent} {member}")
                    }
                });
                push_unique(&mut next, expanded);
            }
        }
        current = next;
    }
    current
}

fn push_unique(values: &mut Vec<String>, value: String) {
    if !value.is_empty() && !values.contains(&value) {
        values.push(value);
    }
}

fn semantic_context_ids_for_selector(
    context_index: &StyleContextIndexV0,
    selector: &str,
) -> Vec<String> {
    let class_names = query_selector_class_names(selector)
        .into_iter()
        .collect::<BTreeSet<_>>();
    let mut ids = BTreeSet::new();
    for membership in context_index
        .layer_index
        .selector_memberships
        .iter()
        .chain(&context_index.container_index.selector_memberships)
        .chain(&context_index.scope_index.selector_memberships)
    {
        if class_names.contains(membership.selector_name.as_str()) {
            ids.insert(membership.context_id.clone());
        }
    }
    ids.into_iter().collect()
}

fn layer_binding_for_span(
    context_index: &StyleContextIndexV0,
    span: ParserByteSpanV0,
) -> (Option<String>, Option<i32>) {
    let layer_index = &context_index.layer_index;
    let resolution = layer_ordinal_for_byte_span(layer_index, span.start, span.end);
    let binding = layer_index
        .block_bindings
        .iter()
        .filter(|binding| {
            binding.byte_span.start <= span.start && span.end <= binding.byte_span.end
        })
        .max_by_key(|binding| binding.nesting_depth);
    match resolution {
        LayerBindingResolutionV0::Resolved(ordinal) => (
            binding.map(|binding| binding.canonical_name.clone()),
            ordinal.map(LayerOrdinal::get),
        ),
        LayerBindingResolutionV0::TopologyIncomplete { .. } => (None, None),
    }
}

fn declaration_id_for_source_order(source_order: u32) -> String {
    format!("decl-{source_order}")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cst_join_keeps_live_url_values_and_wire_ids() {
        for source in [
            ".a { background-image: url(a;b.png)!important; background-image: url(clean.png); }",
            ".a { background-image: url(a}b.png)!important; background-image: url(clean.png); }",
        ] {
            let facts = collect_parsed_declaration_facts("fixture.css", source, StyleDialect::Css);
            assert_eq!(facts.len(), 2, "{facts:#?}");
            assert_eq!(facts[0].declaration_id, "decl-0");
            assert_eq!(facts[1].declaration_id, "decl-1");
            assert!(facts[0].important);
            assert!(facts[0].value.starts_with("url(a"));
            assert!(facts[0].value.ends_with("b.png)"));
            assert_eq!(facts[1].value, "url(clean.png)");
        }
    }

    #[test]
    fn cst_join_preserves_ids_above_a_trailing_edit() {
        let before = collect_parsed_declaration_facts(
            "fixture.css",
            ".a { color: red; color: blue; }",
            StyleDialect::Css,
        );
        let after = collect_parsed_declaration_facts(
            "fixture.css",
            ".a { color: red; color: blue; } .later { display: block; }",
            StyleDialect::Css,
        );
        assert_eq!(
            before
                .iter()
                .map(|fact| fact.declaration_id.as_str())
                .collect::<Vec<_>>(),
            ["decl-0", "decl-1"]
        );
        assert_eq!(before, after[..before.len()]);
    }

    #[test]
    fn cst_join_carries_semantic_context_memberships_and_nested_selectors() {
        let source = r#"
@layer theme {
  @container card (inline-size > 20rem) {
    .card { &:hover { color: red; } }
  }
}
"#;
        let facts = collect_parsed_declaration_facts("fixture.scss", source, StyleDialect::Scss);
        assert_eq!(facts.len(), 1, "{facts:#?}");
        assert_eq!(facts[0].selector, ".card:hover");
        assert_eq!(facts[0].layer_name.as_deref(), Some("theme"));
        assert!(facts[0].layer_order.is_some());
        assert!(
            facts[0]
                .semantic_context_ids
                .iter()
                .any(|id| id.starts_with("layer:")),
            "{:?}",
            facts[0].semantic_context_ids
        );
        assert!(
            facts[0]
                .semantic_context_ids
                .iter()
                .any(|id| id.starts_with("container:")),
            "{:?}",
            facts[0].semantic_context_ids
        );
    }

    #[test]
    fn declaration_id_derivation_is_the_existing_wire_spelling() {
        assert_eq!(declaration_id_for_source_order(0), "decl-0");
        assert_eq!(declaration_id_for_source_order(41), "decl-41");
    }
}

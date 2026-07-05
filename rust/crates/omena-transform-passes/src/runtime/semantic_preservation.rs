#[cfg(test)]
use omena_parser::StyleDialect;
#[cfg(test)]
use omena_transform_cst::lower_transform_ir_from_source;
use omena_transform_cst::{IrNodeKindV0, IrNodeV0, TransformIrV0, TransformPassKind};
#[cfg(test)]
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

use crate::model::TransformSemanticPreservationTelemetryV0;

impl TransformSemanticPreservationTelemetryV0 {
    pub(crate) fn record(&mut self, decision: &TransformSemanticPreservationDecisionV0) {
        self.observed_pass_count += 1;
        if decision.preserved {
            self.preserved_pass_count += 1;
        } else {
            self.blocked_pass_count += 1;
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct TransformSemanticPreservationDecisionV0 {
    pub pass_id: &'static str,
    pub preserved: bool,
    pub input_entry_count: usize,
    pub output_entry_count: usize,
    pub mismatch_count: usize,
}

pub(crate) fn semantic_preservation_applies(pass: TransformPassKind) -> bool {
    matches!(
        pass,
        TransformPassKind::EmptyRuleRemoval | TransformPassKind::RuleDeduplication
    )
}

pub(crate) fn compare_semantic_observation_for_pass(
    pass_id: &'static str,
    input_ir: &TransformIrV0,
    output_ir: &TransformIrV0,
) -> TransformSemanticPreservationDecisionV0 {
    let input = semantic_observation(input_ir);
    let output = semantic_observation(output_ir);
    let mismatch_count = semantic_observation_mismatch_count(&input, &output);
    TransformSemanticPreservationDecisionV0 {
        pass_id,
        preserved: mismatch_count == 0,
        input_entry_count: input.len(),
        output_entry_count: output.len(),
        mismatch_count,
    }
}

#[cfg(test)]
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct TransformSemanticPreservationKillRateReportV0 {
    pub schema_version: &'static str,
    pub product: &'static str,
    pub fixture_count: usize,
    pub rejected_count: usize,
    pub required_rejected_count: usize,
    pub non_empty_corpus: bool,
    pub kill_rate_passed: bool,
}

#[cfg(test)]
pub(crate) fn summarize_semantic_preservation_kill_rate_for_fixture_source(
    source: &str,
    dialect: StyleDialect,
) -> Result<TransformSemanticPreservationKillRateReportV0, serde_json::Error> {
    let fixtures = serde_json::from_str::<Vec<TransformSemanticPreservationFixtureV0>>(source)?;
    let mut rejected_count = 0usize;

    for fixture in &fixtures {
        let Some(pass) = transform_pass_kind_from_fixture_id(fixture.pass_id.as_str()) else {
            continue;
        };
        if !semantic_preservation_applies(pass) {
            continue;
        }
        let input_ir = lower_transform_ir_from_source(
            fixture.input.as_str(),
            dialect,
            "omena-transform-passes.semantic-preservation.input",
        );
        let output_ir = lower_transform_ir_from_source(
            fixture.output.as_str(),
            dialect,
            "omena-transform-passes.semantic-preservation.output",
        );
        let decision = compare_semantic_observation_for_pass(pass.id(), &input_ir, &output_ir);
        if !decision.preserved {
            rejected_count += 1;
        }
    }

    let required_rejected_count = fixtures
        .iter()
        .filter(|fixture| fixture.expected_rejected)
        .count();
    Ok(TransformSemanticPreservationKillRateReportV0 {
        schema_version: "0",
        product: "omena-transform-passes.semantic-preservation-kill-rate",
        fixture_count: fixtures.len(),
        rejected_count,
        required_rejected_count,
        non_empty_corpus: !fixtures.is_empty(),
        kill_rate_passed: !fixtures.is_empty() && rejected_count >= required_rejected_count,
    })
}

#[cfg(test)]
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "camelCase")]
struct TransformSemanticPreservationFixtureV0 {
    pass_id: String,
    input: String,
    output: String,
    expected_rejected: bool,
}

#[cfg(test)]
fn transform_pass_kind_from_fixture_id(pass_id: &str) -> Option<TransformPassKind> {
    match pass_id {
        "empty-rule-removal" => Some(TransformPassKind::EmptyRuleRemoval),
        "rule-deduplication" => Some(TransformPassKind::RuleDeduplication),
        _ => None,
    }
}

type SemanticObservationV0 = BTreeMap<SemanticObservationKeyV0, SemanticObservationValueV0>;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
struct SemanticObservationKeyV0 {
    selector_key: String,
    property: String,
    context_key: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct SemanticObservationValueV0 {
    value: String,
    important: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct SemanticDeclarationCandidateV0 {
    key: SemanticObservationKeyV0,
    value: SemanticObservationValueV0,
    source_order: usize,
}

fn semantic_observation(ir: &TransformIrV0) -> SemanticObservationV0 {
    let mut observation = SemanticObservationV0::new();
    let mut candidates = ir
        .nodes
        .iter()
        .filter(|node| !node.deleted && node.kind == IrNodeKindV0::StyleRule)
        .filter_map(|node| semantic_style_rule_candidates(ir, node))
        .flatten()
        .collect::<Vec<_>>();
    candidates.sort_by_key(|candidate| candidate.source_order);

    for candidate in candidates {
        match observation.get(&candidate.key) {
            Some(current) if current.important && !candidate.value.important => {
                continue;
            }
            _ => {
                observation.insert(candidate.key, candidate.value);
            }
        }
    }

    observation
}

fn semantic_style_rule_candidates(
    ir: &TransformIrV0,
    node: &IrNodeV0,
) -> Option<Vec<SemanticDeclarationCandidateV0>> {
    if has_deleted_ancestor(ir, node) || has_style_rule_ancestor(ir, node) {
        return None;
    }
    let selector_key = style_rule_selector_key(ir, node)?;
    if selector_key.eq_ignore_ascii_case(":export") || selector_key.starts_with(":import") {
        return None;
    }
    let context_key = ancestor_context_key(ir, node);
    let mut declarations = node
        .children
        .iter()
        .filter_map(|child_id| ir.nodes.get(child_id.index()))
        .filter(|child| !child.deleted && child.kind == IrNodeKindV0::Declaration)
        .filter_map(|child| semantic_declaration_from_ir(ir, child))
        .collect::<Vec<_>>();
    declarations.sort_by_key(|declaration| declaration.source_order);

    Some(
        declarations
            .into_iter()
            .map(|declaration| SemanticDeclarationCandidateV0 {
                key: SemanticObservationKeyV0 {
                    selector_key: selector_key.clone(),
                    property: declaration.property,
                    context_key: context_key.clone(),
                },
                value: SemanticObservationValueV0 {
                    value: declaration.value,
                    important: declaration.important,
                },
                source_order: declaration.source_order,
            })
            .collect(),
    )
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct SemanticDeclarationV0 {
    property: String,
    value: String,
    important: bool,
    source_order: usize,
}

fn semantic_declaration_from_ir(
    ir: &TransformIrV0,
    node: &IrNodeV0,
) -> Option<SemanticDeclarationV0> {
    if has_deleted_ancestor(ir, node) {
        return None;
    }
    let source = node_text(ir, node)?.trim().trim_end_matches(';').trim();
    if source.is_empty() || contains_nested_block_or_comment(source) {
        return None;
    }
    let colon = declaration_colon_index(source)?;
    let property = source.get(..colon)?.trim();
    let value = source.get(colon + 1..)?.trim();
    if property.is_empty() || value.is_empty() {
        return None;
    }
    let property = if property.starts_with("--") {
        property.to_string()
    } else {
        property.to_ascii_lowercase()
    };
    Some(SemanticDeclarationV0 {
        property,
        value: normalize_declaration_value(value),
        important: declaration_value_is_important(value),
        source_order: node.global_order,
    })
}

fn style_rule_selector_key(ir: &TransformIrV0, node: &IrNodeV0) -> Option<String> {
    let source = node_text(ir, node)?;
    let open = source.find('{')?;
    let selector = source.get(..open)?;
    Some(normalize_selector_key(selector))
}

fn ancestor_context_key(ir: &TransformIrV0, node: &IrNodeV0) -> String {
    let mut ancestors = Vec::new();
    let mut parent = node.parent;
    while let Some(parent_id) = parent {
        let Some(parent_node) = ir.nodes.get(parent_id.index()) else {
            break;
        };
        if parent_node.deleted {
            break;
        }
        if matches!(
            parent_node.kind,
            IrNodeKindV0::AtRule | IrNodeKindV0::StyleRule
        ) && let Some(context) = context_component(ir, parent_node)
        {
            ancestors.push(context);
        }
        parent = parent_node.parent;
    }
    ancestors.reverse();
    ancestors.join("|")
}

fn context_component(ir: &TransformIrV0, node: &IrNodeV0) -> Option<String> {
    let source = node_text(ir, node)?;
    let open = source.find('{').unwrap_or(source.len());
    let prelude = source.get(..open)?.trim();
    if prelude.is_empty() {
        return None;
    }
    Some(normalize_space(prelude))
}

fn semantic_observation_mismatch_count(
    input: &SemanticObservationV0,
    output: &SemanticObservationV0,
) -> usize {
    let missing_or_changed = input
        .iter()
        .filter(|(key, value)| output.get(*key) != Some(*value))
        .count();
    let added = output
        .keys()
        .filter(|key| !input.contains_key(*key))
        .count();
    missing_or_changed + added
}

fn has_deleted_ancestor(ir: &TransformIrV0, node: &IrNodeV0) -> bool {
    let mut parent = node.parent;
    while let Some(parent_id) = parent {
        let Some(parent_node) = ir.nodes.get(parent_id.index()) else {
            return true;
        };
        if parent_node.deleted {
            return true;
        }
        parent = parent_node.parent;
    }
    false
}

fn has_style_rule_ancestor(ir: &TransformIrV0, node: &IrNodeV0) -> bool {
    let mut parent = node.parent;
    while let Some(parent_id) = parent {
        let Some(parent_node) = ir.nodes.get(parent_id.index()) else {
            return false;
        };
        if parent_node.kind == IrNodeKindV0::StyleRule {
            return true;
        }
        parent = parent_node.parent;
    }
    false
}

fn node_text<'a>(ir: &'a TransformIrV0, node: &'a IrNodeV0) -> Option<&'a str> {
    node.canonical_text.as_deref().or_else(|| {
        ir.source_text()
            .get(node.source_span_start..node.source_span_end)
    })
}

fn normalize_selector_key(selector: &str) -> String {
    normalize_space(selector)
}

fn normalize_declaration_value(value: &str) -> String {
    normalize_space(value)
}

fn normalize_space(value: &str) -> String {
    value.split_whitespace().collect::<Vec<_>>().join(" ")
}

fn contains_nested_block_or_comment(source: &str) -> bool {
    let bytes = source.as_bytes();
    let mut index = 0usize;
    let mut quote = None;
    let mut escaped = false;
    while index < bytes.len() {
        let byte = bytes[index];
        if let Some(quote_byte) = quote {
            if escaped {
                escaped = false;
            } else if byte == b'\\' {
                escaped = true;
            } else if byte == quote_byte {
                quote = None;
            }
            index += 1;
            continue;
        }
        if matches!(byte, b'\'' | b'"') {
            quote = Some(byte);
            index += 1;
            continue;
        }
        if matches!(byte, b'{' | b'}') || (byte == b'/' && bytes.get(index + 1) == Some(&b'*')) {
            return true;
        }
        index += 1;
    }
    false
}

fn declaration_colon_index(source: &str) -> Option<usize> {
    let bytes = source.as_bytes();
    let mut index = 0usize;
    let mut quote = None;
    let mut escaped = false;
    let mut paren_depth = 0usize;
    let mut bracket_depth = 0usize;

    while index < bytes.len() {
        let byte = bytes[index];
        if let Some(quote_byte) = quote {
            if escaped {
                escaped = false;
            } else if byte == b'\\' {
                escaped = true;
            } else if byte == quote_byte {
                quote = None;
            }
            index += 1;
            continue;
        }
        match byte {
            b'\'' | b'"' => quote = Some(byte),
            b'(' => paren_depth = paren_depth.saturating_add(1),
            b')' => paren_depth = paren_depth.saturating_sub(1),
            b'[' => bracket_depth = bracket_depth.saturating_add(1),
            b']' => bracket_depth = bracket_depth.saturating_sub(1),
            b':' if paren_depth == 0 && bracket_depth == 0 => return Some(index),
            _ => {}
        }
        index += 1;
    }
    None
}

fn declaration_value_is_important(value: &str) -> bool {
    let bytes = value.as_bytes();
    let mut index = 0usize;
    while index < bytes.len() {
        if bytes[index] == b'!' {
            let rest = value.get(index + 1..).unwrap_or_default().trim_start();
            return rest
                .get(.."important".len())
                .is_some_and(|candidate| candidate.eq_ignore_ascii_case("important"));
        }
        index += 1;
    }
    false
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn observation_ignores_removed_empty_rules() {
        let input = lower_transform_ir_from_source(
            ".a { color: red; }\n.empty {}\n",
            StyleDialect::Css,
            "test",
        );
        let output =
            lower_transform_ir_from_source(".a { color: red; }\n", StyleDialect::Css, "test");
        let decision = compare_semantic_observation_for_pass("empty-rule-removal", &input, &output);

        assert!(decision.preserved);
        assert_eq!(decision.mismatch_count, 0);
        assert_eq!(decision.input_entry_count, 1);
        assert_eq!(decision.output_entry_count, 1);
    }

    #[test]
    fn observation_catches_declared_value_changes() {
        let input = lower_transform_ir_from_source(".a { color: red; }", StyleDialect::Css, "test");
        let output =
            lower_transform_ir_from_source(".a { color: blue; }", StyleDialect::Css, "test");
        let decision = compare_semantic_observation_for_pass("rule-deduplication", &input, &output);

        assert!(!decision.preserved);
        assert_eq!(decision.mismatch_count, 1);
    }

    #[test]
    fn semantic_preservation_broken_translation_corpus_rejects_known_bad_outputs()
    -> Result<(), serde_json::Error> {
        let report = summarize_semantic_preservation_kill_rate_for_fixture_source(
            include_str!("../../fixtures/semantic-preservation/broken-simple.json"),
            StyleDialect::Css,
        )?;

        assert!(report.non_empty_corpus);
        assert_eq!(report.fixture_count, 2);
        assert_eq!(report.required_rejected_count, 2);
        assert_eq!(report.rejected_count, 2);
        assert!(report.kill_rate_passed);
        Ok(())
    }
}

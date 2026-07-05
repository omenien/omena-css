use std::collections::{BTreeMap, VecDeque};

use cstree::text::{TextRange, TextSize};
use omena_cascade::{
    CascadeValue, CustomPropertyEnv, resolve_custom_property_env_least_fixed_point,
    substitute_custom_properties,
};
use omena_parser::{LexedToken, StyleDialect};
use omena_syntax::SyntaxKind;
use omena_transform_cst::{IrNodeIdV0, IrNodeKindV0, IrNodeV0, TransformIrV0};

use crate::runtime::lex_cache::lex_cached as lex;

use crate::domains::{
    css_module_global::{
        CssModuleScopeBlock, CssModuleScopeBlockKind, collect_css_module_scope_blocks,
    },
    css_modules_values::at_rule_block_has_reachable_ordinary_rule,
    custom_property_icss::{
        CustomPropertyIcssExportDeclaration, CustomPropertyIcssExportRule,
        collect_static_custom_property_icss_export_rules, custom_property_icss_export_is_reachable,
    },
    keyframes::{
        KeyframesRuleSlice, collect_keyframes_rules, collect_keyframes_rules_from_ir,
        collect_referenced_keyframe_names, collect_referenced_keyframe_names_from_ir,
        keyframe_name_is_reachable,
    },
    reachability::rule_slice_matches_reachable_class_context,
};
use crate::helpers::{
    blocks::{at_rule_block_start, at_rule_prelude_end_index, rule_block_token_indexes},
    collections::push_unique_string,
    declarations::collect_simple_declarations_in_block,
    identifiers::{is_css_ident_continue, normalize_custom_property_name},
    ir_transaction::{
        TransformIrReplacementKindV0, TransformIrSourceReplacementErrorV0,
        TransformIrSourceReplacementV0, delete_ir_nodes_in_ir,
    },
    rules::{
        SimpleRuleSlice, collect_declaration_ordinary_rule_slices,
        collect_top_level_ordinary_rule_slices,
    },
    source_rewrite::replace_source_ranges,
    tokens::{matching_right_brace_index, skip_whitespace_tokens, token_end, token_start},
    values::{
        matching_function_call_end, parse_whole_function_value_arguments,
        split_top_level_value_arguments,
    },
};
use crate::model::TransformSemanticRemovalCandidate;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct CustomPropertySemanticFactV0 {
    pub(crate) fact_kind: &'static str,
    pub(crate) name: String,
    pub(crate) value: String,
    pub(crate) source_span_start: usize,
    pub(crate) source_span_end: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct CustomPropertyRegistrationRule {
    pub(crate) name: String,
    pub(crate) start: usize,
    pub(crate) end: usize,
    pub(crate) syntax: Option<String>,
    pub(crate) inherits: Option<String>,
    pub(crate) initial_value: Option<String>,
}

pub(crate) fn collect_custom_property_registration_rules(
    tokens: &[omena_parser::LexedToken],
) -> Vec<CustomPropertyRegistrationRule> {
    let mut rules = Vec::new();
    let mut index = 0;

    while index < tokens.len() {
        if tokens[index].kind == SyntaxKind::AtKeyword
            && tokens[index].text.eq_ignore_ascii_case("@property")
            && let Some((rule, next_index)) = parse_custom_property_registration_rule(tokens, index)
        {
            rules.push(rule);
            index = next_index;
            continue;
        }
        index += 1;
    }

    rules
}

fn parse_custom_property_registration_rule(
    tokens: &[omena_parser::LexedToken],
    at_property_index: usize,
) -> Option<(CustomPropertyRegistrationRule, usize)> {
    let name_index = skip_whitespace_tokens(tokens, at_property_index + 1, tokens.len());
    let name = normalize_custom_property_name(tokens.get(name_index)?.text.as_str())?.to_string();
    let block_start_index = at_rule_block_start(tokens, name_index + 1)?;
    let close_index = matching_right_brace_index(tokens, block_start_index)?;
    let declarations = collect_simple_declarations_in_block(tokens, block_start_index, close_index);
    let syntax = declarations
        .iter()
        .find(|declaration| declaration.property == "syntax" && !declaration.important)
        .map(|declaration| declaration.value.clone());
    let inherits = declarations
        .iter()
        .find(|declaration| declaration.property == "inherits" && !declaration.important)
        .map(|declaration| declaration.value.clone());
    let initial_value = declarations
        .into_iter()
        .find(|declaration| declaration.property == "initial-value" && !declaration.important)
        .map(|declaration| declaration.value);

    Some((
        CustomPropertyRegistrationRule {
            name,
            start: token_start(&tokens[at_property_index]),
            end: token_end(&tokens[close_index]),
            syntax,
            inherits,
            initial_value,
        },
        close_index + 1,
    ))
}

pub(crate) fn close_custom_property_dependency_graph(
    roots: Vec<String>,
    dependencies_by_name: &BTreeMap<String, Vec<String>>,
) -> Vec<String> {
    let mut reachable = Vec::new();
    let mut queue = roots.into_iter().collect::<VecDeque<_>>();

    while let Some(name) = queue.pop_front() {
        if reachable.iter().any(|existing| existing == &name) {
            continue;
        }
        reachable.push(name.clone());
        if let Some(dependencies) = dependencies_by_name.get(&name) {
            for dependency in dependencies {
                queue.push_back(dependency.clone());
            }
        }
    }

    reachable.sort();
    reachable
}

pub(crate) fn collect_custom_property_references_in_value(value: &str) -> Vec<String> {
    let mut names = Vec::new();
    let mut index = 0usize;
    let mut quote: Option<char> = None;

    while index < value.len() {
        let Some(ch) = value[index..].chars().next() else {
            break;
        };

        if let Some(quote_ch) = quote {
            index += ch.len_utf8();
            if ch == '\\' {
                if let Some(escaped) = value[index..].chars().next() {
                    index += escaped.len_utf8();
                }
            } else if ch == quote_ch {
                quote = None;
            }
            continue;
        }

        match ch {
            '"' | '\'' => {
                quote = Some(ch);
                index += ch.len_utf8();
            }
            _ if value[index..]
                .get(.."var(".len())
                .is_some_and(|text| text.eq_ignore_ascii_case("var(")) =>
            {
                let left_paren_index = index + "var".len();
                let Some(close_index) = matching_function_call_end(value, left_paren_index) else {
                    index += ch.len_utf8();
                    continue;
                };
                let Some(arguments) =
                    split_top_level_value_arguments(&value[left_paren_index + 1..close_index])
                else {
                    index = close_index + ')'.len_utf8();
                    continue;
                };
                if let [name, fallback @ ..] = arguments.as_slice()
                    && let Some(name) = normalize_custom_property_name(name)
                {
                    push_unique_string(&mut names, name.to_string());
                    for fallback_value in fallback {
                        for fallback_name in
                            collect_custom_property_references_in_value(fallback_value)
                        {
                            push_unique_string(&mut names, fallback_name);
                        }
                    }
                }
                index = close_index + ')'.len_utf8();
            }
            _ => {
                index += ch.len_utf8();
            }
        }
    }

    names
}

pub(crate) fn collect_custom_property_references_in_container_style_query_prelude(
    prelude: &str,
) -> Vec<String> {
    let mut names = Vec::new();
    let mut index = 0usize;
    let mut quote: Option<char> = None;

    while index < prelude.len() {
        let Some(ch) = prelude[index..].chars().next() else {
            break;
        };

        if let Some(quote_ch) = quote {
            index += ch.len_utf8();
            if ch == '\\' {
                if let Some(escaped) = prelude[index..].chars().next() {
                    index += escaped.len_utf8();
                }
            } else if ch == quote_ch {
                quote = None;
            }
            continue;
        }

        match ch {
            '"' | '\'' => {
                quote = Some(ch);
                index += ch.len_utf8();
            }
            _ if css_function_name_starts_at(prelude, index, "style") => {
                let left_paren_index = index + "style".len();
                let Some(close_index) = matching_function_call_end(prelude, left_paren_index)
                else {
                    index += ch.len_utf8();
                    continue;
                };
                collect_custom_property_names_in_style_query(
                    &prelude[left_paren_index + 1..close_index],
                    &mut names,
                );
                index = close_index + ')'.len_utf8();
            }
            _ => {
                index += ch.len_utf8();
            }
        }
    }

    names
}

pub(crate) fn collect_custom_property_roots_from_container_style_query_preludes(
    source: &str,
    tokens: &[omena_parser::LexedToken],
    mut block_is_reachable: impl FnMut(usize, usize) -> bool,
) -> Vec<String> {
    let mut roots = Vec::new();
    let mut index = 0usize;

    while index < tokens.len() {
        if tokens[index].kind != SyntaxKind::AtKeyword
            || !tokens[index].text.eq_ignore_ascii_case("@container")
        {
            index += 1;
            continue;
        }
        let Some(prelude_end_index) = at_rule_prelude_end_index(tokens, index + 1) else {
            break;
        };
        let block_is_reachable = tokens[prelude_end_index].kind == SyntaxKind::LeftBrace
            && matching_right_brace_index(tokens, prelude_end_index)
                .is_some_and(|close_index| block_is_reachable(prelude_end_index, close_index));
        if block_is_reachable {
            let prelude_start = token_end(&tokens[index]);
            let prelude_end = token_start(&tokens[prelude_end_index]);
            for name in collect_custom_property_references_in_container_style_query_prelude(
                &source[prelude_start..prelude_end],
            ) {
                push_unique_string(&mut roots, name);
            }
        }
        index = prelude_end_index.saturating_add(1);
    }

    roots
}

fn collect_custom_property_roots_from_container_style_query_preludes_from_ir(
    ir: &TransformIrV0,
    mut block_is_reachable: impl FnMut(usize, usize) -> bool,
) -> Vec<String> {
    let mut roots = Vec::new();
    for at_rule in collect_custom_property_at_rule_preludes_from_ir(ir) {
        if !at_rule.keyword.eq_ignore_ascii_case("@container") {
            continue;
        }
        let CustomPropertyAtRulePreludeTerminatorV0::Block {
            block_start,
            block_end,
        } = at_rule.terminator
        else {
            continue;
        };
        if block_is_reachable(block_start, block_end) {
            for name in
                collect_custom_property_references_in_container_style_query_prelude(at_rule.prelude)
            {
                push_unique_string(&mut roots, name);
            }
        }
    }
    roots
}

pub(crate) fn tree_shake_css_custom_properties_with_lexer(
    source: &str,
    dialect: StyleDialect,
    reachable_custom_property_names: &[String],
    reachable_keyframe_names: &[String],
    reachable_class_names: &[String],
) -> (String, Vec<TransformSemanticRemovalCandidate>) {
    let (replacements, removals) = collect_tree_shake_css_custom_property_replacements(
        source,
        dialect,
        reachable_custom_property_names,
        reachable_keyframe_names,
        reachable_class_names,
    );
    let ranges = replacements
        .iter()
        .map(|replacement| {
            (
                replacement.source_span_start,
                replacement.source_span_end,
                replacement.replacement.clone(),
            )
        })
        .collect::<Vec<_>>();
    let (output, _) = replace_source_ranges(source, &ranges);
    (output, removals)
}

pub(crate) fn tree_shake_css_custom_properties_with_ir_transaction_on_ir(
    ir: &mut TransformIrV0,
    dialect: StyleDialect,
    reachable_custom_property_names: &[String],
    reachable_keyframe_names: &[String],
    reachable_class_names: &[String],
) -> Result<Vec<TransformSemanticRemovalCandidate>, TransformIrSourceReplacementErrorV0> {
    let (replacements, removals) = collect_tree_shake_css_custom_property_replacements_from_ir(
        ir,
        dialect,
        reachable_custom_property_names,
        reachable_keyframe_names,
        reachable_class_names,
    );
    let replacements = non_overlapping_custom_property_replacements(replacements);
    let node_deletion_ids = custom_property_deletion_node_ids(ir, replacements.as_slice())?;
    if !node_deletion_ids.is_empty() {
        delete_ir_nodes_in_ir(
            ir,
            "tree-shake-custom-property",
            node_deletion_ids.as_slice(),
        )?;
    }
    Ok(removals)
}

pub(crate) fn collect_tree_shake_css_custom_property_removals_from_ir(
    ir: &TransformIrV0,
    dialect: StyleDialect,
    reachable_custom_property_names: &[String],
    reachable_keyframe_names: &[String],
    reachable_class_names: &[String],
) -> Vec<TransformSemanticRemovalCandidate> {
    let (_, removals) = collect_tree_shake_css_custom_property_replacements_from_ir(
        ir,
        dialect,
        reachable_custom_property_names,
        reachable_keyframe_names,
        reachable_class_names,
    );
    removals
}

pub(crate) fn collect_css_custom_property_semantic_facts_from_ir(
    ir: &TransformIrV0,
) -> Vec<CustomPropertySemanticFactV0> {
    let mut facts = collect_custom_property_registration_rules_from_ir(ir)
        .into_iter()
        .map(|registration| CustomPropertySemanticFactV0 {
            fact_kind: "custom-property-registration",
            name: registration.name,
            value: format!(
                "syntax={};inherits={};initial={}",
                registration.syntax.unwrap_or_default(),
                registration.inherits.unwrap_or_default(),
                registration.initial_value.unwrap_or_default()
            ),
            source_span_start: registration.start,
            source_span_end: registration.end,
        })
        .collect::<Vec<_>>();

    facts.extend(
        collect_static_custom_property_icss_export_rules_from_ir(ir)
            .into_iter()
            .flat_map(|rule| {
                rule.declarations
                    .into_iter()
                    .map(|declaration| CustomPropertySemanticFactV0 {
                        fact_kind: "custom-property-export",
                        name: declaration.export_name,
                        value: declaration.value,
                        source_span_start: declaration.start,
                        source_span_end: declaration.end,
                    })
                    .collect::<Vec<_>>()
            }),
    );

    facts.sort_by(|left, right| {
        (left.fact_kind, left.name.as_str(), left.source_span_start).cmp(&(
            right.fact_kind,
            right.name.as_str(),
            right.source_span_start,
        ))
    });
    facts
}

fn collect_tree_shake_css_custom_property_replacements(
    source: &str,
    dialect: StyleDialect,
    reachable_custom_property_names: &[String],
    reachable_keyframe_names: &[String],
    reachable_class_names: &[String],
) -> (
    Vec<TransformIrSourceReplacementV0>,
    Vec<TransformSemanticRemovalCandidate>,
) {
    let lexed = lex(source, dialect);
    let tokens = lexed.tokens();
    let Some(referenced_names) = collect_reachable_custom_property_names(
        source,
        tokens,
        reachable_custom_property_names,
        reachable_keyframe_names,
        reachable_class_names,
    ) else {
        return (Vec::new(), Vec::new());
    };

    let mut removals = Vec::new();
    let mut export_removal_replacements = Vec::new();
    for registration in collect_custom_property_registration_rules(tokens) {
        if !referenced_names
            .iter()
            .any(|name| name == &registration.name)
        {
            removals.push(TransformSemanticRemovalCandidate {
                symbol_kind: "customPropertyRegistration",
                name: registration.name,
                source_span_start: registration.start,
                source_span_end: registration.end,
                reason: "custom-property registration was absent from the closed-style-world reachable custom-property set",
            });
        }
    }
    let scope_blocks = collect_css_module_scope_blocks(source, tokens);
    let keyframes = collect_keyframes_rules(tokens);
    let reachable_keyframe_names = collect_reachable_keyframe_names(
        source,
        tokens,
        reachable_keyframe_names,
        reachable_class_names,
    );
    let export_rules = collect_static_custom_property_icss_export_rules(source, tokens);
    for rule in &export_rules {
        let unreachable_exports = rule
            .declarations
            .iter()
            .filter(|declaration| {
                !custom_property_icss_export_is_reachable(
                    &declaration.export_name,
                    reachable_custom_property_names,
                )
            })
            .collect::<Vec<_>>();
        if unreachable_exports.is_empty() {
            continue;
        }
        if unreachable_exports.len() == rule.declarations.len() {
            export_removal_replacements.push(TransformIrSourceReplacementV0 {
                source_span_start: rule.start,
                source_span_end: rule.end,
                replacement: String::new(),
                kind: TransformIrReplacementKindV0::IcssExportName,
            });
        } else {
            export_removal_replacements.extend(unreachable_exports.iter().map(|declaration| {
                TransformIrSourceReplacementV0 {
                    source_span_start: declaration.start,
                    source_span_end: declaration.end,
                    replacement: String::new(),
                    kind: TransformIrReplacementKindV0::Declaration,
                }
            }));
        }
        removals.extend(
            unreachable_exports
                .iter()
                .map(|declaration| TransformSemanticRemovalCandidate {
                    symbol_kind: "customPropertyIcssExport",
                    name: declaration.export_name.clone(),
                    source_span_start: declaration.start,
                    source_span_end: declaration.end,
                    reason: "ICSS export declaration was absent from the closed-style-world reachable custom-property export set",
                }),
        );
    }
    for rule in collect_declaration_ordinary_rule_slices(source, tokens) {
        let rule_is_reachable = custom_property_rule_is_reachable(
            &rule,
            &scope_blocks,
            &keyframes,
            reachable_keyframe_names.as_deref(),
            reachable_class_names,
        );
        let Some((block_start_index, block_end_index)) =
            rule_block_token_indexes(tokens, rule.block_start, rule.block_end)
        else {
            continue;
        };
        for declaration in
            collect_simple_declarations_in_block(tokens, block_start_index, block_end_index)
        {
            if !declaration.property.starts_with("--") {
                continue;
            }
            let name_is_referenced = referenced_names
                .iter()
                .any(|name| name == &declaration.property);
            if !rule_is_reachable || !name_is_referenced {
                removals.push(TransformSemanticRemovalCandidate {
                    symbol_kind: "customProperty",
                    name: declaration.property,
                    source_span_start: declaration.start,
                    source_span_end: declaration.end,
                    reason: if rule_is_reachable {
                        "custom property declaration was absent from transitive var() references and the closed-style-world reachable custom-property set"
                    } else {
                        "custom property declaration belonged to an unreachable closed-style-world rule"
                    },
                });
            }
        }
    }

    let mut ranges = removals
        .iter()
        .filter(|removal| removal.symbol_kind != "customPropertyIcssExport")
        .map(|removal| TransformIrSourceReplacementV0 {
            source_span_start: removal.source_span_start,
            source_span_end: removal.source_span_end,
            replacement: String::new(),
            kind: match removal.symbol_kind {
                "customPropertyRegistration" => TransformIrReplacementKindV0::AtRule,
                _ => TransformIrReplacementKindV0::CustomPropertyDeclaration,
            },
        })
        .collect::<Vec<_>>();
    ranges.extend(export_removal_replacements);
    (ranges, removals)
}

fn collect_tree_shake_css_custom_property_replacements_from_ir(
    ir: &TransformIrV0,
    dialect: StyleDialect,
    reachable_custom_property_names: &[String],
    reachable_keyframe_names: &[String],
    reachable_class_names: &[String],
) -> (
    Vec<TransformIrSourceReplacementV0>,
    Vec<TransformSemanticRemovalCandidate>,
) {
    let Some(referenced_names) = collect_reachable_custom_property_names_from_ir(
        ir,
        dialect,
        reachable_custom_property_names,
        reachable_keyframe_names,
        reachable_class_names,
    ) else {
        return (Vec::new(), Vec::new());
    };

    let mut removals = Vec::new();
    let mut export_removal_replacements = Vec::new();
    for registration in collect_custom_property_registration_rules_from_ir(ir) {
        if !referenced_names
            .iter()
            .any(|name| name == &registration.name)
        {
            removals.push(TransformSemanticRemovalCandidate {
                symbol_kind: "customPropertyRegistration",
                name: registration.name,
                source_span_start: registration.start,
                source_span_end: registration.end,
                reason: "custom-property registration was absent from the closed-style-world reachable custom-property set",
            });
        }
    }
    let scope_blocks = collect_css_module_scope_blocks_from_ir(ir);
    let keyframes = collect_keyframes_rules_from_ir(ir);
    let reachable_keyframe_names = collect_reachable_keyframe_names_from_ir(
        ir,
        reachable_keyframe_names,
        reachable_class_names,
    );
    let export_rules = collect_static_custom_property_icss_export_rules_from_ir(ir);
    for rule in &export_rules {
        let unreachable_exports = rule
            .declarations
            .iter()
            .filter(|declaration| {
                !custom_property_icss_export_is_reachable(
                    &declaration.export_name,
                    reachable_custom_property_names,
                )
            })
            .collect::<Vec<_>>();
        if unreachable_exports.is_empty() {
            continue;
        }
        if unreachable_exports.len() == rule.declarations.len() {
            export_removal_replacements.push(TransformIrSourceReplacementV0 {
                source_span_start: rule.start,
                source_span_end: rule.end,
                replacement: String::new(),
                kind: TransformIrReplacementKindV0::IcssExportName,
            });
        } else {
            export_removal_replacements.extend(unreachable_exports.iter().map(|declaration| {
                TransformIrSourceReplacementV0 {
                    source_span_start: declaration.start,
                    source_span_end: declaration.end,
                    replacement: String::new(),
                    kind: TransformIrReplacementKindV0::Declaration,
                }
            }));
        }
        removals.extend(
            unreachable_exports
                .iter()
                .map(|declaration| TransformSemanticRemovalCandidate {
                    symbol_kind: "customPropertyIcssExport",
                    name: declaration.export_name.clone(),
                    source_span_start: declaration.start,
                    source_span_end: declaration.end,
                    reason: "ICSS export declaration was absent from the closed-style-world reachable custom-property export set",
                }),
        );
    }
    for rule in collect_declaration_ordinary_rule_slices_from_ir(ir) {
        let rule_is_reachable = custom_property_rule_is_reachable(
            &rule,
            &scope_blocks,
            &keyframes,
            reachable_keyframe_names.as_deref(),
            reachable_class_names,
        );
        push_custom_property_rule_removals_from_declarations(
            &mut removals,
            collect_simple_declarations_from_ir(ir, &rule),
            rule_is_reachable,
            &referenced_names,
        );
    }
    for rule in collect_keyframe_declaration_rule_slices_from_ir(ir, dialect, &keyframes) {
        let rule_is_reachable = custom_property_rule_is_reachable(
            &rule,
            &scope_blocks,
            &keyframes,
            reachable_keyframe_names.as_deref(),
            reachable_class_names,
        );
        push_custom_property_rule_removals_from_declarations(
            &mut removals,
            collect_simple_declarations_from_keyframe_slice(ir.source_text(), dialect, &rule),
            rule_is_reachable,
            &referenced_names,
        );
    }

    let mut ranges = removals
        .iter()
        .filter(|removal| removal.symbol_kind != "customPropertyIcssExport")
        .map(|removal| TransformIrSourceReplacementV0 {
            source_span_start: removal.source_span_start,
            source_span_end: removal.source_span_end,
            replacement: String::new(),
            kind: match removal.symbol_kind {
                "customPropertyRegistration" => TransformIrReplacementKindV0::AtRule,
                _ => TransformIrReplacementKindV0::CustomPropertyDeclaration,
            },
        })
        .collect::<Vec<_>>();
    ranges.extend(export_removal_replacements);
    (ranges, removals)
}

fn push_custom_property_rule_removals_from_declarations(
    removals: &mut Vec<TransformSemanticRemovalCandidate>,
    declarations: Vec<CustomPropertyDeclarationIrViewV0>,
    rule_is_reachable: bool,
    referenced_names: &[String],
) {
    for declaration in declarations {
        if !declaration.property.starts_with("--") {
            continue;
        }
        let name_is_referenced = referenced_names
            .iter()
            .any(|name| name == &declaration.property);
        if rule_is_reachable && name_is_referenced {
            continue;
        }
        if removals.iter().any(|removal| {
            removal.symbol_kind == "customProperty"
                && removal.source_span_start == declaration.start
                && removal.source_span_end == declaration.end
        }) {
            continue;
        }
        removals.push(TransformSemanticRemovalCandidate {
            symbol_kind: "customProperty",
            name: declaration.property,
            source_span_start: declaration.start,
            source_span_end: declaration.end,
            reason: if rule_is_reachable {
                "custom property declaration was absent from transitive var() references and the closed-style-world reachable custom-property set"
            } else {
                "custom property declaration belonged to an unreachable closed-style-world rule"
            },
        });
    }
}

fn non_overlapping_custom_property_replacements(
    mut replacements: Vec<TransformIrSourceReplacementV0>,
) -> Vec<TransformIrSourceReplacementV0> {
    replacements.sort_by_key(|replacement| replacement.source_span_start);
    let mut retained = Vec::new();
    let mut cursor = 0usize;

    for replacement in replacements {
        if replacement.source_span_start >= cursor {
            cursor = replacement.source_span_end;
            retained.push(replacement);
        }
    }

    retained
}

fn custom_property_deletion_node_ids(
    ir: &TransformIrV0,
    replacements: &[TransformIrSourceReplacementV0],
) -> Result<Vec<IrNodeIdV0>, TransformIrSourceReplacementErrorV0> {
    replacements
        .iter()
        .map(|replacement| custom_property_deletion_node_id(ir, replacement))
        .collect()
}

fn custom_property_deletion_node_id(
    ir: &TransformIrV0,
    replacement: &TransformIrSourceReplacementV0,
) -> Result<IrNodeIdV0, TransformIrSourceReplacementErrorV0> {
    let Some(kind) = custom_property_deletion_node_kind(replacement.kind) else {
        return Err(TransformIrSourceReplacementErrorV0::MissingNode {
            source_span_start: replacement.source_span_start,
            source_span_end: replacement.source_span_end,
            kind: replacement.kind,
            candidate_spans: Vec::new(),
        });
    };
    ir.nodes
        .iter()
        .find(|node| {
            !node.deleted
                && node.kind == kind
                && node.source_span_start == replacement.source_span_start
                && node.source_span_end == replacement.source_span_end
        })
        .map(|node| node.node_id)
        .ok_or_else(|| TransformIrSourceReplacementErrorV0::MissingNode {
            source_span_start: replacement.source_span_start,
            source_span_end: replacement.source_span_end,
            kind: replacement.kind,
            candidate_spans: ir
                .nodes
                .iter()
                .filter(|node| !node.deleted && node.kind == kind)
                .map(|node| (node.source_span_start, node.source_span_end))
                .collect(),
        })
}

const fn custom_property_deletion_node_kind(
    kind: TransformIrReplacementKindV0,
) -> Option<IrNodeKindV0> {
    match kind {
        TransformIrReplacementKindV0::AtRule => Some(IrNodeKindV0::AtRule),
        TransformIrReplacementKindV0::Declaration
        | TransformIrReplacementKindV0::CustomPropertyDeclaration => {
            Some(IrNodeKindV0::Declaration)
        }
        TransformIrReplacementKindV0::IcssExportName => Some(IrNodeKindV0::StyleRule),
        _ => None,
    }
}

fn custom_property_rule_is_reachable(
    rule: &SimpleRuleSlice,
    scope_blocks: &[CssModuleScopeBlock],
    keyframes: &[KeyframesRuleSlice],
    reachable_keyframe_names: Option<&[String]>,
    reachable_class_names: &[String],
) -> bool {
    if let Some(keyframe_name) = enclosing_keyframe_name_for_rule(rule, keyframes)
        && let Some(reachable_keyframe_names) = reachable_keyframe_names
    {
        return keyframe_name_is_reachable(keyframe_name, reachable_keyframe_names);
    }

    rule_slice_matches_reachable_class_context(rule, scope_blocks, reachable_class_names)
}

fn collect_reachable_custom_property_names(
    source: &str,
    tokens: &[omena_parser::LexedToken],
    external_roots: &[String],
    external_keyframe_roots: &[String],
    reachable_class_names: &[String],
) -> Option<Vec<String>> {
    let mut root_names = Vec::new();
    let mut dependencies_by_name = BTreeMap::<String, Vec<String>>::new();
    let scope_blocks = collect_css_module_scope_blocks(source, tokens);
    let keyframes = collect_keyframes_rules(tokens);
    let reachable_keyframe_names = collect_reachable_keyframe_names(
        source,
        tokens,
        external_keyframe_roots,
        reachable_class_names,
    );

    for name in external_roots {
        if let Some(name) = normalize_custom_property_name(name) {
            push_unique_string(&mut root_names, name.to_string());
        }
    }
    for name in collect_custom_property_roots_from_container_style_query_preludes(
        source,
        tokens,
        |block_start_index, block_end_index| {
            at_rule_block_has_reachable_ordinary_rule(
                source,
                tokens,
                block_start_index,
                block_end_index,
                reachable_class_names,
                &scope_blocks,
            )
        },
    ) {
        push_unique_string(&mut root_names, name);
    }
    for name in collect_custom_property_roots_from_reachable_at_rule_preludes(
        source,
        tokens,
        |block_start_index, block_end_index| {
            at_rule_block_has_reachable_ordinary_rule(
                source,
                tokens,
                block_start_index,
                block_end_index,
                reachable_class_names,
                &scope_blocks,
            )
        },
    ) {
        push_unique_string(&mut root_names, name);
    }
    for name in collect_custom_property_roots_from_descriptor_at_rules(tokens) {
        push_unique_string(&mut root_names, name);
    }
    for rule in collect_static_custom_property_icss_export_rules(source, tokens) {
        for declaration in rule.declarations {
            if !custom_property_icss_export_is_reachable(&declaration.export_name, external_roots) {
                continue;
            }
            for name in collect_custom_property_references_in_value(&declaration.value) {
                push_unique_string(&mut root_names, name);
            }
        }
    }

    for registration in collect_custom_property_registration_rules(tokens) {
        let Some(initial_value) = registration.initial_value else {
            continue;
        };
        let dependencies = dependencies_by_name.entry(registration.name).or_default();
        for name in collect_custom_property_references_in_value(&initial_value) {
            push_unique_string(dependencies, name);
        }
    }

    for rule in collect_declaration_ordinary_rule_slices(source, tokens) {
        if rule.selector.trim().eq_ignore_ascii_case(":export") {
            continue;
        }
        if let Some(keyframe_name) = enclosing_keyframe_name_for_rule(&rule, &keyframes)
            && let Some(reachable_keyframe_names) = reachable_keyframe_names.as_ref()
            && !keyframe_name_is_reachable(keyframe_name, reachable_keyframe_names)
        {
            continue;
        }
        let rule_is_reachable =
            rule_slice_matches_reachable_class_context(&rule, &scope_blocks, reachable_class_names);
        let Some((block_start_index, block_end_index)) =
            rule_block_token_indexes(tokens, rule.block_start, rule.block_end)
        else {
            continue;
        };
        for declaration in
            collect_simple_declarations_in_block(tokens, block_start_index, block_end_index)
        {
            if declaration.property.starts_with("--") {
                if !rule_is_reachable {
                    continue;
                }
                let referenced_names =
                    collect_custom_property_references_in_value(&declaration.value);
                let dependencies = dependencies_by_name
                    .entry(declaration.property)
                    .or_default();
                for name in referenced_names {
                    push_unique_string(dependencies, name);
                }
            } else if rule_is_reachable {
                let referenced_names =
                    collect_custom_property_references_in_value(&declaration.value);
                for name in referenced_names {
                    push_unique_string(&mut root_names, name);
                }
            }
        }
    }

    Some(close_custom_property_dependency_graph(
        root_names,
        &dependencies_by_name,
    ))
}

fn collect_reachable_custom_property_names_from_ir(
    ir: &TransformIrV0,
    dialect: StyleDialect,
    external_roots: &[String],
    external_keyframe_roots: &[String],
    reachable_class_names: &[String],
) -> Option<Vec<String>> {
    let mut root_names = Vec::new();
    let mut dependencies_by_name = BTreeMap::<String, Vec<String>>::new();
    let scope_blocks = collect_css_module_scope_blocks_from_ir(ir);
    let keyframes = collect_keyframes_rules_from_ir(ir);
    let reachable_keyframe_names = collect_reachable_keyframe_names_from_ir(
        ir,
        external_keyframe_roots,
        reachable_class_names,
    );

    for name in external_roots {
        if let Some(name) = normalize_custom_property_name(name) {
            push_unique_string(&mut root_names, name.to_string());
        }
    }
    for name in collect_custom_property_roots_from_container_style_query_preludes_from_ir(
        ir,
        |block_start, block_end| {
            at_rule_block_has_reachable_ordinary_rule_from_ir(
                ir,
                block_start,
                block_end,
                reachable_class_names,
                &scope_blocks,
            )
        },
    ) {
        push_unique_string(&mut root_names, name);
    }
    for name in collect_custom_property_roots_from_reachable_at_rule_preludes_from_ir(
        ir,
        |block_start, block_end| {
            at_rule_block_has_reachable_ordinary_rule_from_ir(
                ir,
                block_start,
                block_end,
                reachable_class_names,
                &scope_blocks,
            )
        },
    ) {
        push_unique_string(&mut root_names, name);
    }
    for name in collect_custom_property_roots_from_descriptor_at_rules_from_ir(ir) {
        push_unique_string(&mut root_names, name);
    }
    for rule in collect_static_custom_property_icss_export_rules_from_ir(ir) {
        for declaration in rule.declarations {
            if !custom_property_icss_export_is_reachable(&declaration.export_name, external_roots) {
                continue;
            }
            for name in collect_custom_property_references_in_value(&declaration.value) {
                push_unique_string(&mut root_names, name);
            }
        }
    }

    for registration in collect_custom_property_registration_rules_from_ir(ir) {
        let Some(initial_value) = registration.initial_value else {
            continue;
        };
        let dependencies = dependencies_by_name.entry(registration.name).or_default();
        for name in collect_custom_property_references_in_value(&initial_value) {
            push_unique_string(dependencies, name);
        }
    }

    for rule in collect_declaration_ordinary_rule_slices_from_ir(ir) {
        if rule.selector.trim().eq_ignore_ascii_case(":export") {
            continue;
        }
        if let Some(keyframe_name) = enclosing_keyframe_name_for_rule(&rule, &keyframes)
            && let Some(reachable_keyframe_names) = reachable_keyframe_names.as_ref()
            && !keyframe_name_is_reachable(keyframe_name, reachable_keyframe_names)
        {
            continue;
        }
        let rule_is_reachable =
            rule_slice_matches_reachable_class_context(&rule, &scope_blocks, reachable_class_names);
        for declaration in collect_simple_declarations_from_ir(ir, &rule) {
            if declaration.property.starts_with("--") {
                if !rule_is_reachable {
                    continue;
                }
                let referenced_names =
                    collect_custom_property_references_in_value(&declaration.value);
                let dependencies = dependencies_by_name
                    .entry(declaration.property)
                    .or_default();
                for name in referenced_names {
                    push_unique_string(dependencies, name);
                }
            } else if rule_is_reachable {
                let referenced_names =
                    collect_custom_property_references_in_value(&declaration.value);
                for name in referenced_names {
                    push_unique_string(&mut root_names, name);
                }
            }
        }
    }
    collect_reachable_custom_property_names_from_keyframes_from_ir(
        CustomPropertyKeyframeReachabilityInputV0 {
            ir,
            dialect,
            keyframes: &keyframes,
            reachable_keyframe_names: reachable_keyframe_names.as_deref(),
            reachable_class_names,
            scope_blocks: &scope_blocks,
            root_names: &mut root_names,
            dependencies_by_name: &mut dependencies_by_name,
        },
    );

    Some(close_custom_property_dependency_graph(
        root_names,
        &dependencies_by_name,
    ))
}

struct CustomPropertyKeyframeReachabilityInputV0<'a> {
    ir: &'a TransformIrV0,
    dialect: StyleDialect,
    keyframes: &'a [KeyframesRuleSlice],
    reachable_keyframe_names: Option<&'a [String]>,
    reachable_class_names: &'a [String],
    scope_blocks: &'a [CssModuleScopeBlock],
    root_names: &'a mut Vec<String>,
    dependencies_by_name: &'a mut BTreeMap<String, Vec<String>>,
}

fn collect_reachable_custom_property_names_from_keyframes_from_ir(
    input: CustomPropertyKeyframeReachabilityInputV0<'_>,
) {
    for keyframe in input.keyframes {
        if let Some(reachable_keyframe_names) = input.reachable_keyframe_names
            && !keyframe_name_is_reachable(&keyframe.name, reachable_keyframe_names)
        {
            continue;
        }
        for rule in collect_keyframe_declaration_rule_slices_from_ir(
            input.ir,
            input.dialect,
            std::slice::from_ref(keyframe),
        ) {
            let rule_is_reachable = rule_slice_matches_reachable_class_context(
                &rule,
                input.scope_blocks,
                input.reachable_class_names,
            );
            for declaration in collect_simple_declarations_from_keyframe_slice(
                input.ir.source_text(),
                input.dialect,
                &rule,
            ) {
                if declaration.property.starts_with("--") {
                    if !rule_is_reachable {
                        continue;
                    }
                    let referenced_names =
                        collect_custom_property_references_in_value(&declaration.value);
                    let dependencies = input
                        .dependencies_by_name
                        .entry(declaration.property)
                        .or_default();
                    for name in referenced_names {
                        push_unique_string(dependencies, name);
                    }
                } else if rule_is_reachable {
                    let referenced_names =
                        collect_custom_property_references_in_value(&declaration.value);
                    for name in referenced_names {
                        push_unique_string(input.root_names, name);
                    }
                }
            }
        }
    }
}

fn collect_reachable_keyframe_names(
    source: &str,
    tokens: &[omena_parser::LexedToken],
    external_roots: &[String],
    reachable_class_names: &[String],
) -> Option<Vec<String>> {
    let mut names = collect_referenced_keyframe_names(source, tokens, reachable_class_names)?;
    for name in external_roots {
        push_unique_string(&mut names, name.clone());
    }
    Some(names)
}

fn collect_reachable_keyframe_names_from_ir(
    ir: &TransformIrV0,
    external_roots: &[String],
    reachable_class_names: &[String],
) -> Option<Vec<String>> {
    let mut names = collect_referenced_keyframe_names_from_ir(ir, reachable_class_names)?;
    for name in external_roots {
        push_unique_string(&mut names, name.clone());
    }
    Some(names)
}

fn enclosing_keyframe_name_for_rule<'a>(
    rule: &SimpleRuleSlice,
    keyframes: &'a [KeyframesRuleSlice],
) -> Option<&'a str> {
    keyframes
        .iter()
        .find(|keyframe| rule.start >= keyframe.start && rule.end <= keyframe.end)
        .map(|keyframe| keyframe.name.as_str())
}

fn collect_custom_property_roots_from_reachable_at_rule_preludes(
    source: &str,
    tokens: &[omena_parser::LexedToken],
    mut block_is_reachable: impl FnMut(usize, usize) -> bool,
) -> Vec<String> {
    let mut roots = Vec::new();
    let mut index = 0usize;

    while index < tokens.len() {
        if tokens[index].kind != SyntaxKind::AtKeyword
            || !at_rule_prelude_can_reference_custom_properties(&tokens[index].text)
        {
            index += 1;
            continue;
        }
        let Some(prelude_end_index) = at_rule_prelude_end_index(tokens, index + 1) else {
            break;
        };
        let prelude_can_keep_roots = match tokens[prelude_end_index].kind {
            SyntaxKind::LeftBrace => matching_right_brace_index(tokens, prelude_end_index)
                .is_some_and(|close_index| block_is_reachable(prelude_end_index, close_index)),
            SyntaxKind::Semicolon => true,
            _ => false,
        };
        if prelude_can_keep_roots {
            let prelude_start = token_end(&tokens[index]);
            let prelude_end = token_start(&tokens[prelude_end_index]);
            for name in
                collect_custom_property_references_in_value(&source[prelude_start..prelude_end])
            {
                push_unique_string(&mut roots, name);
            }
        }
        index = prelude_end_index.saturating_add(1);
    }

    roots
}

fn collect_custom_property_roots_from_reachable_at_rule_preludes_from_ir(
    ir: &TransformIrV0,
    mut block_is_reachable: impl FnMut(usize, usize) -> bool,
) -> Vec<String> {
    let mut roots = Vec::new();
    for at_rule in collect_custom_property_at_rule_preludes_from_ir(ir) {
        if !at_rule_prelude_can_reference_custom_properties(at_rule.keyword) {
            continue;
        }
        let prelude_can_keep_roots = match at_rule.terminator {
            CustomPropertyAtRulePreludeTerminatorV0::Block {
                block_start,
                block_end,
            } => block_is_reachable(block_start, block_end),
            CustomPropertyAtRulePreludeTerminatorV0::Semicolon => true,
            CustomPropertyAtRulePreludeTerminatorV0::Unknown => false,
        };
        if prelude_can_keep_roots {
            for name in collect_custom_property_references_in_value(at_rule.prelude) {
                push_unique_string(&mut roots, name);
            }
        }
    }
    roots
}

fn collect_custom_property_roots_from_descriptor_at_rules(
    tokens: &[omena_parser::LexedToken],
) -> Vec<String> {
    let mut roots = Vec::new();
    let mut index = 0usize;

    while index < tokens.len() {
        if tokens[index].kind != SyntaxKind::AtKeyword
            || !descriptor_at_rule_can_reference_custom_properties(&tokens[index].text)
        {
            index += 1;
            continue;
        }
        let Some(block_start_index) = at_rule_prelude_end_index(tokens, index + 1) else {
            break;
        };
        if tokens[block_start_index].kind != SyntaxKind::LeftBrace {
            index = block_start_index.saturating_add(1);
            continue;
        }
        let Some(block_end_index) = matching_right_brace_index(tokens, block_start_index) else {
            break;
        };

        for declaration in
            collect_simple_declarations_in_block(tokens, block_start_index, block_end_index)
        {
            for name in collect_custom_property_references_in_value(&declaration.value) {
                push_unique_string(&mut roots, name);
            }
        }
        index = block_end_index + 1;
    }

    roots
}

fn collect_custom_property_roots_from_descriptor_at_rules_from_ir(
    ir: &TransformIrV0,
) -> Vec<String> {
    let mut roots = Vec::new();
    for node in ir
        .nodes
        .iter()
        .filter(|node| !node.deleted && node.kind == IrNodeKindV0::AtRule)
    {
        let Some((keyword, _, _)) = ir_at_rule_keyword_bounds(ir.source_text(), node) else {
            continue;
        };
        if !descriptor_at_rule_can_reference_custom_properties(keyword) {
            continue;
        }
        let Some((block_start, block_end)) = at_rule_body_bounds_from_ir(ir.source_text(), node)
        else {
            continue;
        };
        for declaration in collect_simple_declarations_between_from_ir(ir, block_start, block_end) {
            for name in collect_custom_property_references_in_value(&declaration.value) {
                push_unique_string(&mut roots, name);
            }
        }
    }
    roots
}

fn descriptor_at_rule_can_reference_custom_properties(text: &str) -> bool {
    matches!(
        text.to_ascii_lowercase().as_str(),
        "@color-profile" | "@counter-style" | "@font-face" | "@font-palette-values" | "@page"
    )
}

fn at_rule_prelude_can_reference_custom_properties(text: &str) -> bool {
    matches!(
        text.to_ascii_lowercase().as_str(),
        "@media" | "@supports" | "@container" | "@custom-media" | "@scope"
    )
}

fn collect_custom_property_registration_rules_from_ir(
    ir: &TransformIrV0,
) -> Vec<CustomPropertyRegistrationRule> {
    let mut rules = ir
        .nodes
        .iter()
        .filter(|node| !node.deleted && node.kind == IrNodeKindV0::AtRule)
        .filter_map(|node| custom_property_registration_rule_from_ir(ir, node))
        .collect::<Vec<_>>();
    rules.sort_by_key(|rule| (rule.start, rule.end));
    rules
}

fn custom_property_registration_rule_from_ir(
    ir: &TransformIrV0,
    node: &IrNodeV0,
) -> Option<CustomPropertyRegistrationRule> {
    let source = ir.source_text();
    let (keyword, keyword_start, keyword_end) = ir_at_rule_keyword_bounds(source, node)?;
    if !keyword.eq_ignore_ascii_case("@property") {
        return None;
    }
    let CustomPropertyAtRulePreludeTerminatorV0::Block {
        block_start,
        block_end,
    } = at_rule_prelude_terminator_from_source(source, keyword_end, node.source_span_end)?
    else {
        return None;
    };
    let name = normalize_custom_property_name(source.get(keyword_end..block_start)?.trim())?;
    let declarations = collect_simple_declarations_between_from_ir(ir, block_start, block_end);
    let syntax = declarations
        .iter()
        .find(|declaration| declaration.property == "syntax" && !declaration.important)
        .map(|declaration| declaration.value.clone());
    let inherits = declarations
        .iter()
        .find(|declaration| declaration.property == "inherits" && !declaration.important)
        .map(|declaration| declaration.value.clone());
    let initial_value = declarations
        .into_iter()
        .find(|declaration| declaration.property == "initial-value" && !declaration.important)
        .map(|declaration| declaration.value);
    Some(CustomPropertyRegistrationRule {
        name: name.to_string(),
        start: keyword_start,
        end: node.source_span_end,
        syntax,
        inherits,
        initial_value,
    })
}

fn collect_static_custom_property_icss_export_rules_from_ir(
    ir: &TransformIrV0,
) -> Vec<CustomPropertyIcssExportRule> {
    collect_declaration_ordinary_rule_slices_from_ir(ir)
        .into_iter()
        .filter(|rule| rule.selector.trim().eq_ignore_ascii_case(":export"))
        .filter_map(|rule| {
            let declarations = collect_simple_declarations_from_ir(ir, &rule)
                .into_iter()
                .filter(|declaration| {
                    !collect_custom_property_references_in_value(&declaration.value).is_empty()
                })
                .map(|declaration| CustomPropertyIcssExportDeclaration {
                    export_name: declaration.property,
                    value: declaration.value,
                    start: declaration.start,
                    end: declaration.end,
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

fn collect_css_module_scope_blocks_from_ir(ir: &TransformIrV0) -> Vec<CssModuleScopeBlock> {
    let mut blocks = ir
        .nodes
        .iter()
        .filter(|node| !node.deleted && node.kind == IrNodeKindV0::StyleRule)
        .filter_map(|node| css_module_scope_block_from_ir(ir, node))
        .collect::<Vec<_>>();
    blocks.sort_by_key(|block| (block.start, block.end));
    blocks
}

fn css_module_scope_block_from_ir(
    ir: &TransformIrV0,
    node: &IrNodeV0,
) -> Option<CssModuleScopeBlock> {
    let selector = style_rule_selector_from_ir(ir, node)?;
    let kind = if selector.eq_ignore_ascii_case(":local") {
        CssModuleScopeBlockKind::Local
    } else if selector.eq_ignore_ascii_case(":global") {
        CssModuleScopeBlockKind::Global
    } else {
        return None;
    };
    let (body_start, body_end) = style_rule_body_bounds_from_ir(ir.source_text(), node)?;
    Some(CssModuleScopeBlock {
        start: node.source_span_start,
        end: node.source_span_end,
        body_start,
        body_end,
        kind,
    })
}

fn at_rule_block_has_reachable_ordinary_rule_from_ir(
    ir: &TransformIrV0,
    block_start: usize,
    block_end: usize,
    reachable_class_names: &[String],
    scope_blocks: &[CssModuleScopeBlock],
) -> bool {
    collect_declaration_ordinary_rule_slices_from_ir(ir)
        .iter()
        .any(|rule| {
            rule.context_start >= block_start
                && rule.context_end <= block_end
                && rule_slice_matches_reachable_class_context(
                    rule,
                    scope_blocks,
                    reachable_class_names,
                )
        })
}

fn collect_declaration_ordinary_rule_slices_from_ir(ir: &TransformIrV0) -> Vec<SimpleRuleSlice> {
    let mut rules = ir
        .nodes
        .iter()
        .filter(|node| !node.deleted && node.kind == IrNodeKindV0::StyleRule)
        .filter_map(|node| declaration_ordinary_rule_slice_from_ir(ir, node))
        .collect::<Vec<_>>();
    rules.sort_by_key(|rule| (rule.start, rule.end));
    rules
}

fn declaration_ordinary_rule_slice_from_ir(
    ir: &TransformIrV0,
    node: &IrNodeV0,
) -> Option<SimpleRuleSlice> {
    if node.children.iter().any(|child_id| {
        ir.nodes.get(child_id.index()).is_some_and(|child| {
            !child.deleted && matches!(child.kind, IrNodeKindV0::StyleRule | IrNodeKindV0::AtRule)
        })
    }) {
        return None;
    }
    let source = ir.source_text();
    if rule_prelude_contains_comment_before_selector(source, node.source_span_start) {
        return None;
    }
    let selector = style_rule_selector_from_ir(ir, node)?.trim().to_string();
    let (body_start, body_end) = style_rule_body_bounds_from_ir(source, node)?;
    let block = source.get(body_start..body_end)?.trim().to_string();
    if selector.is_empty() || block.is_empty() || source_text_contains_comment(&block) {
        return None;
    }
    let (context_start, context_end) = style_rule_context_from_ir(ir, node);
    Some(SimpleRuleSlice {
        selector,
        block,
        start: node.source_span_start,
        end: node.source_span_end,
        block_start: body_start.saturating_sub(1),
        block_end: body_end,
        context_start,
        context_end,
    })
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct CustomPropertyDeclarationIrViewV0 {
    property: String,
    value: String,
    important: bool,
    start: usize,
    end: usize,
}

fn collect_simple_declarations_from_ir(
    ir: &TransformIrV0,
    rule: &SimpleRuleSlice,
) -> Vec<CustomPropertyDeclarationIrViewV0> {
    collect_simple_declarations_between_from_ir(ir, rule.block_start, rule.block_end)
}

fn collect_simple_declarations_between_from_ir(
    ir: &TransformIrV0,
    block_start: usize,
    block_end: usize,
) -> Vec<CustomPropertyDeclarationIrViewV0> {
    let mut declarations = ir
        .nodes
        .iter()
        .filter(|node| {
            !node.deleted
                && node.kind == IrNodeKindV0::Declaration
                && node.source_span_start >= block_start
                && node.source_span_end <= block_end
        })
        .filter_map(|node| simple_declaration_from_ir(ir, node))
        .collect::<Vec<_>>();
    declarations.sort_by_key(|declaration| declaration.start);
    declarations
}

fn collect_keyframe_declaration_rule_slices_from_ir(
    ir: &TransformIrV0,
    dialect: StyleDialect,
    keyframes: &[KeyframesRuleSlice],
) -> Vec<SimpleRuleSlice> {
    let mut rules = Vec::new();
    for keyframe in keyframes {
        let Some(tokens) = lex_ir_source_slice_with_offset(
            ir.source_text(),
            dialect,
            keyframe.start,
            keyframe.end,
        ) else {
            continue;
        };
        rules.extend(
            collect_declaration_ordinary_rule_slices(ir.source_text(), &tokens)
                .into_iter()
                .filter(|rule| {
                    enclosing_keyframe_name_for_rule(rule, std::slice::from_ref(keyframe)).is_some()
                }),
        );
    }
    rules.sort_by_key(|rule| (rule.start, rule.end));
    rules
}

fn collect_simple_declarations_from_keyframe_slice(
    source: &str,
    dialect: StyleDialect,
    rule: &SimpleRuleSlice,
) -> Vec<CustomPropertyDeclarationIrViewV0> {
    let Some(tokens) = lex_ir_source_slice_with_offset(source, dialect, rule.start, rule.end)
    else {
        return Vec::new();
    };
    let Some((block_start_index, block_end_index)) =
        rule_block_token_indexes(&tokens, rule.block_start, rule.block_end)
    else {
        return Vec::new();
    };
    collect_simple_declarations_in_block(&tokens, block_start_index, block_end_index)
        .into_iter()
        .map(|declaration| CustomPropertyDeclarationIrViewV0 {
            property: declaration.property,
            value: declaration.value,
            important: declaration.important,
            start: declaration.start,
            end: declaration.end,
        })
        .collect()
}

fn lex_ir_source_slice_with_offset(
    source: &str,
    dialect: StyleDialect,
    source_span_start: usize,
    source_span_end: usize,
) -> Option<Vec<LexedToken>> {
    let slice = source.get(source_span_start..source_span_end)?;
    let lexed = lex(slice, dialect);
    Some(
        lexed
            .tokens()
            .iter()
            .map(|token| LexedToken {
                kind: token.kind,
                range: TextRange::new(
                    TextSize::from((token_start(token) + source_span_start) as u32),
                    TextSize::from((token_end(token) + source_span_start) as u32),
                ),
                text: token.text.clone(),
            })
            .collect(),
    )
}

fn simple_declaration_from_ir(
    ir: &TransformIrV0,
    node: &IrNodeV0,
) -> Option<CustomPropertyDeclarationIrViewV0> {
    let source = ir
        .source_text()
        .get(node.source_span_start..node.source_span_end)?
        .trim()
        .trim_end_matches(';')
        .trim();
    if source.is_empty() || source_text_contains_comment(source) {
        return None;
    }
    let colon = source.find(':')?;
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
    Some(CustomPropertyDeclarationIrViewV0 {
        property,
        value: value.to_string(),
        important: value
            .split_whitespace()
            .any(|part| part.eq_ignore_ascii_case("!important")),
        start: node.source_span_start,
        end: node.source_span_end,
    })
}

fn style_rule_context_from_ir(ir: &TransformIrV0, node: &IrNodeV0) -> (usize, usize) {
    let Some(parent_id) = node.parent else {
        return (0, ir.source_text().len());
    };
    let Some(parent) = ir.nodes.get(parent_id.index()) else {
        return (0, ir.source_text().len());
    };
    let Some((body_start, body_end)) = style_rule_body_bounds_from_ir(ir.source_text(), parent)
    else {
        return (0, ir.source_text().len());
    };
    (body_start.saturating_sub(1), body_end.saturating_add(1))
}

fn style_rule_selector_from_ir<'source>(
    ir: &'source TransformIrV0,
    node: &IrNodeV0,
) -> Option<&'source str> {
    let source = ir.source_text();
    let rule_source = source.get(node.source_span_start..node.source_span_end)?;
    let open = rule_source.find('{')?;
    source
        .get(node.source_span_start..node.source_span_start + open)
        .map(str::trim)
}

fn style_rule_body_bounds_from_ir(source: &str, node: &IrNodeV0) -> Option<(usize, usize)> {
    let rule_source = source.get(node.source_span_start..node.source_span_end)?;
    let open = rule_source.find('{')?;
    let close = rule_source.rfind('}')?;
    if open >= close {
        return None;
    }
    Some((
        node.source_span_start.checked_add(open + 1)?,
        node.source_span_start.checked_add(close)?,
    ))
}

fn at_rule_body_bounds_from_ir(source: &str, node: &IrNodeV0) -> Option<(usize, usize)> {
    let node_source = source.get(node.source_span_start..node.source_span_end)?;
    let open = node_source.find('{')?;
    let close = node_source.rfind('}')?;
    if open >= close {
        return None;
    }
    Some((
        node.source_span_start.checked_add(open + 1)?,
        node.source_span_start.checked_add(close)?,
    ))
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum CustomPropertyAtRulePreludeTerminatorV0 {
    Block {
        block_start: usize,
        block_end: usize,
    },
    Semicolon,
    Unknown,
}

struct CustomPropertyAtRulePreludeIrViewV0<'source> {
    keyword: &'source str,
    prelude: &'source str,
    prelude_start: usize,
    terminator: CustomPropertyAtRulePreludeTerminatorV0,
}

fn collect_custom_property_at_rule_preludes_from_ir(
    ir: &TransformIrV0,
) -> Vec<CustomPropertyAtRulePreludeIrViewV0<'_>> {
    let mut at_rules = ir
        .nodes
        .iter()
        .filter(|node| !node.deleted && node.kind == IrNodeKindV0::AtRule)
        .filter_map(|node| custom_property_at_rule_prelude_from_ir(ir, node))
        .collect::<Vec<_>>();
    at_rules.sort_by_key(|at_rule| at_rule.prelude_start);
    at_rules
}

fn custom_property_at_rule_prelude_from_ir<'source>(
    ir: &'source TransformIrV0,
    node: &IrNodeV0,
) -> Option<CustomPropertyAtRulePreludeIrViewV0<'source>> {
    let source = ir.source_text();
    let (keyword, _, keyword_end) = ir_at_rule_keyword_bounds(source, node)?;
    let terminator =
        at_rule_prelude_terminator_from_source(source, keyword_end, node.source_span_end)
            .unwrap_or(CustomPropertyAtRulePreludeTerminatorV0::Unknown);
    let prelude_end = match terminator {
        CustomPropertyAtRulePreludeTerminatorV0::Block { block_start, .. } => block_start,
        CustomPropertyAtRulePreludeTerminatorV0::Semicolon
        | CustomPropertyAtRulePreludeTerminatorV0::Unknown => node.source_span_end,
    };
    let prelude = source.get(keyword_end..prelude_end)?;
    Some(CustomPropertyAtRulePreludeIrViewV0 {
        keyword,
        prelude,
        prelude_start: keyword_end,
        terminator,
    })
}

fn ir_at_rule_keyword_bounds<'source>(
    source: &'source str,
    node: &IrNodeV0,
) -> Option<(&'source str, usize, usize)> {
    let node_source = source.get(node.source_span_start..node.source_span_end)?;
    let leading_offset = node_source
        .len()
        .saturating_sub(node_source.trim_start().len());
    let keyword_start = node.source_span_start.checked_add(leading_offset)?;
    let at_rule_source = source.get(keyword_start..node.source_span_end)?;
    let keyword_len = at_rule_source
        .find(|ch: char| ch.is_whitespace() || matches!(ch, '{' | '(' | ';'))
        .unwrap_or(at_rule_source.len());
    let keyword_end = keyword_start.checked_add(keyword_len)?;
    Some((
        source.get(keyword_start..keyword_end)?,
        keyword_start,
        keyword_end,
    ))
}

fn at_rule_prelude_terminator_from_source(
    source: &str,
    start: usize,
    end: usize,
) -> Option<CustomPropertyAtRulePreludeTerminatorV0> {
    let bytes = source.as_bytes();
    let mut index = start;
    let mut quote = None;
    let mut escaped = false;
    let mut in_comment = false;
    let mut paren_depth = 0usize;
    let mut bracket_depth = 0usize;

    while index < end {
        let byte = *bytes.get(index)?;
        if in_comment {
            if byte == b'*' && bytes.get(index + 1) == Some(&b'/') {
                in_comment = false;
                index += 2;
            } else {
                index += 1;
            }
            continue;
        }
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
        if byte == b'/' && bytes.get(index + 1) == Some(&b'*') {
            in_comment = true;
            index += 2;
            continue;
        }
        match byte {
            b'\'' | b'"' => quote = Some(byte),
            b'(' => paren_depth += 1,
            b')' => paren_depth = paren_depth.saturating_sub(1),
            b'[' => bracket_depth += 1,
            b']' => bracket_depth = bracket_depth.saturating_sub(1),
            b'{' if paren_depth == 0 && bracket_depth == 0 => {
                let close = source.get(index..end)?.rfind('}')?;
                return Some(CustomPropertyAtRulePreludeTerminatorV0::Block {
                    block_start: index,
                    block_end: index + close + 1,
                });
            }
            b';' if paren_depth == 0 && bracket_depth == 0 => {
                return Some(CustomPropertyAtRulePreludeTerminatorV0::Semicolon);
            }
            _ => {}
        }
        index += 1;
    }

    None
}

fn rule_prelude_contains_comment_before_selector(source: &str, selector_start: usize) -> bool {
    let prelude_start = source
        .get(..selector_start)
        .and_then(|prefix| prefix.rfind([';', '{', '}']))
        .map_or(0, |offset| offset.saturating_add(1));
    source
        .get(prelude_start..selector_start)
        .is_some_and(source_text_contains_comment)
}

fn source_text_contains_comment(source: &str) -> bool {
    source.as_bytes().windows(2).any(|bytes| bytes == b"/*")
}

fn collect_custom_property_names_in_style_query(query: &str, names: &mut Vec<String>) {
    let mut index = 0usize;
    let mut quote: Option<char> = None;

    while index < query.len() {
        let Some(ch) = query[index..].chars().next() else {
            break;
        };

        if let Some(quote_ch) = quote {
            index += ch.len_utf8();
            if ch == '\\' {
                if let Some(escaped) = query[index..].chars().next() {
                    index += escaped.len_utf8();
                }
            } else if ch == quote_ch {
                quote = None;
            }
            continue;
        }

        match ch {
            '"' | '\'' => {
                quote = Some(ch);
                index += ch.len_utf8();
            }
            '-' if query[index..].starts_with("--") => {
                let name_end = custom_property_name_end(query, index + "--".len());
                if name_end > index + "--".len()
                    && let Some(name) = normalize_custom_property_name(&query[index..name_end])
                {
                    push_unique_string(names, name.to_string());
                }
                index = name_end;
            }
            _ => {
                index += ch.len_utf8();
            }
        }
    }
}
fn custom_property_name_end(value: &str, mut index: usize) -> usize {
    while index < value.len() {
        let Some(ch) = value[index..].chars().next() else {
            break;
        };
        if !is_css_ident_continue(ch) {
            break;
        }
        index += ch.len_utf8();
    }
    index
}

pub(crate) fn substitute_static_css_custom_properties_with_lexer(
    source: &str,
    dialect: StyleDialect,
) -> (String, usize) {
    let lexed = lex(source, dialect);
    let tokens = lexed.tokens();
    let env_rules = collect_top_level_ordinary_rule_slices(source, tokens);
    let env = resolve_custom_property_env_least_fixed_point(
        &collect_static_root_custom_property_env(tokens, &env_rules),
    );
    if env.is_empty() {
        return (source.to_string(), 0);
    }

    let mut replacements = Vec::new();
    replacements.extend(collect_static_custom_property_at_rule_prelude_replacements(
        source, tokens, &env,
    ));
    let mut index = 0;
    while index < tokens.len() {
        let Some(close_index) = (tokens[index].kind == SyntaxKind::LeftBrace)
            .then(|| matching_right_brace_index(tokens, index))
            .flatten()
        else {
            index += 1;
            continue;
        };
        for declaration in collect_simple_declarations_in_block(tokens, index, close_index) {
            if declaration.property.starts_with("--") {
                continue;
            }
            let Some(resolved_value) =
                substitute_static_custom_property_references_in_value(&declaration.value, &env)
            else {
                continue;
            };
            replacements.push((
                declaration.start,
                declaration.end,
                format!("{}: {resolved_value};", declaration.property),
            ));
        }
        index += 1;
    }

    replace_source_ranges(source, &replacements)
}

fn collect_static_custom_property_at_rule_prelude_replacements(
    source: &str,
    tokens: &[omena_parser::LexedToken],
    env: &CustomPropertyEnv,
) -> Vec<(usize, usize, String)> {
    let mut replacements = Vec::new();
    let mut index = 0usize;

    while index < tokens.len() {
        if tokens[index].kind != SyntaxKind::AtKeyword
            || !at_rule_prelude_can_reference_custom_properties(&tokens[index].text)
        {
            index += 1;
            continue;
        }
        let Some(prelude_end_index) = at_rule_prelude_end_index(tokens, index + 1) else {
            break;
        };
        if !matches!(
            tokens[prelude_end_index].kind,
            SyntaxKind::LeftBrace | SyntaxKind::Semicolon
        ) {
            index = prelude_end_index.saturating_add(1);
            continue;
        }
        let start = token_end(&tokens[index]);
        let end = token_start(&tokens[prelude_end_index]);
        if start < end
            && let Some(resolved) =
                substitute_static_custom_property_references_in_value(&source[start..end], env)
        {
            replacements.push((start, end, resolved));
        }
        index = prelude_end_index.saturating_add(1);
    }

    replacements
}

fn substitute_static_custom_property_references_in_value(
    value: &str,
    env: &CustomPropertyEnv,
) -> Option<String> {
    let mut output = String::with_capacity(value.len());
    let mut cursor = 0usize;
    let mut index = 0usize;
    let mut quote: Option<char> = None;
    let mut changed = false;

    while index < value.len() {
        let ch = value[index..].chars().next()?;

        if let Some(quote_ch) = quote {
            index += ch.len_utf8();
            if ch == '\\' {
                let escaped = value[index..].chars().next()?;
                index += escaped.len_utf8();
            } else if ch == quote_ch {
                quote = None;
            }
            continue;
        }

        match ch {
            '"' | '\'' => {
                quote = Some(ch);
                index += ch.len_utf8();
            }
            _ if value[index..]
                .get(.."var(".len())
                .is_some_and(|text| text.eq_ignore_ascii_case("var(")) =>
            {
                let left_paren_index = index + "var".len();
                let Some(close_index) = matching_function_call_end(value, left_paren_index) else {
                    index += ch.len_utf8();
                    continue;
                };
                let Some(arguments) =
                    split_top_level_value_arguments(&value[left_paren_index + 1..close_index])
                else {
                    index += ch.len_utf8();
                    continue;
                };
                let Some(var_value) = parse_static_var_arguments(&arguments) else {
                    index += ch.len_utf8();
                    continue;
                };
                let resolved_value = substitute_custom_properties(&var_value, env);
                let Some(resolved_value) = render_static_cascade_value(&resolved_value) else {
                    index += ch.len_utf8();
                    continue;
                };
                output.push_str(&value[cursor..index]);
                output.push_str(&resolved_value);
                index = close_index + ')'.len_utf8();
                cursor = index;
                changed = true;
            }
            _ => {
                index += ch.len_utf8();
            }
        }
    }

    if !changed {
        return None;
    }
    output.push_str(&value[cursor..]);
    Some(output)
}

pub(crate) fn collect_static_root_custom_property_env(
    tokens: &[omena_parser::LexedToken],
    rules: &[SimpleRuleSlice],
) -> CustomPropertyEnv {
    let mut env = CustomPropertyEnv::new();
    let mut blocked_names = Vec::new();
    let registrations = collect_custom_property_registration_rules(tokens);

    for rule in rules {
        if rule.selector == ":root" {
            continue;
        }
        let Some((block_start_index, block_end_index)) =
            rule_block_token_indexes(tokens, rule.block_start, rule.block_end)
        else {
            continue;
        };
        for declaration in
            collect_simple_declarations_in_block(tokens, block_start_index, block_end_index)
        {
            if declaration.property.starts_with("--")
                && !blocked_names.contains(&declaration.property)
            {
                blocked_names.push(declaration.property);
            }
        }
    }

    for rule in rules {
        if rule.selector != ":root" {
            continue;
        }
        let Some((block_start_index, block_end_index)) =
            rule_block_token_indexes(tokens, rule.block_start, rule.block_end)
        else {
            continue;
        };
        for declaration in
            collect_simple_declarations_in_block(tokens, block_start_index, block_end_index)
        {
            if !declaration.property.starts_with("--") {
                continue;
            }
            if declaration.important {
                env.remove(&declaration.property);
                if !blocked_names.contains(&declaration.property) {
                    blocked_names.push(declaration.property);
                }
                continue;
            }
            if blocked_names.contains(&declaration.property) {
                continue;
            }
            if env.contains_key(&declaration.property) {
                env.remove(&declaration.property);
                blocked_names.push(declaration.property);
                continue;
            }
            let Some(value) = parse_static_custom_property_env_value(&declaration.value) else {
                env.remove(&declaration.property);
                if !blocked_names.contains(&declaration.property) {
                    blocked_names.push(declaration.property);
                }
                continue;
            };
            env.insert(declaration.property, value);
        }
    }

    let mut registration_names = Vec::new();
    for registration in registrations {
        if blocked_names.contains(&registration.name) {
            continue;
        }
        if registration_names.contains(&registration.name) {
            env.remove(&registration.name);
            blocked_names.push(registration.name);
            continue;
        }
        registration_names.push(registration.name.clone());
        if env.contains_key(&registration.name) {
            continue;
        }
        let Some(initial_value) = registration.initial_value else {
            continue;
        };
        let Some(value) = parse_static_custom_property_env_value(&initial_value) else {
            continue;
        };
        env.insert(registration.name, value);
    }

    env
}

pub(crate) fn parse_static_custom_property_env_value(value: &str) -> Option<CascadeValue> {
    if let Some(value) = parse_css_wide_custom_property_env_value(value) {
        return Some(value);
    }
    if contains_runtime_dependent_css_function(value) {
        return None;
    }
    parse_static_var_value(value)
        .or_else(|| parse_static_composite_custom_property_env_value(value))
}

fn parse_css_wide_custom_property_env_value(value: &str) -> Option<CascadeValue> {
    match value.trim().to_ascii_lowercase().as_str() {
        "initial" => Some(CascadeValue::Initial),
        "inherit" | "unset" | "revert" | "revert-layer" => Some(CascadeValue::Inherit),
        _ => None,
    }
}

fn contains_runtime_dependent_css_function(value: &str) -> bool {
    let mut index = 0usize;
    let mut quote: Option<char> = None;

    while index < value.len() {
        let Some(ch) = value[index..].chars().next() else {
            break;
        };

        if let Some(quote_ch) = quote {
            index += ch.len_utf8();
            if ch == '\\' {
                if let Some(escaped) = value[index..].chars().next() {
                    index += escaped.len_utf8();
                }
            } else if ch == quote_ch {
                quote = None;
            }
            continue;
        }

        match ch {
            '"' | '\'' => {
                quote = Some(ch);
                index += ch.len_utf8();
            }
            _ if css_function_name_starts_at(value, index, "env")
                || css_function_name_starts_at(value, index, "attr") =>
            {
                return true;
            }
            _ => {
                index += ch.len_utf8();
            }
        }
    }

    false
}

fn css_function_name_starts_at(value: &str, index: usize, function_name: &str) -> bool {
    let Some(name) = value.get(index..index + function_name.len()) else {
        return false;
    };
    if !name.eq_ignore_ascii_case(function_name) {
        return false;
    }
    if !value[index + function_name.len()..].starts_with('(') {
        return false;
    }
    if index == 0 {
        return true;
    }
    let Some(previous) = value[..index].chars().next_back() else {
        return true;
    };
    !is_css_ident_continue(previous)
}

fn parse_static_composite_custom_property_env_value(value: &str) -> Option<CascadeValue> {
    let mut parts = Vec::new();
    let mut cursor = 0;
    let mut index = 0;
    let mut quote = None;
    let mut found_var = false;

    while index < value.len() {
        let Some(ch) = value[index..].chars().next() else {
            break;
        };
        if let Some(quote_ch) = quote {
            index += ch.len_utf8();
            if ch == '\\' {
                if let Some(next_ch) = value[index..].chars().next() {
                    index += next_ch.len_utf8();
                }
            } else if ch == quote_ch {
                quote = None;
            }
            continue;
        }

        match ch {
            '"' | '\'' => {
                quote = Some(ch);
                index += ch.len_utf8();
            }
            _ if value[index..]
                .get(.."var(".len())
                .is_some_and(|text| text.eq_ignore_ascii_case("var(")) =>
            {
                let left_paren_index = index + "var".len();
                let Some(close_index) = matching_function_call_end(value, left_paren_index) else {
                    index += ch.len_utf8();
                    continue;
                };
                let Some(arguments) =
                    split_top_level_value_arguments(&value[left_paren_index + 1..close_index])
                else {
                    index = close_index + ')'.len_utf8();
                    continue;
                };
                let Some(var_value) = parse_static_var_arguments(&arguments) else {
                    index += ch.len_utf8();
                    continue;
                };
                if cursor < index {
                    parts.push(CascadeValue::Literal(value[cursor..index].to_string()));
                }
                parts.push(var_value);
                index = close_index + ')'.len_utf8();
                cursor = index;
                found_var = true;
            }
            _ => {
                index += ch.len_utf8();
            }
        }
    }

    if !found_var {
        return Some(CascadeValue::Literal(value.to_string()));
    }
    if cursor < value.len() {
        parts.push(CascadeValue::Literal(value[cursor..].to_string()));
    }
    Some(CascadeValue::Composite(parts))
}

fn parse_static_var_value(value: &str) -> Option<CascadeValue> {
    let arguments = parse_whole_function_value_arguments(value, "var")?;
    parse_static_var_arguments(&arguments)
}

fn parse_static_var_arguments(arguments: &[String]) -> Option<CascadeValue> {
    let (name, fallback_arguments) = arguments.split_first()?;
    if !name.starts_with("--") {
        return None;
    }
    if fallback_arguments.is_empty() {
        return Some(CascadeValue::Var {
            name: name.to_string(),
            fallback: None,
        });
    }

    let fallback = parse_static_custom_property_env_value(&fallback_arguments.join(", "))?;
    Some(CascadeValue::Var {
        name: name.to_string(),
        fallback: Some(Box::new(fallback)),
    })
}

fn render_static_cascade_value(value: &CascadeValue) -> Option<String> {
    match value {
        CascadeValue::Literal(value) => Some(value.clone()),
        CascadeValue::Composite(parts) => {
            let mut output = String::new();
            for part in parts {
                output.push_str(&render_static_cascade_value(part)?);
            }
            Some(output)
        }
        CascadeValue::Var { .. }
        | CascadeValue::Initial
        | CascadeValue::Inherit
        | CascadeValue::GuaranteedInvalid
        | CascadeValue::Unset => None,
    }
}

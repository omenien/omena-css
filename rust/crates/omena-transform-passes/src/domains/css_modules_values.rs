//! CSS Modules `@value` resolution and reachability transform helpers.
//!
//! This domain owns static local value substitution, imported value resolution,
//! and closed-world value tree-shaking for the transform runtime.

use std::collections::{BTreeMap, VecDeque};

use cstree::text::{TextRange, TextSize};
use omena_parser::{LexedToken, StyleDialect};
use omena_syntax::SyntaxKind;
use omena_transform_cst::{IrNodeIdV0, IrNodeKindV0, IrNodeV0, TransformIrV0};

use crate::runtime::lex_cache::lex_cached as lex;

use crate::{
    domains::{
        color::parse_static_srgb_color,
        css_module_global::{CssModuleScopeBlock, collect_css_module_scope_blocks},
        keyframes::{
            KeyframesRuleSlice, collect_keyframes_rules, collect_keyframes_rules_from_ir,
            collect_referenced_keyframe_names, collect_referenced_keyframe_names_from_ir,
        },
        number::parse_numeric_value_with_unit,
        reachability::rule_slice_matches_reachable_class_context,
    },
    helpers::{
        blocks::{
            at_rule_prelude_end_index, previous_significant_token_kind, rule_block_token_indexes,
        },
        collections::push_unique_string,
        css_modules_imports::{
            StaticCssModulesValueImportStatement,
            collect_static_css_modules_value_import_statements,
        },
        css_modules_values::{
            StaticCssModulesValueDefinition, collect_static_local_css_modules_value_definitions,
        },
        declarations::collect_simple_declarations_in_block,
        ir_transaction::{
            TransformIrReplacementKindV0, TransformIrSourceReplacementErrorV0,
            TransformIrSourceReplacementV0, delete_ir_nodes_in_ir, replace_ir_nodes_in_ir,
        },
        rules::{SimpleRuleSlice, collect_declaration_ordinary_rule_slices},
        source_rewrite::replace_source_ranges,
        tokens::{
            is_comment_token, matching_right_brace_index, next_non_comment_token_kind, token_end,
            token_start,
        },
    },
    model::{TransformCssModuleValueResolutionV0, TransformSemanticRemovalCandidate},
};

pub(crate) fn resolve_static_css_modules_values_with_lexer(
    source: &str,
    dialect: StyleDialect,
    resolutions: &[TransformCssModuleValueResolutionV0],
) -> (String, usize) {
    let lexed = lex(source, dialect);
    let tokens = lexed.tokens();
    let definitions = collect_static_local_css_modules_value_definitions(tokens);
    let unique_definitions_by_name =
        unique_static_css_modules_value_definitions_by_name(&definitions);
    let resolved_definitions = unique_definitions_by_name
        .keys()
        .filter_map(|name| {
            let definition = unique_definitions_by_name.get(name)?;
            let resolved_value = resolve_static_css_modules_value_definition(
                name,
                dialect,
                &unique_definitions_by_name,
                &mut Vec::new(),
            )?;
            Some((*definition, resolved_value))
        })
        .collect::<Vec<_>>();
    if resolved_definitions.is_empty() && resolutions.is_empty() {
        return (source.to_string(), 0);
    }

    let mut replacements = resolved_definitions
        .iter()
        .map(|(definition, _)| (definition.start, definition.end, String::new()))
        .collect::<Vec<_>>();
    let resolved_definitions_by_name = resolved_definitions
        .iter()
        .map(|(definition, resolved_value)| (definition.name.clone(), resolved_value.clone()))
        .chain(resolutions.iter().map(|resolution| {
            (
                resolution.local_name.clone(),
                resolution.resolved_value.clone(),
            )
        }))
        .collect::<BTreeMap<_, _>>();
    replacements.extend(collect_static_css_modules_value_import_replacements(
        tokens,
        &resolved_definitions_by_name,
    ));
    replacements.extend(collect_static_css_modules_value_query_prelude_replacements(
        tokens,
        &resolved_definitions_by_name,
    ));
    let mut index = 0;
    while index < tokens.len() {
        if tokens[index].kind == SyntaxKind::LeftBrace
            && let Some(close_index) = matching_right_brace_index(tokens, index)
        {
            for declaration in collect_simple_declarations_in_block(tokens, index, close_index) {
                let Some(resolved_value) = substitute_resolved_css_modules_value_references(
                    &declaration.value,
                    dialect,
                    &resolved_definitions_by_name,
                ) else {
                    continue;
                };
                replacements.push((
                    declaration.start,
                    declaration.end,
                    format!("{}: {resolved_value};", declaration.property),
                ));
            }
            index += 1;
            continue;
        }
        index += 1;
    }

    replacements.sort_by_key(|(start, _, _)| *start);
    let mut output = String::with_capacity(source.len());
    let mut cursor = 0;
    let mut mutation_count = 0;
    for (start, end, replacement) in &replacements {
        if *start < cursor {
            continue;
        }
        if *start > cursor {
            output.push_str(&source[cursor..*start]);
        }
        output.push_str(replacement);
        cursor = *end;
        mutation_count += 1;
    }
    if cursor < source.len() {
        output.push_str(&source[cursor..]);
    }

    (output, mutation_count)
}

pub fn resolve_static_css_modules_local_value_resolutions_from_source(
    source: &str,
    dialect: StyleDialect,
) -> Vec<TransformCssModuleValueResolutionV0> {
    let lexed = lex(source, dialect);
    let tokens = lexed.tokens();
    let definitions = collect_static_local_css_modules_value_definitions(tokens);
    let unique_definitions_by_name =
        unique_static_css_modules_value_definitions_by_name(&definitions);

    let mut resolutions = unique_definitions_by_name
        .keys()
        .filter_map(|name| {
            let resolved_value = resolve_static_css_modules_value_definition(
                name,
                dialect,
                &unique_definitions_by_name,
                &mut Vec::new(),
            )?;
            Some(TransformCssModuleValueResolutionV0 {
                local_name: name.clone(),
                resolved_value,
            })
        })
        .collect::<Vec<_>>();
    resolutions.sort_by(|left, right| left.local_name.cmp(&right.local_name));
    resolutions
}

fn unique_static_css_modules_value_definitions_by_name(
    definitions: &[StaticCssModulesValueDefinition],
) -> BTreeMap<String, &StaticCssModulesValueDefinition> {
    let mut count_by_name = BTreeMap::<String, usize>::new();
    for definition in definitions {
        *count_by_name.entry(definition.name.clone()).or_default() += 1;
    }

    definitions
        .iter()
        .filter(|definition| count_by_name.get(&definition.name) == Some(&1))
        .map(|definition| (definition.name.clone(), definition))
        .collect()
}

fn collect_static_css_modules_value_import_replacements(
    tokens: &[omena_parser::LexedToken],
    resolved_definitions_by_name: &BTreeMap<String, String>,
) -> Vec<(usize, usize, String)> {
    collect_static_css_modules_value_import_statements(tokens)
        .into_iter()
        .filter(|statement| {
            statement
                .local_names
                .iter()
                .any(|name| resolved_definitions_by_name.contains_key(name))
        })
        .map(|statement| {
            let unresolved_binding_texts = statement
                .bindings
                .iter()
                .filter(|binding| !resolved_definitions_by_name.contains_key(&binding.local_name))
                .map(|binding| binding.binding_text.as_str())
                .collect::<Vec<_>>();
            let replacement = if unresolved_binding_texts.is_empty() {
                String::new()
            } else {
                format!(
                    "@value {} {};",
                    unresolved_binding_texts.join(", "),
                    statement.from_clause
                )
            };
            (statement.start, statement.end, replacement)
        })
        .collect()
}

fn collect_static_css_modules_value_query_prelude_replacements(
    tokens: &[omena_parser::LexedToken],
    resolved_definitions_by_name: &BTreeMap<String, String>,
) -> Vec<(usize, usize, String)> {
    let mut replacements = Vec::new();
    let mut index = 0;

    while index < tokens.len() {
        if tokens[index].kind == SyntaxKind::AtKeyword
            && css_modules_value_query_prelude_at_rule(&tokens[index].text)
        {
            let prelude_start = index + 1;
            let Some(prelude_end) = at_rule_prelude_end_index(tokens, prelude_start) else {
                index += 1;
                continue;
            };
            for candidate_index in prelude_start..prelude_end {
                let token = &tokens[candidate_index];
                if token.kind != SyntaxKind::Ident
                    || !query_prelude_ident_is_css_modules_value_reference(
                        tokens,
                        candidate_index,
                        prelude_start,
                        &tokens[index].text,
                    )
                {
                    continue;
                }
                let Some(resolved) = resolved_definitions_by_name.get(&token.text) else {
                    continue;
                };
                replacements.push((token_start(token), token_end(token), resolved.clone()));
            }
            index = prelude_end;
            continue;
        }
        index += 1;
    }

    replacements
}

fn css_modules_value_query_prelude_at_rule(text: &str) -> bool {
    matches!(
        text.to_ascii_lowercase().as_str(),
        "@media" | "@supports" | "@container" | "@custom-media" | "@scope"
    )
}

fn query_prelude_ident_is_css_modules_value_reference(
    tokens: &[omena_parser::LexedToken],
    candidate_index: usize,
    prelude_start: usize,
    at_rule_text: &str,
) -> bool {
    if at_rule_text.eq_ignore_ascii_case("@scope") {
        return !matches!(
            tokens[candidate_index].text.to_ascii_lowercase().as_str(),
            "to"
        );
    }
    query_prelude_ident_is_feature_value(tokens, candidate_index, prelude_start)
}

fn query_prelude_ident_is_feature_value(
    tokens: &[omena_parser::LexedToken],
    candidate_index: usize,
    prelude_start: usize,
) -> bool {
    if query_prelude_ident_is_known_feature_name(&tokens[candidate_index].text) {
        return false;
    }

    previous_significant_token_kind(tokens, candidate_index, prelude_start)
        .is_some_and(|kind| kind == SyntaxKind::Colon || query_prelude_token_is_comparator(kind))
        || next_non_comment_token_kind(tokens, candidate_index)
            .is_some_and(query_prelude_token_is_comparator)
}

fn query_prelude_token_is_comparator(kind: SyntaxKind) -> bool {
    matches!(
        kind,
        SyntaxKind::GreaterThan | SyntaxKind::LessThan | SyntaxKind::Equals
    )
}

fn query_prelude_ident_is_known_feature_name(text: &str) -> bool {
    matches!(
        text.to_ascii_lowercase().as_str(),
        "any-hover"
            | "any-pointer"
            | "aspect-ratio"
            | "block-size"
            | "color"
            | "color-gamut"
            | "color-index"
            | "display-mode"
            | "dynamic-range"
            | "forced-colors"
            | "grid"
            | "height"
            | "hover"
            | "inline-size"
            | "inverted-colors"
            | "monochrome"
            | "orientation"
            | "overflow-block"
            | "overflow-inline"
            | "pointer"
            | "prefers-color-scheme"
            | "prefers-contrast"
            | "prefers-reduced-data"
            | "prefers-reduced-motion"
            | "prefers-reduced-transparency"
            | "resolution"
            | "scripting"
            | "update"
            | "video-dynamic-range"
            | "width"
    )
}

fn resolve_static_css_modules_value_definition(
    name: &str,
    dialect: StyleDialect,
    definitions_by_name: &BTreeMap<String, &StaticCssModulesValueDefinition>,
    visiting: &mut Vec<String>,
) -> Option<String> {
    if visiting.iter().any(|candidate| candidate == name) {
        return None;
    }
    let definition = definitions_by_name.get(name)?;
    if css_modules_value_references_known_definition(
        &definition.value,
        dialect,
        definitions_by_name,
    ) {
        visiting.push(name.to_string());
        let resolved = substitute_static_css_modules_value_references(
            &definition.value,
            dialect,
            definitions_by_name,
            visiting,
        );
        visiting.pop();
        return resolved;
    }
    is_static_css_modules_value_literal(&definition.value, dialect)
        .then(|| definition.value.clone())
}

fn substitute_static_css_modules_value_references(
    value: &str,
    dialect: StyleDialect,
    definitions_by_name: &BTreeMap<String, &StaticCssModulesValueDefinition>,
    visiting: &mut Vec<String>,
) -> Option<String> {
    let lexed = lex(value, dialect);
    let tokens = lexed.tokens();
    let mut replacements = Vec::new();

    for token in tokens {
        if token.kind != SyntaxKind::Ident || !definitions_by_name.contains_key(&token.text) {
            continue;
        }
        let resolved = resolve_static_css_modules_value_definition(
            &token.text,
            dialect,
            definitions_by_name,
            visiting,
        )?;
        replacements.push((token_start(token), token_end(token), resolved));
    }

    if replacements.is_empty() {
        return None;
    }

    let (output, mutation_count) = replace_source_ranges(value, &replacements);
    (mutation_count > 0).then_some(output)
}

fn substitute_resolved_css_modules_value_references(
    value: &str,
    dialect: StyleDialect,
    resolved_definitions_by_name: &BTreeMap<String, String>,
) -> Option<String> {
    let lexed = lex(value, dialect);
    let tokens = lexed.tokens();
    let mut replacements = Vec::new();

    for token in tokens {
        if token.kind != SyntaxKind::Ident {
            continue;
        }
        let Some(resolved) = resolved_definitions_by_name.get(&token.text) else {
            continue;
        };
        replacements.push((token_start(token), token_end(token), resolved.clone()));
    }

    if replacements.is_empty() {
        return None;
    }

    let (output, mutation_count) = replace_source_ranges(value, &replacements);
    (mutation_count > 0).then_some(output)
}

fn css_modules_value_references_known_definition(
    value: &str,
    dialect: StyleDialect,
    definitions_by_name: &BTreeMap<String, &StaticCssModulesValueDefinition>,
) -> bool {
    let lexed = lex(value, dialect);
    lexed.tokens().iter().any(|token| {
        token.kind == SyntaxKind::Ident && definitions_by_name.contains_key(&token.text)
    })
}

fn is_static_css_modules_value_literal(value: &str, dialect: StyleDialect) -> bool {
    parse_static_srgb_color(value).is_some()
        || parse_numeric_value_with_unit(value)
            .map(|numeric| {
                numeric.unit.is_empty() || css_modules_value_unit_is_static(numeric.unit)
            })
            .unwrap_or(false)
        || is_static_css_modules_identifier_literal(value, dialect)
        || is_static_css_modules_simple_selector_literal(value, dialect)
}

fn is_static_css_modules_identifier_literal(value: &str, dialect: StyleDialect) -> bool {
    let lexed = lex(value, dialect);
    let significant_tokens = lexed
        .tokens()
        .iter()
        .filter(|token| token.kind != SyntaxKind::Whitespace && !is_comment_token(token.kind))
        .collect::<Vec<_>>();
    significant_tokens.len() == 1
        && significant_tokens[0].kind == SyntaxKind::Ident
        && significant_tokens[0].text == value.trim()
}

fn is_static_css_modules_simple_selector_literal(value: &str, dialect: StyleDialect) -> bool {
    let trimmed = value.trim();
    let lexed = lex(value, dialect);
    let significant_tokens = lexed
        .tokens()
        .iter()
        .filter(|token| token.kind != SyntaxKind::Whitespace && !is_comment_token(token.kind))
        .collect::<Vec<_>>();
    let token_text = significant_tokens
        .iter()
        .map(|token| token.text.as_str())
        .collect::<String>();
    if token_text != trimmed {
        return false;
    }
    match significant_tokens.as_slice() {
        [token] => token.kind == SyntaxKind::Hash,
        [prefix, ident] => {
            ident.kind == SyntaxKind::Ident
                && matches!(prefix.kind, SyntaxKind::Dot | SyntaxKind::Colon)
        }
        _ => false,
    }
}

fn css_modules_value_unit_is_static(unit: &str) -> bool {
    matches!(
        unit.to_ascii_lowercase().as_str(),
        "%" | "ch"
            | "cm"
            | "deg"
            | "dppx"
            | "em"
            | "fr"
            | "in"
            | "ms"
            | "pc"
            | "pt"
            | "px"
            | "rem"
            | "s"
            | "turn"
            | "vh"
            | "vmax"
            | "vmin"
            | "vw"
    )
}

pub(crate) fn tree_shake_css_modules_values_with_lexer(
    source: &str,
    dialect: StyleDialect,
    reachable_value_names: &[String],
    reachable_keyframe_names: &[String],
    reachable_class_names: &[String],
) -> (String, Vec<TransformSemanticRemovalCandidate>) {
    let (replacements, removals) = collect_tree_shake_css_modules_value_replacements(
        source,
        dialect,
        reachable_value_names,
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

pub(crate) fn tree_shake_css_modules_values_with_ir_transaction_on_ir(
    ir: &mut TransformIrV0,
    dialect: StyleDialect,
    reachable_value_names: &[String],
    reachable_keyframe_names: &[String],
    reachable_class_names: &[String],
) -> Result<Vec<TransformSemanticRemovalCandidate>, TransformIrSourceReplacementErrorV0> {
    let (replacements, removals) = collect_tree_shake_css_modules_value_replacements_from_ir(
        ir,
        dialect,
        reachable_value_names,
        reachable_keyframe_names,
        reachable_class_names,
    );
    let replacements = non_overlapping_css_modules_value_replacements(replacements);
    let (node_deletions, source_replacements): (Vec<_>, Vec<_>) =
        replacements.into_iter().partition(|replacement| {
            replacement.replacement.is_empty()
                && css_modules_value_deletion_node_kind(replacement.kind).is_some()
        });
    let node_deletion_ids = css_modules_value_deletion_node_ids(ir, node_deletions.as_slice())?;
    let source_replacements =
        css_modules_value_import_node_replacements(source_replacements.as_slice())?;
    replace_ir_nodes_in_ir(ir, "tree-shake-value", source_replacements.as_slice())?;
    if !node_deletion_ids.is_empty() {
        delete_ir_nodes_in_ir(ir, "tree-shake-value", node_deletion_ids.as_slice())?;
    }
    Ok(removals)
}

fn css_modules_value_import_node_replacements(
    replacements: &[TransformIrSourceReplacementV0],
) -> Result<Vec<TransformIrSourceReplacementV0>, TransformIrSourceReplacementErrorV0> {
    replacements
        .iter()
        .map(|replacement| {
            if replacement.kind != TransformIrReplacementKindV0::CssModuleValueImportSource {
                return Err(TransformIrSourceReplacementErrorV0::MissingNode {
                    source_span_start: replacement.source_span_start,
                    source_span_end: replacement.source_span_end,
                    kind: replacement.kind,
                    candidate_spans: Vec::new(),
                });
            }
            Ok(TransformIrSourceReplacementV0 {
                source_span_start: replacement.source_span_start,
                source_span_end: replacement.source_span_end,
                replacement: replacement.replacement.clone(),
                kind: TransformIrReplacementKindV0::AtRule,
            })
        })
        .collect()
}

fn collect_tree_shake_css_modules_value_replacements_from_ir(
    ir: &TransformIrV0,
    dialect: StyleDialect,
    reachable_value_names: &[String],
    reachable_keyframe_names: &[String],
    reachable_class_names: &[String],
) -> (
    Vec<TransformIrSourceReplacementV0>,
    Vec<TransformSemanticRemovalCandidate>,
) {
    let definitions = collect_static_local_css_modules_value_definitions_from_ir(ir, dialect);
    let import_statements = collect_static_css_modules_value_import_statements_from_ir(ir, dialect);
    let export_rules = collect_static_css_modules_icss_export_rules_from_ir(ir);
    if definitions.is_empty() && import_statements.is_empty() && export_rules.is_empty() {
        return (Vec::new(), Vec::new());
    }
    let mut value_names = definitions
        .iter()
        .map(|definition| definition.name.clone())
        .collect::<Vec<_>>();
    for statement in &import_statements {
        for local_name in &statement.local_names {
            push_unique_string(&mut value_names, local_name.clone());
        }
    }

    let referenced_names =
        collect_reachable_css_modules_value_names_from_ir(CssModulesValueIrReachabilityInput {
            ir,
            dialect,
            definitions: &definitions,
            value_names: &value_names,
            external_roots: reachable_value_names,
            external_keyframe_roots: reachable_keyframe_names,
            reachable_class_names,
            export_rules: &export_rules,
        });

    let mut removals = definitions
        .iter()
        .filter(|definition| {
            can_tree_shake_local_css_modules_value_definition(definition, &definitions)
                && !referenced_names.iter().any(|name| name == &definition.name)
        })
        .map(|definition| TransformSemanticRemovalCandidate {
            symbol_kind: "cssModuleValue",
            name: definition.name.clone(),
            source_span_start: definition.start,
            source_span_end: definition.end,
            reason: "CSS Modules value definition was absent from transitive value references and the closed-style-world reachable value set",
        })
        .collect::<Vec<_>>();
    let mut replacements = removals
        .iter()
        .map(|removal| TransformIrSourceReplacementV0 {
            source_span_start: removal.source_span_start,
            source_span_end: removal.source_span_end,
            replacement: String::new(),
            kind: TransformIrReplacementKindV0::CssModuleValueDefinition,
        })
        .collect::<Vec<_>>();

    for statement in &import_statements {
        let unreachable_bindings = statement
            .bindings
            .iter()
            .filter(|binding| {
                !referenced_names
                    .iter()
                    .any(|reachable| reachable == &binding.local_name)
            })
            .collect::<Vec<_>>();
        if unreachable_bindings.is_empty() {
            continue;
        }
        removals.extend(
            unreachable_bindings
                .iter()
                .map(|binding| TransformSemanticRemovalCandidate {
                    symbol_kind: "cssModuleValue",
                    name: binding.local_name.clone(),
                    source_span_start: binding.start,
                    source_span_end: binding.end,
                    reason: "imported CSS Modules value binding was absent from transitive value references and the closed-style-world reachable value set",
                }),
        );
        let reachable_binding_texts = statement
            .bindings
            .iter()
            .filter(|binding| {
                referenced_names
                    .iter()
                    .any(|reachable| reachable == &binding.local_name)
            })
            .map(|binding| binding.binding_text.as_str())
            .collect::<Vec<_>>();
        let replacement = if reachable_binding_texts.is_empty() {
            String::new()
        } else {
            format!(
                "@value {} {};",
                reachable_binding_texts.join(", "),
                statement.from_clause
            )
        };
        replacements.push(TransformIrSourceReplacementV0 {
            source_span_start: statement.start,
            source_span_end: statement.end,
            replacement,
            kind: TransformIrReplacementKindV0::CssModuleValueImportSource,
        });
    }

    for rule in &export_rules {
        let unreachable_exports = rule
            .declarations
            .iter()
            .filter(|declaration| {
                !reachable_value_names
                    .iter()
                    .any(|reachable| reachable == &declaration.export_name)
            })
            .collect::<Vec<_>>();
        if unreachable_exports.is_empty() {
            continue;
        }
        removals.extend(
            unreachable_exports
                .iter()
                .map(|declaration| TransformSemanticRemovalCandidate {
                    symbol_kind: "cssModuleIcssExport",
                    name: declaration.export_name.clone(),
                    source_span_start: declaration.start,
                    source_span_end: declaration.end,
                    reason: "ICSS export declaration was absent from the closed-style-world reachable value export set",
                }),
        );
        if unreachable_exports.len() == rule.declarations.len() {
            replacements.push(TransformIrSourceReplacementV0 {
                source_span_start: rule.start,
                source_span_end: rule.end,
                replacement: String::new(),
                kind: TransformIrReplacementKindV0::IcssExportName,
            });
        } else {
            replacements.extend(unreachable_exports.iter().map(|declaration| {
                TransformIrSourceReplacementV0 {
                    source_span_start: declaration.start,
                    source_span_end: declaration.end,
                    replacement: String::new(),
                    kind: TransformIrReplacementKindV0::Declaration,
                }
            }));
        }
    }

    (replacements, removals)
}

fn collect_static_local_css_modules_value_definitions_from_ir(
    ir: &TransformIrV0,
    dialect: StyleDialect,
) -> Vec<StaticCssModulesValueDefinition> {
    ir_value_at_rule_tokens(ir, dialect)
        .into_iter()
        .flat_map(|tokens| collect_static_local_css_modules_value_definitions(&tokens))
        .collect()
}

fn collect_static_css_modules_value_import_statements_from_ir(
    ir: &TransformIrV0,
    dialect: StyleDialect,
) -> Vec<StaticCssModulesValueImportStatement> {
    ir_value_at_rule_tokens(ir, dialect)
        .into_iter()
        .flat_map(|tokens| collect_static_css_modules_value_import_statements(&tokens))
        .collect()
}

fn ir_value_at_rule_tokens(ir: &TransformIrV0, dialect: StyleDialect) -> Vec<Vec<LexedToken>> {
    ir.nodes
        .iter()
        .filter(|node| !node.deleted && node.kind == IrNodeKindV0::AtRule && node.parent.is_none())
        .filter(|node| {
            ir_at_rule_keyword(ir.source_text(), node)
                .is_some_and(|keyword| keyword.eq_ignore_ascii_case("@value"))
        })
        .filter_map(|node| {
            lex_ir_source_slice_with_offset(
                ir.source_text(),
                dialect,
                node.source_span_start,
                node.source_span_end,
            )
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

fn collect_static_css_modules_icss_export_rules_from_ir(
    ir: &TransformIrV0,
) -> Vec<CssModulesIcssExportRule> {
    collect_declaration_ordinary_rule_slices_from_ir(ir)
        .into_iter()
        .filter(|rule| rule.selector.trim().eq_ignore_ascii_case(":export"))
        .filter_map(|rule| {
            let declarations = collect_simple_declarations_from_ir(ir, &rule)
                .into_iter()
                .map(|declaration| CssModulesIcssExportDeclaration {
                    export_name: declaration.property,
                    value: declaration.value,
                    start: declaration.start,
                    end: declaration.end,
                })
                .collect::<Vec<_>>();
            (!declarations.is_empty()).then_some(CssModulesIcssExportRule {
                start: rule.start,
                end: rule.end,
                declarations,
            })
        })
        .collect()
}

fn collect_tree_shake_css_modules_value_replacements(
    source: &str,
    dialect: StyleDialect,
    reachable_value_names: &[String],
    reachable_keyframe_names: &[String],
    reachable_class_names: &[String],
) -> (
    Vec<TransformIrSourceReplacementV0>,
    Vec<TransformSemanticRemovalCandidate>,
) {
    let lexed = lex(source, dialect);
    let tokens = lexed.tokens();
    let definitions = collect_static_local_css_modules_value_definitions(tokens);
    let import_statements = collect_static_css_modules_value_import_statements(tokens);
    let export_rules = collect_static_css_modules_icss_export_rules(source, tokens);
    if definitions.is_empty() && import_statements.is_empty() && export_rules.is_empty() {
        return (Vec::new(), Vec::new());
    }
    let mut value_names = definitions
        .iter()
        .map(|definition| definition.name.clone())
        .collect::<Vec<_>>();
    for statement in &import_statements {
        for local_name in &statement.local_names {
            push_unique_string(&mut value_names, local_name.clone());
        }
    }

    let referenced_names =
        collect_reachable_css_modules_value_names(CssModulesValueReachabilityInput {
            source,
            tokens,
            dialect,
            definitions: &definitions,
            value_names: &value_names,
            external_roots: reachable_value_names,
            external_keyframe_roots: reachable_keyframe_names,
            reachable_class_names,
            export_rules: &export_rules,
        });

    let mut removals = definitions
        .iter()
        .filter(|definition| {
            can_tree_shake_local_css_modules_value_definition(definition, &definitions)
                && !referenced_names.iter().any(|name| name == &definition.name)
        })
        .map(|definition| TransformSemanticRemovalCandidate {
            symbol_kind: "cssModuleValue",
            name: definition.name.clone(),
            source_span_start: definition.start,
            source_span_end: definition.end,
            reason: "CSS Modules value definition was absent from transitive value references and the closed-style-world reachable value set",
        })
        .collect::<Vec<_>>();
    let mut replacements = removals
        .iter()
        .map(|removal| TransformIrSourceReplacementV0 {
            source_span_start: removal.source_span_start,
            source_span_end: removal.source_span_end,
            replacement: String::new(),
            kind: TransformIrReplacementKindV0::CssModuleValueDefinition,
        })
        .collect::<Vec<_>>();
    for statement in &import_statements {
        let unreachable_bindings = statement
            .bindings
            .iter()
            .filter(|binding| {
                !referenced_names
                    .iter()
                    .any(|reachable| reachable == &binding.local_name)
            })
            .collect::<Vec<_>>();
        if unreachable_bindings.is_empty() {
            continue;
        }
        removals.extend(
            unreachable_bindings
                .iter()
                .map(|binding| TransformSemanticRemovalCandidate {
                    symbol_kind: "cssModuleValue",
                    name: binding.local_name.clone(),
                    source_span_start: binding.start,
                    source_span_end: binding.end,
                    reason: "imported CSS Modules value binding was absent from transitive value references and the closed-style-world reachable value set",
                }),
        );
        let reachable_binding_texts = statement
            .bindings
            .iter()
            .filter(|binding| {
                referenced_names
                    .iter()
                    .any(|reachable| reachable == &binding.local_name)
            })
            .map(|binding| binding.binding_text.as_str())
            .collect::<Vec<_>>();
        let replacement = if reachable_binding_texts.is_empty() {
            String::new()
        } else {
            format!(
                "@value {} {};",
                reachable_binding_texts.join(", "),
                statement.from_clause
            )
        };
        replacements.push(TransformIrSourceReplacementV0 {
            source_span_start: statement.start,
            source_span_end: statement.end,
            replacement,
            kind: TransformIrReplacementKindV0::CssModuleValueImportSource,
        });
    }
    for rule in &export_rules {
        let unreachable_exports = rule
            .declarations
            .iter()
            .filter(|declaration| {
                !reachable_value_names
                    .iter()
                    .any(|reachable| reachable == &declaration.export_name)
            })
            .collect::<Vec<_>>();
        if unreachable_exports.is_empty() {
            continue;
        }
        removals.extend(
            unreachable_exports
                .iter()
                .map(|declaration| TransformSemanticRemovalCandidate {
                    symbol_kind: "cssModuleIcssExport",
                    name: declaration.export_name.clone(),
                    source_span_start: declaration.start,
                    source_span_end: declaration.end,
                    reason: "ICSS export declaration was absent from the closed-style-world reachable value export set",
                }),
        );
        if unreachable_exports.len() == rule.declarations.len() {
            replacements.push(TransformIrSourceReplacementV0 {
                source_span_start: rule.start,
                source_span_end: rule.end,
                replacement: String::new(),
                kind: TransformIrReplacementKindV0::IcssExportName,
            });
        } else {
            replacements.extend(unreachable_exports.iter().map(|declaration| {
                TransformIrSourceReplacementV0 {
                    source_span_start: declaration.start,
                    source_span_end: declaration.end,
                    replacement: String::new(),
                    kind: TransformIrReplacementKindV0::Declaration,
                }
            }));
        }
    }
    (replacements, removals)
}

fn non_overlapping_css_modules_value_replacements(
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

fn css_modules_value_deletion_node_ids(
    ir: &TransformIrV0,
    replacements: &[TransformIrSourceReplacementV0],
) -> Result<Vec<IrNodeIdV0>, TransformIrSourceReplacementErrorV0> {
    replacements
        .iter()
        .map(|replacement| css_modules_value_deletion_node_id(ir, replacement))
        .collect()
}

fn css_modules_value_deletion_node_id(
    ir: &TransformIrV0,
    replacement: &TransformIrSourceReplacementV0,
) -> Result<IrNodeIdV0, TransformIrSourceReplacementErrorV0> {
    let Some(kind) = css_modules_value_deletion_node_kind(replacement.kind) else {
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

const fn css_modules_value_deletion_node_kind(
    kind: TransformIrReplacementKindV0,
) -> Option<IrNodeKindV0> {
    match kind {
        TransformIrReplacementKindV0::CssModuleValueDefinition
        | TransformIrReplacementKindV0::CssModuleValueImportSource => Some(IrNodeKindV0::AtRule),
        TransformIrReplacementKindV0::Declaration => Some(IrNodeKindV0::Declaration),
        TransformIrReplacementKindV0::IcssExportName => Some(IrNodeKindV0::StyleRule),
        _ => None,
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct CssModulesIcssExportRule {
    start: usize,
    end: usize,
    declarations: Vec<CssModulesIcssExportDeclaration>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct CssModulesIcssExportDeclaration {
    export_name: String,
    value: String,
    start: usize,
    end: usize,
}

fn collect_static_css_modules_icss_export_rules(
    source: &str,
    tokens: &[omena_parser::LexedToken],
) -> Vec<CssModulesIcssExportRule> {
    collect_declaration_ordinary_rule_slices(source, tokens)
        .into_iter()
        .filter(|rule| rule.selector.trim().eq_ignore_ascii_case(":export"))
        .filter_map(|rule| {
            let (block_start_index, block_end_index) =
                rule_block_token_indexes(tokens, rule.block_start, rule.block_end)?;
            let declarations =
                collect_simple_declarations_in_block(tokens, block_start_index, block_end_index)
                    .into_iter()
                    .map(|declaration| CssModulesIcssExportDeclaration {
                        export_name: declaration.property,
                        value: declaration.value,
                        start: declaration.start,
                        end: declaration.end,
                    })
                    .collect::<Vec<_>>();
            (!declarations.is_empty()).then_some(CssModulesIcssExportRule {
                start: rule.start,
                end: rule.end,
                declarations,
            })
        })
        .collect()
}

struct CssModulesValueReachabilityInput<'a> {
    source: &'a str,
    tokens: &'a [omena_parser::LexedToken],
    dialect: StyleDialect,
    definitions: &'a [StaticCssModulesValueDefinition],
    value_names: &'a [String],
    external_roots: &'a [String],
    external_keyframe_roots: &'a [String],
    reachable_class_names: &'a [String],
    export_rules: &'a [CssModulesIcssExportRule],
}

fn collect_reachable_css_modules_value_names(
    input: CssModulesValueReachabilityInput<'_>,
) -> Vec<String> {
    let mut root_names = input.external_roots.to_vec();
    let mut dependencies_by_name = BTreeMap::<String, Vec<String>>::new();
    let scope_blocks = collect_css_module_scope_blocks(input.source, input.tokens);

    for definition in input.definitions {
        for reference_name in collect_css_modules_value_references_in_value(
            &definition.value,
            input.dialect,
            input.value_names,
        ) {
            if reference_name == definition.name {
                continue;
            }
            let dependencies = dependencies_by_name
                .entry(definition.name.clone())
                .or_default();
            push_unique_string(dependencies, reference_name);
        }
    }
    for rule in input.export_rules {
        for declaration in &rule.declarations {
            if !input
                .external_roots
                .iter()
                .any(|root| root == &declaration.export_name)
            {
                continue;
            }
            for reference_name in collect_css_modules_value_references_in_value(
                &declaration.value,
                input.dialect,
                input.value_names,
            ) {
                push_unique_string(&mut root_names, reference_name);
            }
        }
    }
    for name in collect_css_modules_value_roots_from_reachable_keyframes(
        input.source,
        input.tokens,
        input.dialect,
        input.value_names,
        input.external_keyframe_roots,
        input.reachable_class_names,
    ) {
        push_unique_string(&mut root_names, name);
    }
    for name in collect_css_modules_value_roots_from_descriptor_at_rules(
        input.tokens,
        input.dialect,
        input.value_names,
    ) {
        push_unique_string(&mut root_names, name);
    }

    for rule in collect_declaration_ordinary_rule_slices(input.source, input.tokens) {
        if rule.selector.trim().eq_ignore_ascii_case(":export") {
            continue;
        }
        if !rule_slice_matches_reachable_class_context(
            &rule,
            &scope_blocks,
            input.reachable_class_names,
        ) {
            continue;
        }
        let Some((block_start_index, block_end_index)) =
            rule_block_token_indexes(input.tokens, rule.block_start, rule.block_end)
        else {
            continue;
        };
        for declaration in
            collect_simple_declarations_in_block(input.tokens, block_start_index, block_end_index)
        {
            for reference_name in collect_css_modules_value_references_in_value(
                &declaration.value,
                input.dialect,
                input.value_names,
            ) {
                push_unique_string(&mut root_names, reference_name);
            }
        }
    }
    collect_css_modules_value_references_in_at_rule_preludes(
        input.source,
        input.tokens,
        input.value_names,
        &mut root_names,
        input.reachable_class_names,
        &scope_blocks,
    );

    close_css_modules_value_dependency_graph(root_names, &dependencies_by_name)
}

struct CssModulesValueIrReachabilityInput<'a> {
    ir: &'a TransformIrV0,
    dialect: StyleDialect,
    definitions: &'a [StaticCssModulesValueDefinition],
    value_names: &'a [String],
    external_roots: &'a [String],
    external_keyframe_roots: &'a [String],
    reachable_class_names: &'a [String],
    export_rules: &'a [CssModulesIcssExportRule],
}

fn collect_reachable_css_modules_value_names_from_ir(
    input: CssModulesValueIrReachabilityInput<'_>,
) -> Vec<String> {
    let mut root_names = input.external_roots.to_vec();
    let mut dependencies_by_name = BTreeMap::<String, Vec<String>>::new();
    let scope_blocks = collect_css_module_scope_blocks_from_ir(input.ir);

    for definition in input.definitions {
        for reference_name in collect_css_modules_value_references_in_value(
            &definition.value,
            input.dialect,
            input.value_names,
        ) {
            if reference_name == definition.name {
                continue;
            }
            let dependencies = dependencies_by_name
                .entry(definition.name.clone())
                .or_default();
            push_unique_string(dependencies, reference_name);
        }
    }
    for rule in input.export_rules {
        for declaration in &rule.declarations {
            if !input
                .external_roots
                .iter()
                .any(|root| root == &declaration.export_name)
            {
                continue;
            }
            for reference_name in collect_css_modules_value_references_in_value(
                &declaration.value,
                input.dialect,
                input.value_names,
            ) {
                push_unique_string(&mut root_names, reference_name);
            }
        }
    }
    for name in collect_css_modules_value_roots_from_reachable_keyframes_from_ir(
        input.ir,
        input.dialect,
        input.value_names,
        input.external_keyframe_roots,
        input.reachable_class_names,
    ) {
        push_unique_string(&mut root_names, name);
    }
    for name in collect_css_modules_value_roots_from_descriptor_at_rules_from_ir(
        input.ir,
        input.dialect,
        input.value_names,
    ) {
        push_unique_string(&mut root_names, name);
    }

    for rule in collect_declaration_ordinary_rule_slices_from_ir(input.ir) {
        if rule.selector.trim().eq_ignore_ascii_case(":export") {
            continue;
        }
        if !rule_slice_matches_reachable_class_context(
            &rule,
            &scope_blocks,
            input.reachable_class_names,
        ) {
            continue;
        }
        for declaration in collect_simple_declarations_from_ir(input.ir, &rule) {
            for reference_name in collect_css_modules_value_references_in_value(
                &declaration.value,
                input.dialect,
                input.value_names,
            ) {
                push_unique_string(&mut root_names, reference_name);
            }
        }
    }
    collect_css_modules_value_references_in_at_rule_preludes_from_ir(
        input.ir,
        input.dialect,
        input.value_names,
        &mut root_names,
        input.reachable_class_names,
        &scope_blocks,
    );

    close_css_modules_value_dependency_graph(root_names, &dependencies_by_name)
}

fn collect_css_modules_value_roots_from_reachable_keyframes(
    source: &str,
    tokens: &[omena_parser::LexedToken],
    dialect: StyleDialect,
    value_names: &[String],
    external_keyframe_roots: &[String],
    reachable_class_names: &[String],
) -> Vec<String> {
    let keyframes = collect_keyframes_rules(tokens);
    if keyframes.is_empty() {
        return Vec::new();
    }

    let referenced_keyframe_names =
        collect_referenced_keyframe_names(source, tokens, reachable_class_names);
    let dynamic_keyframe_reachability = referenced_keyframe_names.is_none();
    let mut reachable_keyframe_names = referenced_keyframe_names.unwrap_or_default();
    for name in external_keyframe_roots {
        push_unique_string(&mut reachable_keyframe_names, name.clone());
    }

    let mut roots = Vec::new();
    for rule in collect_declaration_ordinary_rule_slices(source, tokens) {
        let Some(keyframe) = enclosing_keyframe_for_value_rule(&rule, &keyframes) else {
            continue;
        };
        if !dynamic_keyframe_reachability
            && !reachable_keyframe_names
                .iter()
                .any(|name| name == &keyframe.name)
        {
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
            for reference_name in collect_css_modules_value_references_in_value(
                &declaration.value,
                dialect,
                value_names,
            ) {
                push_unique_string(&mut roots, reference_name);
            }
        }
    }
    roots
}

fn collect_css_modules_value_roots_from_reachable_keyframes_from_ir(
    ir: &TransformIrV0,
    dialect: StyleDialect,
    value_names: &[String],
    external_keyframe_roots: &[String],
    reachable_class_names: &[String],
) -> Vec<String> {
    let keyframes = collect_keyframes_rules_from_ir(ir);
    if keyframes.is_empty() {
        return Vec::new();
    }

    let referenced_keyframe_names =
        collect_referenced_keyframe_names_from_ir(ir, reachable_class_names);
    let dynamic_keyframe_reachability = referenced_keyframe_names.is_none();
    let mut reachable_keyframe_names = referenced_keyframe_names.unwrap_or_default();
    for name in external_keyframe_roots {
        push_unique_string(&mut reachable_keyframe_names, name.clone());
    }

    let mut roots = Vec::new();
    for keyframe in &keyframes {
        if !dynamic_keyframe_reachability
            && !reachable_keyframe_names
                .iter()
                .any(|name| name == &keyframe.name)
        {
            continue;
        }
        for rule in collect_keyframe_declaration_rule_slices_from_ir(ir, dialect, keyframe) {
            for declaration in
                collect_simple_declarations_from_keyframe_slice(ir.source_text(), dialect, &rule)
            {
                for reference_name in collect_css_modules_value_references_in_value(
                    &declaration.value,
                    dialect,
                    value_names,
                ) {
                    push_unique_string(&mut roots, reference_name);
                }
            }
        }
    }
    roots
}

fn collect_keyframe_declaration_rule_slices_from_ir(
    ir: &TransformIrV0,
    dialect: StyleDialect,
    keyframe: &KeyframesRuleSlice,
) -> Vec<SimpleRuleSlice> {
    let Some(tokens) =
        lex_ir_source_slice_with_offset(ir.source_text(), dialect, keyframe.start, keyframe.end)
    else {
        return Vec::new();
    };
    collect_declaration_ordinary_rule_slices(ir.source_text(), &tokens)
        .into_iter()
        .filter(|rule| {
            enclosing_keyframe_for_value_rule(rule, std::slice::from_ref(keyframe)).is_some()
        })
        .collect()
}

fn collect_simple_declarations_from_keyframe_slice(
    source: &str,
    dialect: StyleDialect,
    rule: &SimpleRuleSlice,
) -> Vec<CssModulesValueDeclarationIrViewV0> {
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
        .map(|declaration| CssModulesValueDeclarationIrViewV0 {
            property: declaration.property,
            value: declaration.value,
            start: declaration.start,
            end: declaration.end,
        })
        .collect()
}

fn collect_css_modules_value_roots_from_descriptor_at_rules(
    tokens: &[omena_parser::LexedToken],
    dialect: StyleDialect,
    value_names: &[String],
) -> Vec<String> {
    let mut roots = Vec::new();
    let mut index = 0usize;

    while index < tokens.len() {
        if tokens[index].kind != SyntaxKind::AtKeyword
            || !descriptor_at_rule_can_reference_css_modules_values(&tokens[index].text)
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
            for name in collect_css_modules_value_references_in_value(
                &declaration.value,
                dialect,
                value_names,
            ) {
                push_unique_string(&mut roots, name);
            }
        }
        index = block_end_index + 1;
    }

    roots
}

fn collect_css_modules_value_roots_from_descriptor_at_rules_from_ir(
    ir: &TransformIrV0,
    dialect: StyleDialect,
    value_names: &[String],
) -> Vec<String> {
    let mut roots = Vec::new();
    for node in ir
        .nodes
        .iter()
        .filter(|node| !node.deleted && node.kind == IrNodeKindV0::AtRule)
    {
        let Some(keyword) = ir_at_rule_keyword(ir.source_text(), node) else {
            continue;
        };
        if !descriptor_at_rule_can_reference_css_modules_values(keyword) {
            continue;
        }
        let Some((block_start, block_end)) = at_rule_body_bounds_from_ir(ir.source_text(), node)
        else {
            continue;
        };
        for declaration in collect_simple_declarations_between_from_ir(ir, block_start, block_end) {
            for name in collect_css_modules_value_references_in_value(
                &declaration.value,
                dialect,
                value_names,
            ) {
                push_unique_string(&mut roots, name);
            }
        }
    }
    roots
}

fn descriptor_at_rule_can_reference_css_modules_values(text: &str) -> bool {
    matches!(
        text.to_ascii_lowercase().as_str(),
        "@color-profile"
            | "@counter-style"
            | "@font-face"
            | "@font-palette-values"
            | "@page"
            | "@property"
    )
}

fn enclosing_keyframe_for_value_rule<'a>(
    rule: &crate::helpers::rules::SimpleRuleSlice,
    keyframes: &'a [KeyframesRuleSlice],
) -> Option<&'a KeyframesRuleSlice> {
    keyframes
        .iter()
        .find(|keyframe| rule.start >= keyframe.start && rule.end <= keyframe.end)
}

fn collect_css_modules_value_references_in_at_rule_preludes(
    source: &str,
    tokens: &[omena_parser::LexedToken],
    definition_names: &[String],
    root_names: &mut Vec<String>,
    reachable_class_names: &[String],
    scope_blocks: &[CssModuleScopeBlock],
) {
    let mut index = 0;
    while index < tokens.len() {
        if tokens[index].kind != SyntaxKind::AtKeyword
            || !at_rule_prelude_can_reference_css_modules_values(&tokens[index].text)
        {
            index += 1;
            continue;
        }

        let mut prelude_index = index + 1;
        let mut prelude_names = Vec::new();
        let mut terminator_index = None;
        while prelude_index < tokens.len() {
            match tokens[prelude_index].kind {
                SyntaxKind::Ident
                    if definition_names
                        .iter()
                        .any(|name| name == &tokens[prelude_index].text)
                        && query_prelude_ident_is_css_modules_value_reference(
                            tokens,
                            prelude_index,
                            index + 1,
                            &tokens[index].text,
                        ) =>
                {
                    push_unique_string(&mut prelude_names, tokens[prelude_index].text.clone());
                }
                SyntaxKind::LeftBrace | SyntaxKind::Semicolon | SyntaxKind::RightBrace => {
                    terminator_index = Some(prelude_index);
                    break;
                }
                _ => {}
            }
            prelude_index += 1;
        }
        let prelude_can_keep_roots = match terminator_index {
            Some(terminator_index) if tokens[terminator_index].kind == SyntaxKind::LeftBrace => {
                matching_right_brace_index(tokens, terminator_index).is_some_and(|close_index| {
                    at_rule_block_has_reachable_ordinary_rule(
                        source,
                        tokens,
                        terminator_index,
                        close_index,
                        reachable_class_names,
                        scope_blocks,
                    )
                })
            }
            Some(terminator_index) => tokens[terminator_index].kind == SyntaxKind::Semicolon,
            None => true,
        };
        if prelude_can_keep_roots {
            for name in prelude_names {
                push_unique_string(root_names, name);
            }
        }
        index = prelude_index.saturating_add(1);
    }
}

fn collect_css_modules_value_references_in_at_rule_preludes_from_ir(
    ir: &TransformIrV0,
    dialect: StyleDialect,
    definition_names: &[String],
    root_names: &mut Vec<String>,
    reachable_class_names: &[String],
    scope_blocks: &[CssModuleScopeBlock],
) {
    for at_rule in collect_css_modules_value_at_rule_preludes_from_ir(ir) {
        if !at_rule_prelude_can_reference_css_modules_values(at_rule.keyword) {
            continue;
        }
        let prelude_names = collect_css_modules_value_references_in_ir_at_rule_prelude(
            at_rule.keyword,
            at_rule.prelude,
            dialect,
            definition_names,
        );
        let prelude_can_keep_roots = match at_rule.terminator {
            CssModulesValueAtRulePreludeTerminatorV0::Block {
                block_start,
                block_end,
            } => at_rule_block_has_reachable_ordinary_rule_from_ir(
                ir,
                block_start,
                block_end,
                reachable_class_names,
                scope_blocks,
            ),
            CssModulesValueAtRulePreludeTerminatorV0::Semicolon => true,
            CssModulesValueAtRulePreludeTerminatorV0::Unknown => true,
        };
        if prelude_can_keep_roots {
            for name in prelude_names {
                push_unique_string(root_names, name);
            }
        }
    }
}

fn collect_css_modules_value_references_in_ir_at_rule_prelude(
    at_rule_text: &str,
    prelude: &str,
    dialect: StyleDialect,
    definition_names: &[String],
) -> Vec<String> {
    let lexed = lex(prelude, dialect);
    let tokens = lexed.tokens();
    let mut names = Vec::new();
    for (candidate_index, token) in tokens.iter().enumerate() {
        if token.kind == SyntaxKind::Ident
            && definition_names.iter().any(|name| name == &token.text)
            && query_prelude_ident_is_css_modules_value_reference(
                tokens,
                candidate_index,
                0,
                at_rule_text,
            )
        {
            push_unique_string(&mut names, token.text.clone());
        }
    }
    names
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum CssModulesValueAtRulePreludeTerminatorV0 {
    Block {
        block_start: usize,
        block_end: usize,
    },
    Semicolon,
    Unknown,
}

struct CssModulesValueAtRulePreludeIrViewV0<'source> {
    keyword: &'source str,
    prelude: &'source str,
    prelude_start: usize,
    terminator: CssModulesValueAtRulePreludeTerminatorV0,
}

fn collect_css_modules_value_at_rule_preludes_from_ir(
    ir: &TransformIrV0,
) -> Vec<CssModulesValueAtRulePreludeIrViewV0<'_>> {
    let mut at_rules = ir
        .nodes
        .iter()
        .filter(|node| !node.deleted && node.kind == IrNodeKindV0::AtRule)
        .filter_map(|node| css_modules_value_at_rule_prelude_from_ir(ir, node))
        .collect::<Vec<_>>();
    at_rules.sort_by_key(|at_rule| at_rule.prelude_start);
    at_rules
}

fn css_modules_value_at_rule_prelude_from_ir<'source>(
    ir: &'source TransformIrV0,
    node: &IrNodeV0,
) -> Option<CssModulesValueAtRulePreludeIrViewV0<'source>> {
    let source = ir.source_text();
    let (keyword, keyword_end) = ir_at_rule_keyword_with_end(source, node)?;
    let terminator =
        at_rule_prelude_terminator_from_source(source, keyword_end, node.source_span_end)
            .unwrap_or(CssModulesValueAtRulePreludeTerminatorV0::Unknown);
    let prelude_end = match terminator {
        CssModulesValueAtRulePreludeTerminatorV0::Block { block_start, .. } => block_start,
        CssModulesValueAtRulePreludeTerminatorV0::Semicolon
        | CssModulesValueAtRulePreludeTerminatorV0::Unknown => node.source_span_end,
    };
    let prelude = source.get(keyword_end..prelude_end)?;
    Some(CssModulesValueAtRulePreludeIrViewV0 {
        keyword,
        prelude,
        prelude_start: keyword_end,
        terminator,
    })
}

fn at_rule_prelude_can_reference_css_modules_values(text: &str) -> bool {
    matches!(
        text.to_ascii_lowercase().as_str(),
        "@media" | "@supports" | "@container" | "@custom-media" | "@scope"
    )
}

pub(crate) fn at_rule_block_has_reachable_ordinary_rule(
    source: &str,
    tokens: &[omena_parser::LexedToken],
    block_start_index: usize,
    block_end_index: usize,
    reachable_class_names: &[String],
    scope_blocks: &[CssModuleScopeBlock],
) -> bool {
    let context_start = token_start(&tokens[block_start_index]);
    let context_end = token_end(&tokens[block_end_index]);

    collect_declaration_ordinary_rule_slices(source, tokens)
        .iter()
        .any(|rule| {
            rule.context_start >= context_start
                && rule.context_end <= context_end
                && rule_slice_matches_reachable_class_context(
                    rule,
                    scope_blocks,
                    reachable_class_names,
                )
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
        crate::domains::css_module_global::CssModuleScopeBlockKind::Local
    } else if selector.eq_ignore_ascii_case(":global") {
        crate::domains::css_module_global::CssModuleScopeBlockKind::Global
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
struct CssModulesValueDeclarationIrViewV0 {
    property: String,
    value: String,
    start: usize,
    end: usize,
}

fn collect_simple_declarations_from_ir(
    ir: &TransformIrV0,
    rule: &SimpleRuleSlice,
) -> Vec<CssModulesValueDeclarationIrViewV0> {
    collect_simple_declarations_between_from_ir(ir, rule.block_start, rule.block_end)
}

fn collect_simple_declarations_between_from_ir(
    ir: &TransformIrV0,
    block_start: usize,
    block_end: usize,
) -> Vec<CssModulesValueDeclarationIrViewV0> {
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

fn simple_declaration_from_ir(
    ir: &TransformIrV0,
    node: &IrNodeV0,
) -> Option<CssModulesValueDeclarationIrViewV0> {
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
    Some(CssModulesValueDeclarationIrViewV0 {
        property: property.to_ascii_lowercase(),
        value: value.to_string(),
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

fn ir_at_rule_keyword<'source>(source: &'source str, node: &IrNodeV0) -> Option<&'source str> {
    ir_at_rule_keyword_with_end(source, node).map(|(keyword, _)| keyword)
}

fn ir_at_rule_keyword_with_end<'source>(
    source: &'source str,
    node: &IrNodeV0,
) -> Option<(&'source str, usize)> {
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
    Some((source.get(keyword_start..keyword_end)?, keyword_end))
}

fn at_rule_prelude_terminator_from_source(
    source: &str,
    start: usize,
    end: usize,
) -> Option<CssModulesValueAtRulePreludeTerminatorV0> {
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
                return Some(CssModulesValueAtRulePreludeTerminatorV0::Block {
                    block_start: index,
                    block_end: index + close + 1,
                });
            }
            b';' if paren_depth == 0 && bracket_depth == 0 => {
                return Some(CssModulesValueAtRulePreludeTerminatorV0::Semicolon);
            }
            _ => {}
        }
        index += 1;
    }

    None
}

fn source_text_contains_comment(source: &str) -> bool {
    source.as_bytes().windows(2).any(|bytes| bytes == b"/*")
}

fn close_css_modules_value_dependency_graph(
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

fn can_tree_shake_local_css_modules_value_definition(
    definition: &StaticCssModulesValueDefinition,
    definitions: &[StaticCssModulesValueDefinition],
) -> bool {
    definitions
        .iter()
        .filter(|candidate| candidate.name == definition.name)
        .count()
        == 1
}

fn collect_css_modules_value_references_in_value(
    value: &str,
    dialect: StyleDialect,
    definition_names: &[String],
) -> Vec<String> {
    let lexed = lex(value, dialect);
    let mut references = Vec::new();
    for token in lexed.tokens() {
        if token.kind == SyntaxKind::Ident
            && definition_names.iter().any(|name| name == &token.text)
        {
            push_unique_string(&mut references, token.text.clone());
        }
    }
    references
}

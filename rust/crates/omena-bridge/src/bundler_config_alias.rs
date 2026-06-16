use std::{
    collections::VecDeque,
    fs,
    path::{Path, PathBuf},
};

use omena_resolver::OmenaResolverBundlerPathAliasMappingV0;
use oxc_allocator::Allocator;
use oxc_ast::ast::{
    ArrayExpression, ArrayExpressionElement, AssignmentOperator, Expression, ObjectExpression,
    ObjectPropertyKind, Program, Statement,
};
use oxc_parser::{Parser, ParserReturn};
use oxc_span::{GetSpan, SourceType, Span};
use serde::Serialize;

mod literals;
mod paths;
mod syntax;

use literals::{
    find_top_level_array_literal, find_top_level_literal, top_level_array_literals,
    top_level_object_literals, unwrap_config_expression, unwrap_export_default_declaration_kind,
    unwrap_static_object_expression,
};
use paths::{static_replacement_path, static_string};
use syntax::{is_module_exports_target, property_key_text};

const BUNDLER_CONFIG_NAMES: [&str; 18] = [
    "next.config.ts",
    "next.config.mts",
    "next.config.cts",
    "next.config.js",
    "next.config.mjs",
    "next.config.cjs",
    "vite.config.ts",
    "vite.config.mts",
    "vite.config.cts",
    "vite.config.js",
    "vite.config.mjs",
    "vite.config.cjs",
    "webpack.config.ts",
    "webpack.config.mts",
    "webpack.config.cts",
    "webpack.config.js",
    "webpack.config.mjs",
    "webpack.config.cjs",
];
const BUNDLER_CONFIG_DISCOVERY_DIR_LIMIT: usize = 256;
const BUNDLER_CONFIG_DISCOVERY_RESULT_LIMIT: usize = 32;
const BUNDLER_CONFIG_DISCOVERY_MAX_DEPTH: usize = 3;

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OmenaBridgeBundlerPathAliasSummaryV0 {
    pub schema_version: &'static str,
    pub product: &'static str,
    pub aliases: Vec<OmenaBridgeBundlerPathAliasMappingV0>,
    pub unrecognized: Vec<OmenaBridgeBundlerAliasUnrecognizedEntryV0>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OmenaBridgeBundlerPathAliasMappingV0 {
    pub config_path: String,
    pub pattern: String,
    pub target_path: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OmenaBridgeBundlerAliasUnrecognizedEntryV0 {
    pub config_path: String,
    pub reason: &'static str,
    pub text: String,
}

enum ExportedConfigObject<'a> {
    Static(&'a ObjectExpression<'a>),
    Dynamic(Span),
}

pub fn load_omena_bridge_workspace_bundler_path_alias_mappings(
    workspace_path: Option<&Path>,
) -> Vec<OmenaResolverBundlerPathAliasMappingV0> {
    let Some(workspace_path) = workspace_path else {
        return Vec::new();
    };
    let mut mappings = Vec::new();
    for config_path in workspace_bundler_config_paths(workspace_path) {
        let Some(config_source) = fs::read_to_string(config_path.as_path()).ok() else {
            continue;
        };
        let summary = summarize_omena_bridge_bundler_path_aliases_for_config(
            config_path.as_path(),
            config_source.as_str(),
        );
        for alias in summary.aliases {
            mappings.push(OmenaResolverBundlerPathAliasMappingV0 {
                pattern: alias.pattern,
                target_path: alias.target_path,
            });
        }
    }
    mappings
}

pub fn summarize_omena_bridge_bundler_path_aliases_for_config(
    config_path: &Path,
    config_source: &str,
) -> OmenaBridgeBundlerPathAliasSummaryV0 {
    let allocator = Allocator::default();
    let source_type = SourceType::from_path(config_path).unwrap_or_else(|_| SourceType::ts());
    let ParserReturn {
        program, panicked, ..
    } = Parser::new(&allocator, config_source, source_type).parse();

    let config_path_text = config_path.to_string_lossy().to_string();
    let mut aliases = Vec::new();
    let mut unrecognized = Vec::new();
    if !panicked {
        collect_bundler_aliases_from_program(
            &program,
            config_path,
            config_source,
            config_path_text.as_str(),
            &mut aliases,
            &mut unrecognized,
        );
    }

    OmenaBridgeBundlerPathAliasSummaryV0 {
        schema_version: "0",
        product: "omena-bridge.bundler-path-aliases",
        aliases,
        unrecognized,
    }
}

fn workspace_bundler_config_paths(workspace_path: &Path) -> Vec<PathBuf> {
    let mut paths = Vec::new();
    let mut queue = VecDeque::from([(workspace_path.to_path_buf(), 0usize)]);
    let mut visited = 0usize;
    while let Some((dir, depth)) = queue.pop_front() {
        if visited >= BUNDLER_CONFIG_DISCOVERY_DIR_LIMIT
            || paths.len() >= BUNDLER_CONFIG_DISCOVERY_RESULT_LIMIT
        {
            break;
        }
        visited += 1;
        for name in BUNDLER_CONFIG_NAMES {
            let path = dir.join(name);
            if path.exists() {
                paths.push(path);
                if paths.len() >= BUNDLER_CONFIG_DISCOVERY_RESULT_LIMIT {
                    break;
                }
            }
        }
        if depth >= BUNDLER_CONFIG_DISCOVERY_MAX_DEPTH {
            continue;
        }
        let Ok(entries) = fs::read_dir(dir.as_path()) else {
            continue;
        };
        for entry in entries.flatten() {
            let path = entry.path();
            let Some(name) = path.file_name().and_then(|name| name.to_str()) else {
                continue;
            };
            if !path.is_dir() || should_skip_bundler_config_discovery_dir(name) {
                continue;
            }
            queue.push_back((path, depth + 1));
        }
    }
    paths.sort();
    paths.dedup();
    paths
}

fn should_skip_bundler_config_discovery_dir(name: &str) -> bool {
    matches!(
        name,
        ".git"
            | ".next"
            | ".nuxt"
            | ".svelte-kit"
            | "coverage"
            | "dist"
            | "node_modules"
            | "target"
    )
}

fn collect_bundler_aliases_from_program<'a>(
    program: &'a Program<'a>,
    config_path: &Path,
    config_source: &str,
    config_path_text: &str,
    aliases: &mut Vec<OmenaBridgeBundlerPathAliasMappingV0>,
    unrecognized: &mut Vec<OmenaBridgeBundlerAliasUnrecognizedEntryV0>,
) {
    let top_level_literals = top_level_object_literals(program);
    let top_level_arrays = top_level_array_literals(program);
    let mut exported_objects = Vec::new();
    let mut saw_config_export = false;
    for statement in &program.body {
        match exported_config_object_from_statement(statement, &top_level_literals) {
            Some(ExportedConfigObject::Static(object)) => {
                saw_config_export = true;
                exported_objects.push(object);
            }
            Some(ExportedConfigObject::Dynamic(span)) => {
                saw_config_export = true;
                push_unrecognized(
                    config_path_text,
                    "dynamic-config-export",
                    config_source,
                    span,
                    unrecognized,
                );
            }
            None => {}
        }
    }

    if exported_objects.is_empty() && !saw_config_export {
        exported_objects.extend(top_level_literals.iter().map(|(_, object)| *object));
    }

    let alias_context = AliasCollectionContext {
        config_path,
        config_source,
        config_path_text,
        top_level_literals: top_level_literals.as_slice(),
        top_level_arrays: top_level_arrays.as_slice(),
    };

    for object in exported_objects {
        let Some(alias_expression) = resolve_alias_expression(
            object,
            top_level_literals.as_slice(),
            config_source,
            config_path_text,
            unrecognized,
        ) else {
            continue;
        };
        collect_alias_expression(&alias_context, alias_expression, aliases, unrecognized);
    }
}

fn exported_config_object_from_statement<'a>(
    statement: &'a Statement<'a>,
    top_level_literals: &[(String, &'a ObjectExpression<'a>)],
) -> Option<ExportedConfigObject<'a>> {
    match statement {
        Statement::ExportDefaultDeclaration(declaration) => {
            match unwrap_export_default_declaration_kind(
                &declaration.declaration,
                top_level_literals,
            ) {
                Some(object) => Some(ExportedConfigObject::Static(object)),
                None => Some(ExportedConfigObject::Dynamic(declaration.span)),
            }
        }
        Statement::ExpressionStatement(statement) => {
            let Expression::AssignmentExpression(assignment) = &statement.expression else {
                return None;
            };
            if assignment.operator != AssignmentOperator::Assign
                || !is_module_exports_target(&assignment.left)
            {
                return None;
            }
            match unwrap_config_expression(&assignment.right, top_level_literals) {
                Some(object) => Some(ExportedConfigObject::Static(object)),
                None => Some(ExportedConfigObject::Dynamic(assignment.right.span())),
            }
        }
        _ => None,
    }
}

fn resolve_alias_expression<'a>(
    object: &'a ObjectExpression<'a>,
    top_level_literals: &[(String, &'a ObjectExpression<'a>)],
    config_source: &str,
    config_path_text: &str,
    unrecognized: &mut Vec<OmenaBridgeBundlerAliasUnrecognizedEntryV0>,
) -> Option<&'a Expression<'a>> {
    let resolve_expression = object_property_value(object, "resolve")?;
    let Some(resolve_object) = unwrap_config_expression(resolve_expression, top_level_literals)
    else {
        push_unrecognized(
            config_path_text,
            "dynamic-alias-container",
            config_source,
            resolve_expression.span(),
            unrecognized,
        );
        return None;
    };
    object_property_value(resolve_object, "alias")
}

fn object_property_value<'a>(
    object: &'a ObjectExpression<'a>,
    name: &str,
) -> Option<&'a Expression<'a>> {
    for property in &object.properties {
        let ObjectPropertyKind::ObjectProperty(property) = property else {
            continue;
        };
        if property.computed || property_key_text(&property.key) != Some(name) {
            continue;
        }
        return Some(&property.value);
    }
    None
}

struct AliasCollectionContext<'a, 'b> {
    config_path: &'a Path,
    config_source: &'a str,
    config_path_text: &'a str,
    top_level_literals: &'a [(String, &'b ObjectExpression<'b>)],
    top_level_arrays: &'a [(String, &'b ArrayExpression<'b>)],
}

fn collect_alias_expression<'a>(
    context: &AliasCollectionContext<'_, 'a>,
    expression: &Expression<'a>,
    aliases: &mut Vec<OmenaBridgeBundlerPathAliasMappingV0>,
    unrecognized: &mut Vec<OmenaBridgeBundlerAliasUnrecognizedEntryV0>,
) {
    match expression {
        Expression::ObjectExpression(object) => collect_object_alias_entries(
            context.config_path,
            context.config_source,
            context.config_path_text,
            object,
            aliases,
            unrecognized,
        ),
        Expression::ArrayExpression(array) => collect_array_alias_entries(
            context.config_path,
            context.config_source,
            context.config_path_text,
            array,
            aliases,
            unrecognized,
        ),
        Expression::Identifier(identifier) => {
            if let Some(object) =
                find_top_level_literal(context.top_level_literals, identifier.name.as_str())
            {
                collect_object_alias_entries(
                    context.config_path,
                    context.config_source,
                    context.config_path_text,
                    object,
                    aliases,
                    unrecognized,
                );
            } else if let Some(array) =
                find_top_level_array_literal(context.top_level_arrays, identifier.name.as_str())
            {
                collect_array_alias_entries(
                    context.config_path,
                    context.config_source,
                    context.config_path_text,
                    array,
                    aliases,
                    unrecognized,
                );
            } else {
                push_unrecognized(
                    context.config_path_text,
                    "dynamic-alias-container",
                    context.config_source,
                    expression.span(),
                    unrecognized,
                );
            }
        }
        Expression::ParenthesizedExpression(expression) => {
            collect_alias_expression(context, &expression.expression, aliases, unrecognized)
        }
        Expression::TSAsExpression(expression) => {
            collect_alias_expression(context, &expression.expression, aliases, unrecognized)
        }
        Expression::TSSatisfiesExpression(expression) => {
            collect_alias_expression(context, &expression.expression, aliases, unrecognized)
        }
        _ => push_unrecognized(
            context.config_path_text,
            "dynamic-alias-container",
            context.config_source,
            expression.span(),
            unrecognized,
        ),
    }
}

fn collect_object_alias_entries(
    config_path: &Path,
    config_source: &str,
    config_path_text: &str,
    object: &ObjectExpression<'_>,
    aliases: &mut Vec<OmenaBridgeBundlerPathAliasMappingV0>,
    unrecognized: &mut Vec<OmenaBridgeBundlerAliasUnrecognizedEntryV0>,
) {
    for property in &object.properties {
        let ObjectPropertyKind::ObjectProperty(property) = property else {
            push_unrecognized(
                config_path_text,
                "dynamic-alias-entry",
                config_source,
                property.span(),
                unrecognized,
            );
            continue;
        };
        if property.computed {
            push_unrecognized(
                config_path_text,
                "dynamic-alias-entry",
                config_source,
                property.span,
                unrecognized,
            );
            continue;
        }
        let Some(pattern) = property_key_text(&property.key) else {
            push_unrecognized(
                config_path_text,
                "dynamic-alias-entry",
                config_source,
                property.span,
                unrecognized,
            );
            continue;
        };
        let Some(target_path) = static_replacement_path(config_path, &property.value) else {
            push_unrecognized(
                config_path_text,
                "dynamic-alias-replacement",
                config_source,
                property.value.span(),
                unrecognized,
            );
            continue;
        };
        upsert_alias(
            aliases,
            OmenaBridgeBundlerPathAliasMappingV0 {
                config_path: config_path_text.to_string(),
                pattern: pattern.to_string(),
                target_path,
            },
        );
    }
}

fn collect_array_alias_entries(
    config_path: &Path,
    config_source: &str,
    config_path_text: &str,
    array: &ArrayExpression<'_>,
    aliases: &mut Vec<OmenaBridgeBundlerPathAliasMappingV0>,
    unrecognized: &mut Vec<OmenaBridgeBundlerAliasUnrecognizedEntryV0>,
) {
    for element in &array.elements {
        let Some(object) = array_element_object_expression(element) else {
            push_unrecognized(
                config_path_text,
                "dynamic-alias-entry",
                config_source,
                element.span(),
                unrecognized,
            );
            continue;
        };
        let Some(find_expression) = object_property_value(object, "find") else {
            push_unrecognized(
                config_path_text,
                "dynamic-alias-entry",
                config_source,
                object.span,
                unrecognized,
            );
            continue;
        };
        if matches!(find_expression, Expression::RegExpLiteral(_)) {
            push_unrecognized(
                config_path_text,
                "regex-alias-find",
                config_source,
                find_expression.span(),
                unrecognized,
            );
            continue;
        }
        let Some(pattern) = static_string(find_expression) else {
            push_unrecognized(
                config_path_text,
                "dynamic-alias-entry",
                config_source,
                find_expression.span(),
                unrecognized,
            );
            continue;
        };
        let Some(replacement_expression) = object_property_value(object, "replacement") else {
            push_unrecognized(
                config_path_text,
                "dynamic-alias-entry",
                config_source,
                object.span,
                unrecognized,
            );
            continue;
        };
        let Some(target_path) = static_replacement_path(config_path, replacement_expression) else {
            push_unrecognized(
                config_path_text,
                "dynamic-alias-replacement",
                config_source,
                replacement_expression.span(),
                unrecognized,
            );
            continue;
        };
        aliases.push(OmenaBridgeBundlerPathAliasMappingV0 {
            config_path: config_path_text.to_string(),
            pattern,
            target_path,
        });
    }
}

fn array_element_object_expression<'a>(
    element: &'a ArrayExpressionElement<'a>,
) -> Option<&'a ObjectExpression<'a>> {
    match element {
        ArrayExpressionElement::ObjectExpression(object) => Some(object),
        ArrayExpressionElement::ParenthesizedExpression(expression) => {
            unwrap_static_object_expression(&expression.expression)
        }
        ArrayExpressionElement::TSAsExpression(expression) => {
            unwrap_static_object_expression(&expression.expression)
        }
        ArrayExpressionElement::TSSatisfiesExpression(expression) => {
            unwrap_static_object_expression(&expression.expression)
        }
        _ => None,
    }
}

fn upsert_alias(
    aliases: &mut Vec<OmenaBridgeBundlerPathAliasMappingV0>,
    alias: OmenaBridgeBundlerPathAliasMappingV0,
) {
    if let Some(index) = aliases
        .iter()
        .position(|entry| entry.pattern == alias.pattern)
    {
        aliases.remove(index);
    }
    aliases.push(alias);
}

fn push_unrecognized(
    config_path: &str,
    reason: &'static str,
    config_source: &str,
    span: Span,
    unrecognized: &mut Vec<OmenaBridgeBundlerAliasUnrecognizedEntryV0>,
) {
    unrecognized.push(OmenaBridgeBundlerAliasUnrecognizedEntryV0 {
        config_path: config_path.to_string(),
        reason,
        text: config_source
            .get(span.start as usize..span.end as usize)
            .unwrap_or("")
            .to_string(),
    });
}

#[cfg(test)]
mod tests;

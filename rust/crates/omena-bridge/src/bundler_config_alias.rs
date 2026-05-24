use std::{
    fs,
    path::{Path, PathBuf},
};

use omena_resolver::OmenaResolverBundlerPathAliasMappingV0;
use oxc_allocator::Allocator;
use oxc_ast::ast::{
    Argument, ArrayExpression, ArrayExpressionElement, AssignmentOperator,
    ExportDefaultDeclarationKind, Expression, ObjectExpression, ObjectPropertyKind, Program,
    Statement,
};
use oxc_parser::{Parser, ParserReturn};
use oxc_span::{GetSpan, SourceType, Span};
use serde::Serialize;

mod paths;
mod syntax;

use paths::{static_replacement_path, static_string};
use syntax::{
    binding_pattern_identifier_name, expression_identifier_name, is_module_exports_target,
    property_key_text,
};

const BUNDLER_CONFIG_NAMES: [&str; 12] = [
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
    BUNDLER_CONFIG_NAMES
        .iter()
        .map(|name| workspace_path.join(name))
        .filter(|path| path.exists())
        .collect()
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
        collect_alias_expression(
            config_path,
            config_source,
            config_path_text,
            alias_expression,
            top_level_literals.as_slice(),
            top_level_arrays.as_slice(),
            aliases,
            unrecognized,
        );
    }
}

fn top_level_object_literals<'a>(
    program: &'a Program<'a>,
) -> Vec<(String, &'a ObjectExpression<'a>)> {
    let mut literals = Vec::new();
    for statement in &program.body {
        let Statement::VariableDeclaration(declaration) = statement else {
            continue;
        };
        for declarator in &declaration.declarations {
            let Some(binding) = binding_pattern_identifier_name(&declarator.id) else {
                continue;
            };
            let Some(init) = declarator.init.as_ref() else {
                continue;
            };
            if let Some(object) = unwrap_config_expression(init, literals.as_slice()) {
                upsert_top_level_literal(&mut literals, binding.to_string(), object);
            }
        }
    }
    literals
}

fn top_level_array_literals<'a>(
    program: &'a Program<'a>,
) -> Vec<(String, &'a ArrayExpression<'a>)> {
    let mut literals = Vec::new();
    for statement in &program.body {
        let Statement::VariableDeclaration(declaration) = statement else {
            continue;
        };
        for declarator in &declaration.declarations {
            let Some(binding) = binding_pattern_identifier_name(&declarator.id) else {
                continue;
            };
            let Some(init) = declarator.init.as_ref() else {
                continue;
            };
            if let Some(array) = unwrap_array_expression(init, literals.as_slice()) {
                upsert_top_level_array_literal(&mut literals, binding.to_string(), array);
            }
        }
    }
    literals
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

fn unwrap_export_default_declaration_kind<'a>(
    declaration: &'a ExportDefaultDeclarationKind<'a>,
    top_level_literals: &[(String, &'a ObjectExpression<'a>)],
) -> Option<&'a ObjectExpression<'a>> {
    match declaration {
        ExportDefaultDeclarationKind::ObjectExpression(object) => Some(object),
        ExportDefaultDeclarationKind::Identifier(identifier) => {
            find_top_level_literal(top_level_literals, identifier.name.as_str())
        }
        ExportDefaultDeclarationKind::ParenthesizedExpression(expression) => {
            unwrap_config_expression(&expression.expression, top_level_literals)
        }
        ExportDefaultDeclarationKind::TSAsExpression(expression) => {
            unwrap_config_expression(&expression.expression, top_level_literals)
        }
        ExportDefaultDeclarationKind::TSSatisfiesExpression(expression) => {
            unwrap_config_expression(&expression.expression, top_level_literals)
        }
        ExportDefaultDeclarationKind::CallExpression(expression) => {
            unwrap_config_call_expression(expression, top_level_literals)
        }
        _ => None,
    }
}

fn unwrap_config_expression<'a>(
    expression: &'a Expression<'a>,
    top_level_literals: &[(String, &'a ObjectExpression<'a>)],
) -> Option<&'a ObjectExpression<'a>> {
    match expression {
        Expression::ObjectExpression(object) => Some(object),
        Expression::Identifier(identifier) => {
            find_top_level_literal(top_level_literals, identifier.name.as_str())
        }
        Expression::ParenthesizedExpression(expression) => {
            unwrap_config_expression(&expression.expression, top_level_literals)
        }
        Expression::TSAsExpression(expression) => {
            unwrap_config_expression(&expression.expression, top_level_literals)
        }
        Expression::TSSatisfiesExpression(expression) => {
            unwrap_config_expression(&expression.expression, top_level_literals)
        }
        Expression::CallExpression(expression) => {
            unwrap_config_call_expression(expression, top_level_literals)
        }
        _ => None,
    }
}

fn unwrap_config_call_expression<'a>(
    expression: &'a oxc_ast::ast::CallExpression<'a>,
    top_level_literals: &[(String, &'a ObjectExpression<'a>)],
) -> Option<&'a ObjectExpression<'a>> {
    if expression_identifier_name(&expression.callee) != Some("defineConfig") {
        return None;
    }
    expression
        .arguments
        .first()
        .and_then(|argument| unwrap_config_argument(argument, top_level_literals))
}

fn unwrap_config_argument<'a>(
    argument: &'a Argument<'a>,
    top_level_literals: &[(String, &'a ObjectExpression<'a>)],
) -> Option<&'a ObjectExpression<'a>> {
    match argument {
        Argument::ObjectExpression(object) => Some(object),
        Argument::Identifier(identifier) => {
            find_top_level_literal(top_level_literals, identifier.name.as_str())
        }
        Argument::ParenthesizedExpression(expression) => {
            unwrap_config_expression(&expression.expression, top_level_literals)
        }
        Argument::TSAsExpression(expression) => {
            unwrap_config_expression(&expression.expression, top_level_literals)
        }
        Argument::TSSatisfiesExpression(expression) => {
            unwrap_config_expression(&expression.expression, top_level_literals)
        }
        Argument::CallExpression(expression) => {
            unwrap_config_call_expression(expression, top_level_literals)
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

fn unwrap_static_object_expression<'a>(
    expression: &'a Expression<'a>,
) -> Option<&'a ObjectExpression<'a>> {
    match expression {
        Expression::ObjectExpression(object) => Some(object),
        Expression::ParenthesizedExpression(expression) => {
            unwrap_static_object_expression(&expression.expression)
        }
        Expression::TSAsExpression(expression) => {
            unwrap_static_object_expression(&expression.expression)
        }
        Expression::TSSatisfiesExpression(expression) => {
            unwrap_static_object_expression(&expression.expression)
        }
        _ => None,
    }
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

fn collect_alias_expression(
    config_path: &Path,
    config_source: &str,
    config_path_text: &str,
    expression: &Expression<'_>,
    top_level_literals: &[(String, &ObjectExpression<'_>)],
    top_level_arrays: &[(String, &ArrayExpression<'_>)],
    aliases: &mut Vec<OmenaBridgeBundlerPathAliasMappingV0>,
    unrecognized: &mut Vec<OmenaBridgeBundlerAliasUnrecognizedEntryV0>,
) {
    match expression {
        Expression::ObjectExpression(object) => collect_object_alias_entries(
            config_path,
            config_source,
            config_path_text,
            object,
            aliases,
            unrecognized,
        ),
        Expression::ArrayExpression(array) => collect_array_alias_entries(
            config_path,
            config_source,
            config_path_text,
            array,
            aliases,
            unrecognized,
        ),
        Expression::Identifier(identifier) => {
            if let Some(object) =
                find_top_level_literal(top_level_literals, identifier.name.as_str())
            {
                collect_object_alias_entries(
                    config_path,
                    config_source,
                    config_path_text,
                    object,
                    aliases,
                    unrecognized,
                );
            } else if let Some(array) =
                find_top_level_array_literal(top_level_arrays, identifier.name.as_str())
            {
                collect_array_alias_entries(
                    config_path,
                    config_source,
                    config_path_text,
                    array,
                    aliases,
                    unrecognized,
                );
            } else {
                push_unrecognized(
                    config_path_text,
                    "dynamic-alias-container",
                    config_source,
                    expression.span(),
                    unrecognized,
                );
            }
        }
        Expression::ParenthesizedExpression(expression) => collect_alias_expression(
            config_path,
            config_source,
            config_path_text,
            &expression.expression,
            top_level_literals,
            top_level_arrays,
            aliases,
            unrecognized,
        ),
        Expression::TSAsExpression(expression) => collect_alias_expression(
            config_path,
            config_source,
            config_path_text,
            &expression.expression,
            top_level_literals,
            top_level_arrays,
            aliases,
            unrecognized,
        ),
        Expression::TSSatisfiesExpression(expression) => collect_alias_expression(
            config_path,
            config_source,
            config_path_text,
            &expression.expression,
            top_level_literals,
            top_level_arrays,
            aliases,
            unrecognized,
        ),
        _ => push_unrecognized(
            config_path_text,
            "dynamic-alias-container",
            config_source,
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

fn unwrap_array_expression<'a>(
    expression: &'a Expression<'a>,
    top_level_arrays: &[(String, &'a ArrayExpression<'a>)],
) -> Option<&'a ArrayExpression<'a>> {
    match expression {
        Expression::ArrayExpression(array) => Some(array),
        Expression::Identifier(identifier) => {
            find_top_level_array_literal(top_level_arrays, identifier.name.as_str())
        }
        Expression::ParenthesizedExpression(expression) => {
            unwrap_array_expression(&expression.expression, top_level_arrays)
        }
        Expression::TSAsExpression(expression) => {
            unwrap_array_expression(&expression.expression, top_level_arrays)
        }
        Expression::TSSatisfiesExpression(expression) => {
            unwrap_array_expression(&expression.expression, top_level_arrays)
        }
        _ => None,
    }
}

fn find_top_level_literal<'a>(
    literals: &[(String, &'a ObjectExpression<'a>)],
    name: &str,
) -> Option<&'a ObjectExpression<'a>> {
    literals
        .iter()
        .find_map(|(literal_name, object)| (literal_name == name).then_some(*object))
}

fn find_top_level_array_literal<'a>(
    literals: &[(String, &'a ArrayExpression<'a>)],
    name: &str,
) -> Option<&'a ArrayExpression<'a>> {
    literals
        .iter()
        .find_map(|(literal_name, array)| (literal_name == name).then_some(*array))
}

fn upsert_top_level_literal<'a>(
    literals: &mut Vec<(String, &'a ObjectExpression<'a>)>,
    name: String,
    object: &'a ObjectExpression<'a>,
) {
    if let Some(index) = literals
        .iter()
        .position(|(literal_name, _)| literal_name == &name)
    {
        literals.remove(index);
    }
    literals.push((name, object));
}

fn upsert_top_level_array_literal<'a>(
    literals: &mut Vec<(String, &'a ArrayExpression<'a>)>,
    name: String,
    array: &'a ArrayExpression<'a>,
) {
    if let Some(index) = literals
        .iter()
        .position(|(literal_name, _)| literal_name == &name)
    {
        literals.remove(index);
    }
    literals.push((name, array));
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

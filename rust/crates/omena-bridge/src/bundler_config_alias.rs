use std::{
    fs,
    path::{Path, PathBuf},
};

use omena_resolver::OmenaResolverBundlerPathAliasMappingV0;
use oxc_allocator::Allocator;
use oxc_ast::ast::{
    Argument, ArrayExpression, ArrayExpressionElement, AssignmentOperator, AssignmentTarget,
    BindingPattern, ExportDefaultDeclarationKind, Expression, ObjectExpression, ObjectPropertyKind,
    Program, PropertyKey, Statement,
};
use oxc_parser::{Parser, ParserReturn};
use oxc_span::{GetSpan, SourceType, Span};
use serde::Serialize;

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
            upsert_resolver_mapping(
                &mut mappings,
                OmenaResolverBundlerPathAliasMappingV0 {
                    pattern: alias.pattern,
                    target_path: alias.target_path,
                },
            );
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
    let mut exported_objects = Vec::new();
    for statement in &program.body {
        if let Some(object) = exported_config_object_from_statement(statement, &top_level_literals)
        {
            exported_objects.push(object);
        }
    }

    if exported_objects.is_empty() {
        exported_objects.extend(top_level_literals.iter().map(|(_, object)| *object));
    }

    for object in exported_objects {
        let Some(alias_expression) =
            resolve_alias_expression(object, config_source, config_path_text, unrecognized)
        else {
            continue;
        };
        collect_alias_expression(
            config_path,
            config_source,
            config_path_text,
            alias_expression,
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

fn exported_config_object_from_statement<'a>(
    statement: &'a Statement<'a>,
    top_level_literals: &[(String, &'a ObjectExpression<'a>)],
) -> Option<&'a ObjectExpression<'a>> {
    match statement {
        Statement::ExportDefaultDeclaration(declaration) => {
            unwrap_export_default_declaration_kind(&declaration.declaration, top_level_literals)
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
            unwrap_config_expression(&assignment.right, top_level_literals)
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
    config_source: &str,
    config_path_text: &str,
    unrecognized: &mut Vec<OmenaBridgeBundlerAliasUnrecognizedEntryV0>,
) -> Option<&'a Expression<'a>> {
    let resolve_expression = object_property_value(object, "resolve")?;
    let Some(resolve_object) = unwrap_static_object_expression(resolve_expression) else {
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
        Expression::ParenthesizedExpression(expression) => collect_alias_expression(
            config_path,
            config_source,
            config_path_text,
            &expression.expression,
            aliases,
            unrecognized,
        ),
        Expression::TSAsExpression(expression) => collect_alias_expression(
            config_path,
            config_source,
            config_path_text,
            &expression.expression,
            aliases,
            unrecognized,
        ),
        Expression::TSSatisfiesExpression(expression) => collect_alias_expression(
            config_path,
            config_source,
            config_path_text,
            &expression.expression,
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
        upsert_alias(
            aliases,
            OmenaBridgeBundlerPathAliasMappingV0 {
                config_path: config_path_text.to_string(),
                pattern,
                target_path,
            },
        );
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

fn static_replacement_path(config_path: &Path, expression: &Expression<'_>) -> Option<String> {
    if let Some(value) = static_string(expression) {
        let target_path = PathBuf::from(value);
        if target_path.is_absolute() {
            return Some(target_path.to_string_lossy().to_string());
        }
        return Some(
            normalize_path_lexical(
                config_path
                    .parent()
                    .unwrap_or_else(|| Path::new("."))
                    .join(target_path),
            )
            .to_string_lossy()
            .to_string(),
        );
    }
    static_path_call(config_path, expression)
}

fn static_path_call(config_path: &Path, expression: &Expression<'_>) -> Option<String> {
    let Expression::CallExpression(call) = skip_parens_and_ts(expression) else {
        return None;
    };
    let Expression::StaticMemberExpression(callee) = skip_parens_and_ts(&call.callee) else {
        return None;
    };
    if expression_identifier_name(&callee.object) != Some("path")
        || (callee.property.name.as_str() != "resolve" && callee.property.name.as_str() != "join")
    {
        return None;
    }
    let mut path = PathBuf::new();
    for argument in &call.arguments {
        let segment = static_path_segment(config_path, argument)?;
        path.push(segment);
    }
    Some(normalize_path_lexical(path).to_string_lossy().to_string())
}

fn static_path_segment(config_path: &Path, argument: &Argument<'_>) -> Option<PathBuf> {
    if let Argument::Identifier(identifier) = argument
        && identifier.name.as_str() == "__dirname"
    {
        return Some(
            config_path
                .parent()
                .unwrap_or_else(|| Path::new("."))
                .to_path_buf(),
        );
    }
    static_string_from_argument(argument).map(PathBuf::from)
}

fn static_string_from_argument(argument: &Argument<'_>) -> Option<String> {
    match argument {
        Argument::StringLiteral(literal) => Some(literal.value.as_str().to_string()),
        Argument::TemplateLiteral(literal)
            if literal.expressions.is_empty() && literal.quasis.len() == 1 =>
        {
            literal.quasis[0]
                .value
                .cooked
                .map(|value| value.as_str().to_string())
        }
        Argument::ParenthesizedExpression(expression) => static_string(&expression.expression),
        Argument::TSAsExpression(expression) => static_string(&expression.expression),
        Argument::TSSatisfiesExpression(expression) => static_string(&expression.expression),
        _ => None,
    }
}

fn static_string(expression: &Expression<'_>) -> Option<String> {
    match skip_parens_and_ts(expression) {
        Expression::StringLiteral(literal) => Some(literal.value.as_str().to_string()),
        Expression::TemplateLiteral(literal)
            if literal.expressions.is_empty() && literal.quasis.len() == 1 =>
        {
            literal.quasis[0]
                .value
                .cooked
                .map(|value| value.as_str().to_string())
        }
        _ => None,
    }
}

fn skip_parens_and_ts<'a>(expression: &'a Expression<'a>) -> &'a Expression<'a> {
    match expression {
        Expression::ParenthesizedExpression(expression) => {
            skip_parens_and_ts(&expression.expression)
        }
        Expression::TSAsExpression(expression) => skip_parens_and_ts(&expression.expression),
        Expression::TSSatisfiesExpression(expression) => skip_parens_and_ts(&expression.expression),
        _ => expression,
    }
}

fn property_key_text<'a>(key: &'a PropertyKey<'a>) -> Option<&'a str> {
    match key {
        PropertyKey::StaticIdentifier(identifier) => Some(identifier.name.as_str()),
        PropertyKey::StringLiteral(literal) => Some(literal.value.as_str()),
        _ => None,
    }
}

fn binding_pattern_identifier_name<'a>(pattern: &'a BindingPattern<'a>) -> Option<&'a str> {
    match pattern {
        BindingPattern::BindingIdentifier(identifier) => Some(identifier.name.as_str()),
        _ => None,
    }
}

fn expression_identifier_name<'a>(expression: &'a Expression<'a>) -> Option<&'a str> {
    match skip_parens_and_ts(expression) {
        Expression::Identifier(identifier) => Some(identifier.name.as_str()),
        _ => None,
    }
}

fn is_module_exports_target(target: &AssignmentTarget<'_>) -> bool {
    let AssignmentTarget::StaticMemberExpression(member) = target else {
        return false;
    };
    expression_identifier_name(&member.object) == Some("module")
        && member.property.name.as_str() == "exports"
}

fn find_top_level_literal<'a>(
    literals: &[(String, &'a ObjectExpression<'a>)],
    name: &str,
) -> Option<&'a ObjectExpression<'a>> {
    literals
        .iter()
        .find_map(|(literal_name, object)| (literal_name == name).then_some(*object))
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

fn upsert_resolver_mapping(
    mappings: &mut Vec<OmenaResolverBundlerPathAliasMappingV0>,
    mapping: OmenaResolverBundlerPathAliasMappingV0,
) {
    if let Some(index) = mappings
        .iter()
        .position(|entry| entry.pattern == mapping.pattern)
    {
        mappings.remove(index);
    }
    mappings.push(mapping);
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

fn normalize_path_lexical(path: PathBuf) -> PathBuf {
    let mut normalized = PathBuf::new();
    for component in path.components() {
        match component {
            std::path::Component::CurDir => {}
            std::path::Component::ParentDir => {
                normalized.pop();
            }
            std::path::Component::Normal(_)
            | std::path::Component::RootDir
            | std::path::Component::Prefix(_) => {
                normalized.push(component.as_os_str());
            }
        }
    }
    normalized
}

#[cfg(test)]
mod tests {
    use std::{fs, time::SystemTime};

    use super::*;

    #[test]
    fn extracts_vite_object_aliases_from_define_config() -> Result<(), Box<dyn std::error::Error>> {
        let root = temp_dir("omena_bridge_vite_alias_define_config")?;
        let config_path = root.join("vite.config.ts");
        let source = r#"
            import { defineConfig } from "vite";
            export default defineConfig({
              resolve: {
                alias: {
                  "@styles": "./src/styles",
                  "@root": path.resolve(__dirname, "src")
                }
              }
            });
        "#;

        let summary =
            summarize_omena_bridge_bundler_path_aliases_for_config(config_path.as_path(), source);

        assert_eq!(summary.unrecognized, Vec::new());
        assert_eq!(summary.aliases.len(), 2);
        assert_eq!(summary.aliases[0].pattern, "@styles");
        assert_eq!(
            summary.aliases[0].target_path,
            root.join("src/styles").to_string_lossy()
        );
        assert_eq!(summary.aliases[1].pattern, "@root");
        assert_eq!(
            summary.aliases[1].target_path,
            root.join("src").to_string_lossy()
        );
        let _ = fs::remove_dir_all(root);
        Ok(())
    }

    #[test]
    fn extracts_webpack_array_aliases_from_module_exports() -> Result<(), Box<dyn std::error::Error>>
    {
        let root = temp_dir("omena_bridge_webpack_alias_array")?;
        let config_path = root.join("webpack.config.js");
        let source = r#"
            module.exports = {
              resolve: {
                alias: [
                  { find: "@theme", replacement: "./src/theme" }
                ]
              }
            };
        "#;

        let summary =
            summarize_omena_bridge_bundler_path_aliases_for_config(config_path.as_path(), source);

        assert_eq!(summary.unrecognized, Vec::new());
        assert_eq!(summary.aliases.len(), 1);
        assert_eq!(summary.aliases[0].pattern, "@theme");
        assert_eq!(
            summary.aliases[0].target_path,
            root.join("src/theme").to_string_lossy()
        );
        let _ = fs::remove_dir_all(root);
        Ok(())
    }

    #[test]
    fn marks_dynamic_alias_entries_unrecognized() -> Result<(), Box<dyn std::error::Error>> {
        let root = temp_dir("omena_bridge_vite_alias_dynamic")?;
        let config_path = root.join("vite.config.ts");
        let source = r#"
            export default {
              resolve: {
                alias: [{ find: /^@dynamic/, replacement: dynamicTarget }]
              }
            };
        "#;

        let summary =
            summarize_omena_bridge_bundler_path_aliases_for_config(config_path.as_path(), source);

        assert_eq!(summary.aliases, Vec::new());
        assert!(
            summary
                .unrecognized
                .iter()
                .any(|entry| entry.reason == "regex-alias-find")
        );
        let _ = fs::remove_dir_all(root);
        Ok(())
    }

    fn temp_dir(prefix: &str) -> Result<PathBuf, Box<dyn std::error::Error>> {
        let suffix = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)?
            .as_nanos();
        let path = std::env::temp_dir().join(format!("{prefix}_{suffix}"));
        fs::create_dir_all(path.as_path())?;
        Ok(path)
    }
}

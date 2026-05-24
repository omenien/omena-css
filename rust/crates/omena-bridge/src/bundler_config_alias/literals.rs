use oxc_ast::ast::{
    Argument, ArrayExpression, CallExpression, ExportDefaultDeclarationKind, Expression,
    ObjectExpression, Program, Statement,
};

use super::syntax::{binding_pattern_identifier_name, expression_identifier_name};

pub(super) fn top_level_object_literals<'a>(
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

pub(super) fn top_level_array_literals<'a>(
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

pub(super) fn unwrap_export_default_declaration_kind<'a>(
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

pub(super) fn unwrap_config_expression<'a>(
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

pub(super) fn unwrap_static_object_expression<'a>(
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

pub(super) fn find_top_level_literal<'a>(
    literals: &[(String, &'a ObjectExpression<'a>)],
    name: &str,
) -> Option<&'a ObjectExpression<'a>> {
    literals
        .iter()
        .find_map(|(literal_name, object)| (literal_name == name).then_some(*object))
}

pub(super) fn find_top_level_array_literal<'a>(
    literals: &[(String, &'a ArrayExpression<'a>)],
    name: &str,
) -> Option<&'a ArrayExpression<'a>> {
    literals
        .iter()
        .find_map(|(literal_name, array)| (literal_name == name).then_some(*array))
}

fn unwrap_config_call_expression<'a>(
    expression: &'a CallExpression<'a>,
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

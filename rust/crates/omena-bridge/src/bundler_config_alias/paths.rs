use std::path::{Path, PathBuf};

use oxc_ast::ast::{Argument, Expression};

use super::syntax::{expression_identifier_name, skip_parens_and_ts};

pub(super) fn static_replacement_path(
    config_path: &Path,
    expression: &Expression<'_>,
) -> Option<String> {
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

pub(super) fn static_string(expression: &Expression<'_>) -> Option<String> {
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

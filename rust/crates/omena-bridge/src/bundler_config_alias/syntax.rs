use oxc_ast::ast::{AssignmentTarget, BindingPattern, Expression, PropertyKey};

pub(super) fn skip_parens_and_ts<'a>(expression: &'a Expression<'a>) -> &'a Expression<'a> {
    match expression {
        Expression::ParenthesizedExpression(expression) => {
            skip_parens_and_ts(&expression.expression)
        }
        Expression::TSAsExpression(expression) => skip_parens_and_ts(&expression.expression),
        Expression::TSSatisfiesExpression(expression) => skip_parens_and_ts(&expression.expression),
        _ => expression,
    }
}

pub(super) fn property_key_text<'a>(key: &'a PropertyKey<'a>) -> Option<&'a str> {
    match key {
        PropertyKey::StaticIdentifier(identifier) => Some(identifier.name.as_str()),
        PropertyKey::StringLiteral(literal) => Some(literal.value.as_str()),
        _ => None,
    }
}

pub(super) fn binding_pattern_identifier_name<'a>(
    pattern: &'a BindingPattern<'a>,
) -> Option<&'a str> {
    match pattern {
        BindingPattern::BindingIdentifier(identifier) => Some(identifier.name.as_str()),
        _ => None,
    }
}

pub(super) fn expression_identifier_name<'a>(expression: &'a Expression<'a>) -> Option<&'a str> {
    match skip_parens_and_ts(expression) {
        Expression::Identifier(identifier) => Some(identifier.name.as_str()),
        _ => None,
    }
}

pub(super) fn is_module_exports_target(target: &AssignmentTarget<'_>) -> bool {
    let AssignmentTarget::StaticMemberExpression(member) = target else {
        return false;
    };
    expression_identifier_name(&member.object) == Some("module")
        && member.property.name.as_str() == "exports"
}

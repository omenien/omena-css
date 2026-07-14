use std::{collections::BTreeSet, path::Path};

use omena_parser::ParserByteSpanV0;
use oxc_allocator::Allocator;
use oxc_ast::ast::{
    Argument, ArrayExpression, ArrayExpressionElement, AssignmentOperator, AssignmentTarget,
    ExportDefaultDeclarationKind, Expression, ObjectExpression, ObjectPropertyKind, Program,
    PropertyKey, Statement,
};
use oxc_parser::Parser;
use oxc_span::{GetSpan, SourceType};

use crate::{
    SourceClassValuePatternMatcherV0, SourceClassValuePatternV0, SourceClassValueUniverseAxisV0,
    SourceClassValueUniverseEntryV0, SourceClassValueUnresolvedV0,
};

use super::{UTILITY_OWNER_NAME, UTILITY_PROVIDER_ID, UtilityConfigKindV0};

const SCALE_PREFIXES: &[(&str, &[&str])] = &[
    (
        "colors",
        &["bg", "text", "border", "outline", "fill", "stroke"],
    ),
    (
        "spacing",
        &[
            "p", "px", "py", "pt", "pr", "pb", "pl", "m", "mx", "my", "mt", "mr", "mb", "ml",
            "gap", "gap-x", "gap-y", "space-x", "space-y", "w", "h", "min-w", "min-h", "max-w",
            "max-h", "inset", "top", "right", "bottom", "left",
        ],
    ),
    ("fontSize", &["text"]),
    ("borderRadius", &["rounded"]),
    ("borderWidth", &["border"]),
    ("opacity", &["opacity"]),
    ("zIndex", &["z"]),
];

pub(super) fn summarize_config(
    config_path: &Path,
    config_source: &str,
    kind: UtilityConfigKindV0,
) -> SourceClassValueUniverseEntryV0 {
    let config_path_text = config_path.to_string_lossy().to_string();
    let mut entry = empty_universe(kind, config_source.len());
    let allocator = Allocator::default();
    let source_type = SourceType::from_path(config_path).unwrap_or_else(|_| SourceType::ts());
    let parsed = Parser::new(&allocator, config_source, source_type).parse();
    if parsed.panicked || !parsed.diagnostics.is_empty() {
        entry.unresolved.push(unresolved(
            config_path_text,
            "config-parse-failed",
            format!(
                "static parser reported {} diagnostic(s)",
                parsed.diagnostics.len()
            ),
        ));
        return entry;
    }

    let top_level_objects = top_level_object_literals(&parsed.program);
    let mut exported = exported_config_objects(&parsed.program, top_level_objects.as_slice());
    if exported.is_empty() {
        entry.unresolved.push(unresolved(
            config_path_text,
            "config-export-unresolved",
            "expected a static default export, defineConfig object, or module.exports object",
        ));
        return entry;
    }

    let mut enumerated = BTreeSet::new();
    let mut patterns = Vec::new();
    let mut unresolved_items = Vec::new();
    for object in exported.drain(..) {
        collect_safelist(
            config_path,
            config_source,
            object,
            &mut enumerated,
            &mut patterns,
            &mut unresolved_items,
        );
        if kind == UtilityConfigKindV0::UnoCss {
            collect_uno_presets(config_path, config_source, object, &mut unresolved_items);
            collect_shortcuts(
                config_path,
                config_source,
                object,
                &mut enumerated,
                &mut unresolved_items,
            );
            collect_uno_rules(
                config_path,
                config_source,
                object,
                &mut patterns,
                &mut unresolved_items,
            );
        }
        collect_theme(
            config_path,
            config_source,
            object,
            &mut enumerated,
            &mut patterns,
            &mut unresolved_items,
        );
        if let Some(plugins) = object_property_value(object, "plugins")
            && !array_is_empty(plugins)
        {
            unresolved_items.push(unresolved_from_expression(
                config_path,
                config_source,
                "plugins",
                "executable-plugin",
                plugins,
            ));
        }
    }

    if kind == UtilityConfigKindV0::Tailwind {
        unresolved_items.push(unresolved(
            config_path_text,
            "default-theme-not-expanded",
            "the upstream default utility catalog is intentionally not replicated",
        ));
    }
    entry.class_names = enumerated.into_iter().collect();
    entry.axes = vec![SourceClassValueUniverseAxisV0 {
        axis_name: "class".to_string(),
        values: entry.class_names.clone(),
    }];
    patterns.sort_by(|left, right| left.source.cmp(&right.source));
    patterns.dedup_by(|left, right| left.source == right.source);
    unresolved_items.sort_by(|left, right| {
        (
            left.path.as_str(),
            left.reason.as_str(),
            left.detail.as_str(),
        )
            .cmp(&(
                right.path.as_str(),
                right.reason.as_str(),
                right.detail.as_str(),
            ))
    });
    unresolved_items.dedup();
    entry.patterns = patterns;
    entry.unresolved = unresolved_items;
    entry
}

fn empty_universe(kind: UtilityConfigKindV0, source_len: usize) -> SourceClassValueUniverseEntryV0 {
    SourceClassValueUniverseEntryV0 {
        plugin_id: UTILITY_PROVIDER_ID,
        domain: kind.domain(),
        owner_name: UTILITY_OWNER_NAME.to_string(),
        class_names: Vec::new(),
        axes: Vec::new(),
        patterns: Vec::new(),
        unresolved: Vec::new(),
        byte_span: ParserByteSpanV0 {
            start: 0,
            end: source_len,
        },
    }
}

fn collect_safelist(
    config_path: &Path,
    source: &str,
    object: &ObjectExpression<'_>,
    enumerated: &mut BTreeSet<String>,
    patterns: &mut Vec<SourceClassValuePatternV0>,
    unresolved_items: &mut Vec<SourceClassValueUnresolvedV0>,
) {
    let Some(expression) = object_property_value(object, "safelist") else {
        return;
    };
    let Some(array) = static_array(expression) else {
        unresolved_items.push(unresolved_from_expression(
            config_path,
            source,
            "safelist",
            "dynamic-safelist",
            expression,
        ));
        return;
    };
    for (index, element) in array.elements.iter().enumerate() {
        let Some(expression) = array_element_expression(element) else {
            unresolved_items.push(unresolved_from_span(
                config_path,
                source,
                format!("safelist[{index}]"),
                "dynamic-safelist-entry",
                element.span(),
            ));
            continue;
        };
        if let Some(value) = static_string(expression) {
            enumerated.extend(value.split_whitespace().map(str::to_string));
        } else if matches!(
            transparent_expression(expression),
            Expression::RegExpLiteral(_)
        ) {
            let raw = source_for_span(source, expression.span());
            patterns.push(SourceClassValuePatternV0 {
                matcher: SourceClassValuePatternMatcherV0::RegexSource,
                source: raw.clone(),
                completion_hint: raw,
                prefix: None,
                suffix: None,
            });
        } else {
            unresolved_items.push(unresolved_from_expression(
                config_path,
                source,
                format!("safelist[{index}]"),
                "dynamic-safelist-entry",
                expression,
            ));
        }
    }
}

fn collect_uno_presets(
    config_path: &Path,
    source: &str,
    object: &ObjectExpression<'_>,
    unresolved_items: &mut Vec<SourceClassValueUnresolvedV0>,
) {
    let Some(expression) = object_property_value(object, "presets") else {
        return;
    };
    if array_is_empty(expression) {
        return;
    }
    unresolved_items.push(unresolved_from_expression(
        config_path,
        source,
        "presets",
        "presets-not-expanded",
        expression,
    ));
}

fn collect_shortcuts(
    config_path: &Path,
    source: &str,
    object: &ObjectExpression<'_>,
    enumerated: &mut BTreeSet<String>,
    unresolved_items: &mut Vec<SourceClassValueUnresolvedV0>,
) {
    let Some(expression) = object_property_value(object, "shortcuts") else {
        return;
    };
    let Some(shortcuts) = static_object(expression, &[]) else {
        unresolved_items.push(unresolved_from_expression(
            config_path,
            source,
            "shortcuts",
            "dynamic-shortcuts",
            expression,
        ));
        return;
    };
    for property in &shortcuts.properties {
        match property {
            ObjectPropertyKind::ObjectProperty(property) if !property.computed => {
                if let Some(name) = property_key_text(&property.key) {
                    enumerated.insert(name.to_string());
                }
            }
            _ => unresolved_items.push(unresolved_from_span(
                config_path,
                source,
                "shortcuts",
                "dynamic-shortcut-name",
                property.span(),
            )),
        }
    }
}

fn collect_uno_rules(
    config_path: &Path,
    source: &str,
    object: &ObjectExpression<'_>,
    patterns: &mut Vec<SourceClassValuePatternV0>,
    unresolved_items: &mut Vec<SourceClassValueUnresolvedV0>,
) {
    let Some(expression) = object_property_value(object, "rules") else {
        return;
    };
    let Some(rules) = static_array(expression) else {
        unresolved_items.push(unresolved_from_expression(
            config_path,
            source,
            "rules",
            "dynamic-rules",
            expression,
        ));
        return;
    };
    for (index, element) in rules.elements.iter().enumerate() {
        let Some(Expression::ArrayExpression(rule)) =
            array_element_expression(element).map(transparent_expression)
        else {
            unresolved_items.push(unresolved_from_span(
                config_path,
                source,
                format!("rules[{index}]"),
                "dynamic-rule",
                element.span(),
            ));
            continue;
        };
        let Some(matcher) = rule.elements.first().and_then(array_element_expression) else {
            continue;
        };
        if let Some(class_name) = static_string(matcher) {
            patterns.push(SourceClassValuePatternV0 {
                matcher: SourceClassValuePatternMatcherV0::PrefixSuffix,
                source: class_name.clone(),
                completion_hint: class_name.clone(),
                prefix: Some(class_name),
                suffix: Some(String::new()),
            });
        } else if matches!(
            transparent_expression(matcher),
            Expression::RegExpLiteral(_)
        ) {
            let raw = source_for_span(source, matcher.span());
            patterns.push(SourceClassValuePatternV0 {
                matcher: SourceClassValuePatternMatcherV0::RegexSource,
                source: raw.clone(),
                completion_hint: raw,
                prefix: None,
                suffix: None,
            });
        } else {
            unresolved_items.push(unresolved_from_expression(
                config_path,
                source,
                format!("rules[{index}][0]"),
                "dynamic-rule-matcher",
                matcher,
            ));
        }
    }
}

fn collect_theme(
    config_path: &Path,
    source: &str,
    object: &ObjectExpression<'_>,
    enumerated: &mut BTreeSet<String>,
    patterns: &mut Vec<SourceClassValuePatternV0>,
    unresolved_items: &mut Vec<SourceClassValueUnresolvedV0>,
) {
    let Some(theme_expression) = object_property_value(object, "theme") else {
        return;
    };
    let Some(theme) = static_object(theme_expression, &[]) else {
        unresolved_items.push(unresolved_from_expression(
            config_path,
            source,
            "theme",
            "dynamic-theme",
            theme_expression,
        ));
        return;
    };
    collect_theme_container(
        config_path,
        source,
        theme,
        "theme",
        enumerated,
        patterns,
        unresolved_items,
    );
    if let Some(extend_expression) = object_property_value(theme, "extend") {
        if let Some(extend) = static_object(extend_expression, &[]) {
            collect_theme_container(
                config_path,
                source,
                extend,
                "theme.extend",
                enumerated,
                patterns,
                unresolved_items,
            );
        } else {
            unresolved_items.push(unresolved_from_expression(
                config_path,
                source,
                "theme.extend",
                "dynamic-theme-extension",
                extend_expression,
            ));
        }
    }
}

#[allow(clippy::too_many_arguments)]
fn collect_theme_container(
    config_path: &Path,
    source: &str,
    theme: &ObjectExpression<'_>,
    path: &str,
    enumerated: &mut BTreeSet<String>,
    patterns: &mut Vec<SourceClassValuePatternV0>,
    unresolved_items: &mut Vec<SourceClassValueUnresolvedV0>,
) {
    for (scale_name, prefixes) in SCALE_PREFIXES {
        let Some(scale_expression) = object_property_value(theme, scale_name) else {
            continue;
        };
        let mut keys = Vec::new();
        collect_static_scale_keys(
            config_path,
            source,
            format!("{path}.{scale_name}").as_str(),
            scale_expression,
            "",
            &mut keys,
            unresolved_items,
        );
        for prefix in *prefixes {
            for key in &keys {
                let class_name = if key.is_empty() {
                    (*prefix).to_string()
                } else {
                    format!("{prefix}-{key}")
                };
                enumerated.insert(class_name);
            }
            patterns.push(arbitrary_value_pattern(prefix));
        }
    }
}

#[allow(clippy::too_many_arguments)]
fn collect_static_scale_keys(
    config_path: &Path,
    source: &str,
    path: &str,
    expression: &Expression<'_>,
    parent_key: &str,
    keys: &mut Vec<String>,
    unresolved_items: &mut Vec<SourceClassValueUnresolvedV0>,
) {
    let Some(object) = static_object(expression, &[]) else {
        unresolved_items.push(unresolved_from_expression(
            config_path,
            source,
            path,
            "dynamic-theme-scale",
            expression,
        ));
        return;
    };
    for property in &object.properties {
        let ObjectPropertyKind::ObjectProperty(property) = property else {
            unresolved_items.push(unresolved_from_span(
                config_path,
                source,
                path,
                "spread-theme-scale",
                property.span(),
            ));
            continue;
        };
        if property.computed {
            unresolved_items.push(unresolved_from_span(
                config_path,
                source,
                path,
                "computed-theme-key",
                property.span,
            ));
            continue;
        }
        let Some(key) = property_key_text(&property.key)
            .map(str::to_string)
            .or_else(|| {
                matches!(property.key, PropertyKey::NumericLiteral(_))
                    .then(|| source_for_span(source, property.key.span()))
            })
        else {
            continue;
        };
        let normalized_key = if key == "DEFAULT" { "" } else { key.as_str() };
        let compound_key = match (parent_key.is_empty(), normalized_key.is_empty()) {
            (true, _) => normalized_key.to_string(),
            (false, true) => parent_key.to_string(),
            (false, false) => format!("{parent_key}-{normalized_key}"),
        };
        if static_object(&property.value, &[]).is_some() {
            collect_static_scale_keys(
                config_path,
                source,
                path,
                &property.value,
                compound_key.as_str(),
                keys,
                unresolved_items,
            );
        } else if static_leaf(&property.value) {
            keys.push(compound_key);
        } else {
            unresolved_items.push(unresolved_from_expression(
                config_path,
                source,
                format!("{path}.{key}"),
                "dynamic-theme-value",
                &property.value,
            ));
        }
    }
}

fn arbitrary_value_pattern(prefix: &str) -> SourceClassValuePatternV0 {
    SourceClassValuePatternV0 {
        matcher: SourceClassValuePatternMatcherV0::PrefixSuffix,
        source: format!("{prefix}-[<value>]"),
        completion_hint: format!("{prefix}-[...]"),
        prefix: Some(format!("{prefix}-[")),
        suffix: Some("]".to_string()),
    }
}

fn static_leaf(expression: &Expression<'_>) -> bool {
    matches!(
        transparent_expression(expression),
        Expression::StringLiteral(_)
            | Expression::NumericLiteral(_)
            | Expression::BooleanLiteral(_)
            | Expression::NullLiteral(_)
            | Expression::ArrayExpression(_)
            | Expression::TemplateLiteral(_)
    )
}

fn exported_config_objects<'a>(
    program: &'a Program<'a>,
    top_level_objects: &[(String, &'a ObjectExpression<'a>)],
) -> Vec<&'a ObjectExpression<'a>> {
    let mut objects = Vec::new();
    for statement in &program.body {
        match statement {
            Statement::ExportDefaultDeclaration(declaration) => {
                if let Some(object) =
                    object_from_export(&declaration.declaration, top_level_objects)
                {
                    objects.push(object);
                }
            }
            Statement::ExpressionStatement(statement) => {
                let Expression::AssignmentExpression(assignment) = &statement.expression else {
                    continue;
                };
                if assignment.operator == AssignmentOperator::Assign
                    && is_module_exports_target(&assignment.left)
                    && let Some(object) = static_object(&assignment.right, top_level_objects)
                {
                    objects.push(object);
                }
            }
            _ => {}
        }
    }
    objects
}

fn object_from_export<'a>(
    declaration: &'a ExportDefaultDeclarationKind<'a>,
    top_level_objects: &[(String, &'a ObjectExpression<'a>)],
) -> Option<&'a ObjectExpression<'a>> {
    match declaration {
        ExportDefaultDeclarationKind::ObjectExpression(object) => Some(object),
        ExportDefaultDeclarationKind::Identifier(identifier) => {
            find_object(top_level_objects, identifier.name.as_str())
        }
        ExportDefaultDeclarationKind::ParenthesizedExpression(expression) => {
            static_object(&expression.expression, top_level_objects)
        }
        ExportDefaultDeclarationKind::TSAsExpression(expression) => {
            static_object(&expression.expression, top_level_objects)
        }
        ExportDefaultDeclarationKind::TSSatisfiesExpression(expression) => {
            static_object(&expression.expression, top_level_objects)
        }
        ExportDefaultDeclarationKind::CallExpression(call) => call
            .arguments
            .first()
            .and_then(argument_expression)
            .and_then(|expression| static_object(expression, top_level_objects)),
        _ => None,
    }
}

fn top_level_object_literals<'a>(
    program: &'a Program<'a>,
) -> Vec<(String, &'a ObjectExpression<'a>)> {
    let mut objects = Vec::new();
    for statement in &program.body {
        let Statement::VariableDeclaration(declaration) = statement else {
            continue;
        };
        for declarator in &declaration.declarations {
            let oxc_ast::ast::BindingPattern::BindingIdentifier(identifier) = &declarator.id else {
                continue;
            };
            let Some(init) = declarator.init.as_ref() else {
                continue;
            };
            if let Some(object) = static_object(init, objects.as_slice()) {
                objects.push((identifier.name.as_str().to_string(), object));
            }
        }
    }
    objects
}

fn static_object<'a>(
    expression: &'a Expression<'a>,
    top_level_objects: &[(String, &'a ObjectExpression<'a>)],
) -> Option<&'a ObjectExpression<'a>> {
    match expression {
        Expression::ObjectExpression(object) => Some(object),
        Expression::Identifier(identifier) => {
            find_object(top_level_objects, identifier.name.as_str())
        }
        Expression::ParenthesizedExpression(expression) => {
            static_object(&expression.expression, top_level_objects)
        }
        Expression::TSAsExpression(expression) => {
            static_object(&expression.expression, top_level_objects)
        }
        Expression::TSSatisfiesExpression(expression) => {
            static_object(&expression.expression, top_level_objects)
        }
        Expression::CallExpression(call) => call
            .arguments
            .first()
            .and_then(argument_expression)
            .and_then(|expression| static_object(expression, top_level_objects)),
        _ => None,
    }
}

fn static_array<'a>(expression: &'a Expression<'a>) -> Option<&'a ArrayExpression<'a>> {
    match transparent_expression(expression) {
        Expression::ArrayExpression(array) => Some(array),
        _ => None,
    }
}

fn array_is_empty(expression: &Expression<'_>) -> bool {
    static_array(expression).is_some_and(|array| array.elements.is_empty())
}

fn find_object<'a>(
    objects: &[(String, &'a ObjectExpression<'a>)],
    name: &str,
) -> Option<&'a ObjectExpression<'a>> {
    objects
        .iter()
        .rev()
        .find(|(object_name, _)| object_name == name)
        .map(|(_, object)| *object)
}

fn object_property_value<'a>(
    object: &'a ObjectExpression<'a>,
    name: &str,
) -> Option<&'a Expression<'a>> {
    object.properties.iter().find_map(|property| {
        let ObjectPropertyKind::ObjectProperty(property) = property else {
            return None;
        };
        (!property.computed && property_key_text(&property.key) == Some(name))
            .then_some(&property.value)
    })
}

fn property_key_text<'a>(key: &'a PropertyKey<'a>) -> Option<&'a str> {
    match key {
        PropertyKey::StaticIdentifier(identifier) => Some(identifier.name.as_str()),
        PropertyKey::StringLiteral(literal) => Some(literal.value.as_str()),
        _ => None,
    }
}

fn static_string(expression: &Expression<'_>) -> Option<String> {
    match transparent_expression(expression) {
        Expression::StringLiteral(literal) => Some(literal.value.as_str().to_string()),
        Expression::TemplateLiteral(template)
            if template.expressions.is_empty() && template.quasis.len() == 1 =>
        {
            template.quasis[0]
                .value
                .cooked
                .map(|value| value.as_str().to_string())
        }
        _ => None,
    }
}

fn transparent_expression<'a>(expression: &'a Expression<'a>) -> &'a Expression<'a> {
    match expression {
        Expression::ParenthesizedExpression(expression) => {
            transparent_expression(&expression.expression)
        }
        Expression::TSAsExpression(expression) => transparent_expression(&expression.expression),
        Expression::TSSatisfiesExpression(expression) => {
            transparent_expression(&expression.expression)
        }
        Expression::TSNonNullExpression(expression) => {
            transparent_expression(&expression.expression)
        }
        _ => expression,
    }
}

fn argument_expression<'a>(argument: &'a Argument<'a>) -> Option<&'a Expression<'a>> {
    argument.as_expression()
}

fn array_element_expression<'a>(
    element: &'a ArrayExpressionElement<'a>,
) -> Option<&'a Expression<'a>> {
    match element {
        ArrayExpressionElement::SpreadElement(_) | ArrayExpressionElement::Elision(_) => None,
        _ => element.as_expression(),
    }
}

fn is_module_exports_target(target: &AssignmentTarget<'_>) -> bool {
    let AssignmentTarget::StaticMemberExpression(member) = target else {
        return false;
    };
    matches!(transparent_expression(&member.object), Expression::Identifier(identifier) if identifier.name.as_str() == "module")
        && member.property.name.as_str() == "exports"
}

fn unresolved_from_expression(
    config_path: &Path,
    source: &str,
    path: impl Into<String>,
    reason: &str,
    expression: &Expression<'_>,
) -> SourceClassValueUnresolvedV0 {
    unresolved_from_span(config_path, source, path, reason, expression.span())
}

fn unresolved_from_span(
    config_path: &Path,
    source: &str,
    path: impl Into<String>,
    reason: &str,
    span: oxc_span::Span,
) -> SourceClassValueUnresolvedV0 {
    unresolved(
        format!("{}#{}", config_path.display(), path.into()),
        reason,
        source_for_span(source, span),
    )
}

fn unresolved(
    path: impl Into<String>,
    reason: impl Into<String>,
    detail: impl Into<String>,
) -> SourceClassValueUnresolvedV0 {
    SourceClassValueUnresolvedV0 {
        path: path.into(),
        reason: reason.into(),
        detail: detail.into(),
    }
}

fn source_for_span(source: &str, span: oxc_span::Span) -> String {
    source
        .get(span.start as usize..span.end as usize)
        .unwrap_or_default()
        .trim()
        .to_string()
}

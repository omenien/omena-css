use std::collections::{BTreeMap, BTreeSet};

use omena_parser::{
    LexedToken, ParsedVariableFact, ParsedVariableFactKind,
    StyleDialect as OmenaParserStyleDialect, collect_style_facts, lex,
};
use omena_syntax::SyntaxKind;
use omena_transform_passes::{TransformModuleEvaluationV0, reduce_static_numeric_expression};

pub(super) fn derive_static_stylesheet_module_evaluation(
    style_source: &str,
    dialect: OmenaParserStyleDialect,
) -> Option<TransformModuleEvaluationV0> {
    let variable_kind = StaticStylesheetVariableKind::for_dialect(dialect)?;
    let facts = collect_style_facts(style_source, dialect);
    let variable_facts = facts.variables.as_slice();
    if variable_kind == StaticStylesheetVariableKind::Less {
        return derive_static_less_stylesheet_module_evaluation(style_source, variable_facts);
    }
    derive_static_scss_stylesheet_module_evaluation(style_source, variable_facts)
}

fn derive_static_scss_stylesheet_module_evaluation(
    style_source: &str,
    variable_facts: &[ParsedVariableFact],
) -> Option<TransformModuleEvaluationV0> {
    if !variable_facts
        .iter()
        .any(|fact| fact.kind == ParsedVariableFactKind::ScssDeclaration)
    {
        return None;
    }
    let scopes = collect_static_stylesheet_scopes(style_source)?;
    let declarations =
        collect_static_scss_variable_declarations(style_source, variable_facts, &scopes)?;

    let mut edits = Vec::new();
    for declaration in &declarations {
        for (start, end) in &declaration.removal_spans {
            edits.push(StaticStylesheetEvaluationEdit {
                start: *start,
                end: *end,
                replacement: String::new(),
            });
        }
    }
    for fact in variable_facts {
        if fact.kind != ParsedVariableFactKind::ScssReference {
            continue;
        }
        let reference_start = parser_text_size_to_usize(fact.range.start().into());
        if static_stylesheet_position_is_inside_scss_declaration(&declarations, reference_start) {
            continue;
        }
        let mut stack = BTreeSet::new();
        let replacement = resolve_static_scss_variable_value_at_position(
            fact.name.as_str(),
            reference_start,
            &scopes,
            &declarations,
            &mut stack,
        )?;
        edits.push(StaticStylesheetEvaluationEdit {
            start: reference_start,
            end: parser_text_size_to_usize(fact.range.end().into()),
            replacement,
        });
    }

    let evaluated_css = apply_static_stylesheet_evaluation_edits(style_source, edits)?;
    if evaluated_css == style_source {
        return None;
    }

    Some(TransformModuleEvaluationV0 {
        evaluator: StaticStylesheetVariableKind::Scss
            .evaluator_label()
            .to_string(),
        evaluated_css,
    })
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum StaticStylesheetVariableKind {
    Scss,
    Less,
}

impl StaticStylesheetVariableKind {
    fn for_dialect(dialect: OmenaParserStyleDialect) -> Option<Self> {
        match dialect {
            OmenaParserStyleDialect::Scss | OmenaParserStyleDialect::Sass => Some(Self::Scss),
            OmenaParserStyleDialect::Less => Some(Self::Less),
            OmenaParserStyleDialect::Css => None,
        }
    }

    fn evaluator_label(self) -> &'static str {
        match self {
            Self::Scss => "omena-query-static-scss-variable-evaluator",
            Self::Less => "omena-query-static-less-variable-evaluator",
        }
    }

    fn reference_prefix(self) -> char {
        match self {
            Self::Scss => '$',
            Self::Less => '@',
        }
    }
}

#[derive(Debug, Clone)]
struct StaticStylesheetVariableDeclaration {
    value: String,
    span_start: usize,
    span_end: usize,
    removal_spans: Vec<(usize, usize)>,
    is_default: bool,
    is_global: bool,
}

#[derive(Debug, Clone)]
struct StaticStylesheetScopedVariableDeclaration {
    name: String,
    scope_id: usize,
    declaration: StaticStylesheetVariableDeclaration,
    removal_spans: Vec<(usize, usize)>,
}

#[derive(Debug, Clone)]
struct StaticStylesheetEvaluationEdit {
    start: usize,
    end: usize,
    replacement: String,
}

#[derive(Debug, Clone)]
struct StaticStylesheetPropertyDeclaration {
    value: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct StaticStylesheetScope {
    parent_id: Option<usize>,
    body_start: usize,
    end: usize,
}

fn collect_static_scss_variable_declarations(
    source: &str,
    variable_facts: &[ParsedVariableFact],
    scopes: &[StaticStylesheetScope],
) -> Option<Vec<StaticStylesheetScopedVariableDeclaration>> {
    let mut declarations = Vec::new();
    for fact in variable_facts {
        if fact.kind != ParsedVariableFactKind::ScssDeclaration {
            continue;
        }
        let start = parser_text_size_to_usize(fact.range.start().into());
        let end = parser_text_size_to_usize(fact.range.end().into());
        let scope_id = static_stylesheet_scope_for_position(scopes, start)?;
        let declaration = extract_static_stylesheet_variable_declaration(
            source,
            start,
            end,
            StaticStylesheetVariableKind::Scss,
        )?;
        if !static_stylesheet_scss_declaration_value_is_removal_safe(&declaration.value) {
            return None;
        }
        declarations.push(StaticStylesheetScopedVariableDeclaration {
            name: fact.name.clone(),
            scope_id: if declaration.is_global { 0 } else { scope_id },
            removal_spans: declaration.removal_spans.clone(),
            declaration,
        });
    }
    declarations.sort_by_key(|declaration| declaration.declaration.span_start);
    Some(declarations)
}

fn derive_static_less_stylesheet_module_evaluation(
    style_source: &str,
    variable_facts: &[ParsedVariableFact],
) -> Option<TransformModuleEvaluationV0> {
    let scopes = collect_static_stylesheet_scopes(style_source)?;
    let lexed = lex(style_source, OmenaParserStyleDialect::Less);
    let tokens = lexed.tokens();
    let declarations =
        collect_static_less_variable_declarations(style_source, variable_facts, &scopes)?;
    let property_declarations =
        collect_static_less_property_declarations(style_source, tokens, &scopes)?;

    let mut edits = Vec::new();
    for declaration in declarations.values() {
        for (start, end) in &declaration.removal_spans {
            edits.push(StaticStylesheetEvaluationEdit {
                start: *start,
                end: *end,
                replacement: String::new(),
            });
        }
    }
    for fact in variable_facts {
        if fact.kind != ParsedVariableFactKind::LessReference {
            continue;
        }
        let reference_start = parser_text_size_to_usize(fact.range.start().into());
        if static_stylesheet_position_is_inside_scoped_declaration(&declarations, reference_start) {
            continue;
        }
        let reference_scope_id = static_stylesheet_scope_for_position(&scopes, reference_start)?;
        let mut stack = BTreeSet::new();
        let replacement = resolve_static_less_variable_value_in_scope(
            fact.name.as_str(),
            reference_scope_id,
            &scopes,
            &declarations,
            &mut stack,
        )?;
        edits.push(StaticStylesheetEvaluationEdit {
            start: reference_start,
            end: parser_text_size_to_usize(fact.range.end().into()),
            replacement,
        });
    }
    for token in tokens {
        if token.kind != SyntaxKind::LessPropertyVariableToken {
            continue;
        }
        let reference_start = static_stylesheet_token_start(token);
        if static_stylesheet_position_is_inside_scoped_declaration(&declarations, reference_start) {
            continue;
        }
        let reference_scope_id = static_stylesheet_scope_for_position(&scopes, reference_start)?;
        let mut stack = BTreeSet::new();
        let replacement = resolve_static_less_property_value_in_scope(
            token.text.as_str(),
            reference_scope_id,
            &scopes,
            &property_declarations,
            &mut stack,
        )?;
        edits.push(StaticStylesheetEvaluationEdit {
            start: reference_start,
            end: static_stylesheet_token_end(token),
            replacement,
        });
    }

    let evaluated_css = apply_static_stylesheet_evaluation_edits(style_source, edits)?;
    if evaluated_css == style_source {
        return None;
    }

    Some(TransformModuleEvaluationV0 {
        evaluator: StaticStylesheetVariableKind::Less
            .evaluator_label()
            .to_string(),
        evaluated_css,
    })
}

fn collect_static_less_variable_declarations(
    source: &str,
    variable_facts: &[ParsedVariableFact],
    scopes: &[StaticStylesheetScope],
) -> Option<BTreeMap<(usize, String), StaticStylesheetVariableDeclaration>> {
    let mut declarations = BTreeMap::<(usize, String), StaticStylesheetVariableDeclaration>::new();
    for fact in variable_facts {
        if fact.kind != ParsedVariableFactKind::LessDeclaration {
            continue;
        }
        let start = parser_text_size_to_usize(fact.range.start().into());
        let end = parser_text_size_to_usize(fact.range.end().into());
        let scope_id = static_stylesheet_scope_for_position(scopes, start)?;
        let declaration = extract_static_stylesheet_variable_declaration(
            source,
            start,
            end,
            StaticStylesheetVariableKind::Less,
        )?;
        if !static_stylesheet_less_declaration_value_is_removal_safe(&declaration.value) {
            return None;
        }
        let key = (scope_id, fact.name.clone());
        if let Some(previous) = declarations.get_mut(&key) {
            merge_static_stylesheet_duplicate_declaration(
                previous,
                declaration,
                StaticStylesheetVariableKind::Less,
            )?;
            continue;
        }
        declarations.insert(key, declaration);
    }
    Some(declarations)
}

fn collect_static_less_property_declarations(
    source: &str,
    tokens: &[LexedToken],
    scopes: &[StaticStylesheetScope],
) -> Option<BTreeMap<(usize, String), StaticStylesheetPropertyDeclaration>> {
    let mut declarations = BTreeMap::<(usize, String), StaticStylesheetPropertyDeclaration>::new();
    let mut index = 0usize;
    while index < tokens.len() {
        if !matches!(
            tokens[index].kind,
            SyntaxKind::Ident | SyntaxKind::CustomPropertyName
        ) || !static_stylesheet_property_name_is_safe(tokens[index].text.as_str())
            || !static_stylesheet_previous_token_starts_declaration(tokens, index)
        {
            index += 1;
            continue;
        }

        let colon_index = static_stylesheet_skip_trivia_tokens(tokens, index + 1);
        if tokens
            .get(colon_index)
            .is_none_or(|token| token.kind != SyntaxKind::Colon)
        {
            index += 1;
            continue;
        }

        let value_start_index = colon_index + 1;
        let value_end_index =
            static_stylesheet_declaration_value_end_token(tokens, value_start_index)?;
        let value_start = static_stylesheet_token_end(&tokens[colon_index]);
        let value_end = static_stylesheet_token_start(&tokens[value_end_index]);
        let value = source.get(value_start..value_end)?.trim().to_string();
        if value.is_empty() || !static_stylesheet_property_value_is_removal_safe(&value) {
            return None;
        }

        let scope_id = static_stylesheet_scope_for_position(
            scopes,
            static_stylesheet_token_start(&tokens[index]),
        )?;
        declarations.insert(
            (scope_id, format!("${}", tokens[index].text)),
            StaticStylesheetPropertyDeclaration { value },
        );
        index = value_end_index + 1;
    }
    Some(declarations)
}

fn collect_static_stylesheet_scopes(source: &str) -> Option<Vec<StaticStylesheetScope>> {
    let mut scopes = vec![StaticStylesheetScope {
        parent_id: None,
        body_start: 0,
        end: source.len(),
    }];
    let mut stack = vec![0usize];
    let mut index = 0usize;
    let mut quote: Option<char> = None;
    let bytes = source.as_bytes();

    while index < source.len() {
        let ch = source[index..].chars().next()?;
        if let Some(quote_ch) = quote {
            index += ch.len_utf8();
            if ch == '\\' {
                if let Some(escaped) = source[index..].chars().next() {
                    index += escaped.len_utf8();
                }
            } else if ch == quote_ch {
                quote = None;
            }
            continue;
        }

        if matches!(ch, '"' | '\'') {
            quote = Some(ch);
            index += ch.len_utf8();
            continue;
        }
        if bytes.get(index..index + 2) == Some(b"/*") {
            let end = source.get(index + 2..)?.find("*/")?;
            index += end + 4;
            continue;
        }
        if bytes.get(index..index + 2) == Some(b"//") {
            let line_end = source
                .get(index + 2..)?
                .find('\n')
                .map(|offset| index + 2 + offset)
                .unwrap_or(source.len());
            index = line_end;
            continue;
        }

        match ch {
            '{' => {
                let parent_id = *stack.last()?;
                let scope_id = scopes.len();
                scopes.push(StaticStylesheetScope {
                    parent_id: Some(parent_id),
                    body_start: index + ch.len_utf8(),
                    end: source.len(),
                });
                stack.push(scope_id);
            }
            '}' => {
                let scope_id = stack.pop()?;
                if scope_id == 0 {
                    return None;
                }
                scopes.get_mut(scope_id)?.end = index;
            }
            _ => {}
        }
        index += ch.len_utf8();
    }

    (stack.len() == 1).then_some(scopes)
}

fn static_stylesheet_scope_for_position(
    scopes: &[StaticStylesheetScope],
    position: usize,
) -> Option<usize> {
    scopes
        .iter()
        .enumerate()
        .rev()
        .find_map(|(scope_id, scope)| {
            (position >= scope.body_start && position < scope.end).then_some(scope_id)
        })
}

fn resolve_static_scss_variable_value_at_position(
    name: &str,
    position: usize,
    scopes: &[StaticStylesheetScope],
    declarations: &[StaticStylesheetScopedVariableDeclaration],
    stack: &mut BTreeSet<(usize, String, usize)>,
) -> Option<String> {
    let scope_id = static_stylesheet_scope_for_position(scopes, position)?;
    resolve_static_scss_variable_value_in_scope(
        name,
        scope_id,
        position,
        scopes,
        declarations,
        stack,
    )
}

fn resolve_static_scss_variable_value_in_scope(
    name: &str,
    scope_id: usize,
    position: usize,
    scopes: &[StaticStylesheetScope],
    declarations: &[StaticStylesheetScopedVariableDeclaration],
    stack: &mut BTreeSet<(usize, String, usize)>,
) -> Option<String> {
    let stack_key = (scope_id, name.to_string(), position);
    if !stack.insert(stack_key.clone()) {
        return None;
    }
    let declaration =
        find_static_scss_variable_declaration(name, scope_id, position, scopes, declarations)?;
    let resolved = resolve_static_scss_variable_value_text(
        declaration.declaration.value.trim(),
        declaration.declaration.span_start,
        scopes,
        declarations,
        stack,
    );
    stack.remove(&stack_key);
    resolved
}

fn find_static_scss_variable_declaration<'a>(
    name: &str,
    mut scope_id: usize,
    position: usize,
    scopes: &[StaticStylesheetScope],
    declarations: &'a [StaticStylesheetScopedVariableDeclaration],
) -> Option<&'a StaticStylesheetScopedVariableDeclaration> {
    loop {
        if let Some(declaration) = find_static_scss_variable_declaration_in_scope(
            name,
            scope_id,
            position,
            scopes,
            declarations,
        ) {
            return Some(declaration);
        }
        scope_id = scopes.get(scope_id)?.parent_id?;
    }
}

fn find_static_scss_variable_declaration_in_scope<'a>(
    name: &str,
    scope_id: usize,
    position: usize,
    scopes: &[StaticStylesheetScope],
    declarations: &'a [StaticStylesheetScopedVariableDeclaration],
) -> Option<&'a StaticStylesheetScopedVariableDeclaration> {
    let mut active = None;
    for declaration in declarations.iter().filter(|declaration| {
        declaration.name == name
            && declaration.scope_id == scope_id
            && declaration.declaration.span_end <= position
    }) {
        if declaration.declaration.is_default {
            let has_visible_value = active.is_some()
                || scopes
                    .get(scope_id)
                    .and_then(|scope| scope.parent_id)
                    .and_then(|parent_scope_id| {
                        find_static_scss_variable_declaration(
                            name,
                            parent_scope_id,
                            declaration.declaration.span_start,
                            scopes,
                            declarations,
                        )
                    })
                    .is_some();
            if !has_visible_value {
                active = Some(declaration);
            }
            continue;
        }
        active = Some(declaration);
    }
    active
}

fn resolve_static_scss_variable_value_text(
    value: &str,
    position: usize,
    scopes: &[StaticStylesheetScope],
    declarations: &[StaticStylesheetScopedVariableDeclaration],
    stack: &mut BTreeSet<(usize, String, usize)>,
) -> Option<String> {
    let references =
        collect_static_stylesheet_variable_references(value, StaticStylesheetVariableKind::Scss)?;
    if references.is_empty() {
        return static_stylesheet_literal_value_is_safe(value).then(|| value.to_string());
    }
    if !static_stylesheet_composite_value_is_safe(value) {
        return None;
    }

    let mut output = String::with_capacity(value.len());
    let mut cursor = 0usize;
    for reference in references {
        let resolved = resolve_static_scss_variable_value_at_position(
            reference.name.as_str(),
            position,
            scopes,
            declarations,
            stack,
        )?;
        output.push_str(&value[cursor..reference.start]);
        output.push_str(&resolved);
        cursor = reference.end;
    }
    output.push_str(&value[cursor..]);
    Some(output)
}

fn resolve_static_less_variable_value_in_scope(
    name: &str,
    scope_id: usize,
    scopes: &[StaticStylesheetScope],
    declarations: &BTreeMap<(usize, String), StaticStylesheetVariableDeclaration>,
    stack: &mut BTreeSet<(usize, String)>,
) -> Option<String> {
    let stack_key = (scope_id, name.to_string());
    if !stack.insert(stack_key.clone()) {
        return None;
    }
    let declaration = find_static_less_variable_declaration(name, scope_id, scopes, declarations)?;
    let resolved = resolve_static_less_variable_value_text(
        declaration.value.trim(),
        scope_id,
        scopes,
        declarations,
        stack,
    );
    stack.remove(&stack_key);
    resolved.map(reduce_static_less_parenthesized_numeric_value)
}

fn find_static_less_variable_declaration<'a>(
    name: &str,
    mut scope_id: usize,
    scopes: &[StaticStylesheetScope],
    declarations: &'a BTreeMap<(usize, String), StaticStylesheetVariableDeclaration>,
) -> Option<&'a StaticStylesheetVariableDeclaration> {
    loop {
        if let Some(declaration) = declarations.get(&(scope_id, name.to_string())) {
            return Some(declaration);
        }
        scope_id = scopes.get(scope_id)?.parent_id?;
    }
}

fn resolve_static_less_variable_value_text(
    value: &str,
    scope_id: usize,
    scopes: &[StaticStylesheetScope],
    declarations: &BTreeMap<(usize, String), StaticStylesheetVariableDeclaration>,
    stack: &mut BTreeSet<(usize, String)>,
) -> Option<String> {
    let references =
        collect_static_stylesheet_variable_references(value, StaticStylesheetVariableKind::Less)?;
    if references.is_empty() {
        return static_stylesheet_literal_value_is_safe(value).then(|| value.to_string());
    }
    if !static_stylesheet_composite_value_is_safe(value) {
        return None;
    }

    let mut output = String::with_capacity(value.len());
    let mut cursor = 0usize;
    for reference in references {
        let resolved = resolve_static_less_variable_value_in_scope(
            reference.name.as_str(),
            scope_id,
            scopes,
            declarations,
            stack,
        )?;
        output.push_str(&value[cursor..reference.start]);
        output.push_str(&resolved);
        cursor = reference.end;
    }
    output.push_str(&value[cursor..]);
    Some(output)
}

fn resolve_static_less_property_value_in_scope(
    name: &str,
    scope_id: usize,
    scopes: &[StaticStylesheetScope],
    declarations: &BTreeMap<(usize, String), StaticStylesheetPropertyDeclaration>,
    stack: &mut BTreeSet<(usize, String)>,
) -> Option<String> {
    let stack_key = (scope_id, name.to_string());
    if !stack.insert(stack_key.clone()) {
        return None;
    }
    let declaration = find_static_less_property_declaration(name, scope_id, scopes, declarations)?;
    let resolved = resolve_static_less_property_value_text(
        declaration.value.trim(),
        scope_id,
        scopes,
        declarations,
        stack,
    );
    stack.remove(&stack_key);
    resolved
}

fn find_static_less_property_declaration<'a>(
    name: &str,
    mut scope_id: usize,
    scopes: &[StaticStylesheetScope],
    declarations: &'a BTreeMap<(usize, String), StaticStylesheetPropertyDeclaration>,
) -> Option<&'a StaticStylesheetPropertyDeclaration> {
    loop {
        if let Some(declaration) = declarations.get(&(scope_id, name.to_string())) {
            return Some(declaration);
        }
        scope_id = scopes.get(scope_id)?.parent_id?;
    }
}

fn resolve_static_less_property_value_text(
    value: &str,
    scope_id: usize,
    scopes: &[StaticStylesheetScope],
    declarations: &BTreeMap<(usize, String), StaticStylesheetPropertyDeclaration>,
    stack: &mut BTreeSet<(usize, String)>,
) -> Option<String> {
    let references =
        collect_static_stylesheet_variable_references(value, StaticStylesheetVariableKind::Scss)?;
    if references.is_empty() {
        return static_stylesheet_literal_value_is_safe(value).then(|| value.to_string());
    }
    if !static_stylesheet_composite_value_is_safe(value) {
        return None;
    }

    let mut output = String::with_capacity(value.len());
    let mut cursor = 0usize;
    for reference in references {
        let resolved = resolve_static_less_property_value_in_scope(
            reference.name.as_str(),
            scope_id,
            scopes,
            declarations,
            stack,
        )?;
        output.push_str(&value[cursor..reference.start]);
        output.push_str(&resolved);
        cursor = reference.end;
    }
    output.push_str(&value[cursor..]);
    Some(output)
}

fn reduce_static_less_parenthesized_numeric_value(value: String) -> String {
    let trimmed = value.trim();
    let Some(inner) = trimmed
        .strip_prefix('(')
        .and_then(|without_left| without_left.strip_suffix(')'))
    else {
        return value;
    };
    reduce_static_numeric_expression(inner.trim()).unwrap_or(value)
}

fn static_stylesheet_less_declaration_value_is_removal_safe(value: &str) -> bool {
    !value.chars().any(|ch| matches!(ch, '{' | '}' | ';' | '!'))
}

fn static_stylesheet_scss_declaration_value_is_removal_safe(value: &str) -> bool {
    !value.chars().any(|ch| matches!(ch, '{' | '}' | ';' | '!'))
}

fn static_stylesheet_property_name_is_safe(name: &str) -> bool {
    !name.is_empty()
        && name
            .chars()
            .all(|ch| ch.is_ascii_alphanumeric() || matches!(ch, '_' | '-'))
}

fn static_stylesheet_property_value_is_removal_safe(value: &str) -> bool {
    !value.chars().any(|ch| matches!(ch, '{' | '}' | ';' | '!'))
}

fn static_stylesheet_previous_token_starts_declaration(
    tokens: &[LexedToken],
    index: usize,
) -> bool {
    tokens[..index]
        .iter()
        .rev()
        .find(|token| !static_stylesheet_token_is_trivia(token.kind))
        .is_some_and(|token| matches!(token.kind, SyntaxKind::LeftBrace | SyntaxKind::Semicolon))
}

fn static_stylesheet_declaration_value_end_token(
    tokens: &[LexedToken],
    mut index: usize,
) -> Option<usize> {
    let mut paren_depth = 0usize;
    let mut bracket_depth = 0usize;
    while index < tokens.len() {
        match tokens[index].kind {
            SyntaxKind::LeftParen => paren_depth += 1,
            SyntaxKind::RightParen => paren_depth = paren_depth.checked_sub(1)?,
            SyntaxKind::LeftBracket => bracket_depth += 1,
            SyntaxKind::RightBracket => bracket_depth = bracket_depth.checked_sub(1)?,
            SyntaxKind::Semicolon | SyntaxKind::RightBrace
                if paren_depth == 0 && bracket_depth == 0 =>
            {
                return Some(index);
            }
            _ => {}
        }
        index += 1;
    }
    None
}

fn static_stylesheet_skip_trivia_tokens(tokens: &[LexedToken], mut index: usize) -> usize {
    while tokens
        .get(index)
        .is_some_and(|token| static_stylesheet_token_is_trivia(token.kind))
    {
        index += 1;
    }
    index
}

fn static_stylesheet_token_is_trivia(kind: SyntaxKind) -> bool {
    matches!(
        kind,
        SyntaxKind::Whitespace
            | SyntaxKind::LineComment
            | SyntaxKind::BlockComment
            | SyntaxKind::ScssSilentComment
    )
}

fn extract_static_stylesheet_variable_declaration(
    source: &str,
    variable_start: usize,
    variable_end: usize,
    variable_kind: StaticStylesheetVariableKind,
) -> Option<StaticStylesheetVariableDeclaration> {
    let after_name = source.get(variable_end..)?;
    let colon_offset = after_name.find(':')?;
    let value_start = variable_end + colon_offset + 1;
    let terminator_offset = source.get(value_start..)?.find(';')?;
    let span_end = value_start + terminator_offset + 1;
    let (value, is_default, is_global) = parse_static_stylesheet_declaration_value(
        source.get(value_start..span_end - 1)?,
        variable_kind,
    );
    Some(StaticStylesheetVariableDeclaration {
        value,
        span_start: variable_start,
        span_end,
        removal_spans: vec![(variable_start, span_end)],
        is_default,
        is_global,
    })
}

fn parse_static_stylesheet_declaration_value(
    value: &str,
    variable_kind: StaticStylesheetVariableKind,
) -> (String, bool, bool) {
    let mut value = value.trim();
    let mut is_default = false;
    let mut is_global = false;
    if variable_kind == StaticStylesheetVariableKind::Scss {
        loop {
            if let Some(before_flag) = value.strip_suffix("!default")
                && before_flag
                    .chars()
                    .next_back()
                    .is_some_and(char::is_whitespace)
            {
                is_default = true;
                value = before_flag.trim_end();
                continue;
            }
            if let Some(before_flag) = value.strip_suffix("!global")
                && before_flag
                    .chars()
                    .next_back()
                    .is_some_and(char::is_whitespace)
            {
                is_global = true;
                value = before_flag.trim_end();
                continue;
            }
            break;
        }
    }
    (value.to_string(), is_default, is_global)
}

fn merge_static_stylesheet_duplicate_declaration(
    previous: &mut StaticStylesheetVariableDeclaration,
    declaration: StaticStylesheetVariableDeclaration,
    variable_kind: StaticStylesheetVariableKind,
) -> Option<()> {
    match variable_kind {
        StaticStylesheetVariableKind::Less => {
            let mut removal_spans = previous.removal_spans.clone();
            removal_spans.extend(declaration.removal_spans.iter().copied());
            *previous = StaticStylesheetVariableDeclaration {
                removal_spans,
                ..declaration
            };
            Some(())
        }
        StaticStylesheetVariableKind::Scss if declaration.is_default => {
            previous
                .removal_spans
                .extend(declaration.removal_spans.iter().copied());
            Some(())
        }
        StaticStylesheetVariableKind::Scss if previous.is_default => {
            let mut removal_spans = previous.removal_spans.clone();
            removal_spans.extend(declaration.removal_spans.iter().copied());
            *previous = StaticStylesheetVariableDeclaration {
                removal_spans,
                ..declaration
            };
            Some(())
        }
        StaticStylesheetVariableKind::Scss => None,
    }
}

fn static_stylesheet_literal_value_is_safe(value: &str) -> bool {
    let value = value.trim();
    !value.is_empty()
        && !value
            .chars()
            .any(|ch| matches!(ch, '{' | '}' | ';' | '$' | '@' | '!'))
}

fn static_stylesheet_variable_name_is_safe(name: &str) -> bool {
    !name.is_empty()
        && name
            .chars()
            .all(|ch| ch.is_ascii_alphanumeric() || ch == '_' || ch == '-')
}

fn static_stylesheet_composite_value_is_safe(value: &str) -> bool {
    let value = value.trim();
    !value.is_empty() && !value.chars().any(|ch| matches!(ch, '{' | '}' | ';' | '!'))
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct StaticStylesheetVariableReference {
    name: String,
    start: usize,
    end: usize,
}

fn collect_static_stylesheet_variable_references(
    value: &str,
    variable_kind: StaticStylesheetVariableKind,
) -> Option<Vec<StaticStylesheetVariableReference>> {
    let prefix = variable_kind.reference_prefix();
    let other_prefix = match variable_kind {
        StaticStylesheetVariableKind::Scss => '@',
        StaticStylesheetVariableKind::Less => '$',
    };
    let mut references = Vec::new();
    let mut index = 0usize;
    let mut quote: Option<char> = None;

    while index < value.len() {
        let ch = value[index..].chars().next()?;
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

        if matches!(ch, '"' | '\'') {
            quote = Some(ch);
            index += ch.len_utf8();
            continue;
        }
        if ch == other_prefix {
            return None;
        }
        if ch != prefix {
            index += ch.len_utf8();
            continue;
        }

        let name_start = index + ch.len_utf8();
        let name_end = static_stylesheet_variable_name_end(value, name_start);
        if name_end == name_start {
            return None;
        }
        let bare_name = &value[name_start..name_end];
        if !static_stylesheet_variable_name_is_safe(bare_name) {
            return None;
        }
        references.push(StaticStylesheetVariableReference {
            name: value[index..name_end].to_string(),
            start: index,
            end: name_end,
        });
        index = name_end;
    }

    Some(references)
}

fn static_stylesheet_variable_name_end(value: &str, mut index: usize) -> usize {
    while index < value.len() {
        let Some(ch) = value[index..].chars().next() else {
            break;
        };
        if !(ch.is_ascii_alphanumeric() || ch == '_' || ch == '-') {
            break;
        }
        index += ch.len_utf8();
    }
    index
}

fn static_stylesheet_position_is_inside_scoped_declaration(
    declarations: &BTreeMap<(usize, String), StaticStylesheetVariableDeclaration>,
    position: usize,
) -> bool {
    declarations.values().any(|declaration| {
        declaration
            .removal_spans
            .iter()
            .any(|(start, end)| position >= *start && position < *end)
    })
}

fn static_stylesheet_position_is_inside_scss_declaration(
    declarations: &[StaticStylesheetScopedVariableDeclaration],
    position: usize,
) -> bool {
    declarations.iter().any(|declaration| {
        declaration
            .removal_spans
            .iter()
            .any(|(start, end)| position >= *start && position < *end)
    })
}

fn apply_static_stylesheet_evaluation_edits(
    source: &str,
    mut edits: Vec<StaticStylesheetEvaluationEdit>,
) -> Option<String> {
    edits.sort_by_key(|edit| edit.start);
    let mut previous_end = 0usize;
    for edit in &edits {
        if edit.start < previous_end || edit.start > edit.end || edit.end > source.len() {
            return None;
        }
        previous_end = edit.end;
    }

    let mut output = source.to_string();
    for edit in edits.into_iter().rev() {
        output.replace_range(edit.start..edit.end, edit.replacement.as_str());
    }
    Some(output)
}

fn parser_text_size_to_usize(value: u32) -> usize {
    value as usize
}

fn static_stylesheet_token_start(token: &LexedToken) -> usize {
    parser_text_size_to_usize(token.range.start().into())
}

fn static_stylesheet_token_end(token: &LexedToken) -> usize {
    parser_text_size_to_usize(token.range.end().into())
}

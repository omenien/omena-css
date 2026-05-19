use std::collections::{BTreeMap, BTreeSet};

use omena_parser::{
    ParsedVariableFact, ParsedVariableFactKind, StyleDialect as OmenaParserStyleDialect,
    collect_style_facts,
};
use omena_transform_passes::TransformModuleEvaluationV0;

pub(super) fn derive_static_stylesheet_module_evaluation(
    style_source: &str,
    dialect: OmenaParserStyleDialect,
) -> Option<TransformModuleEvaluationV0> {
    let variable_kind = StaticStylesheetVariableKind::for_dialect(dialect)?;
    let facts = collect_style_facts(style_source, dialect);
    let variable_facts = facts.variables.as_slice();
    if !variable_facts
        .iter()
        .any(|fact| variable_kind.matches_declaration(fact.kind))
    {
        return None;
    }

    let declarations = collect_static_stylesheet_variable_declarations(
        style_source,
        variable_facts,
        variable_kind,
    )?;
    for fact in variable_facts {
        if !variable_kind.matches_reference(fact.kind) {
            continue;
        }
        let reference_start = parser_text_size_to_usize(fact.range.start().into());
        if static_stylesheet_position_is_inside_declaration(&declarations, reference_start) {
            continue;
        }
        let declaration = declarations.get(fact.name.as_str())?;
        if declaration.span_end > reference_start {
            return None;
        }
    }

    if declarations.is_empty() {
        return None;
    }

    let mut edits = Vec::new();
    for declaration in declarations.values() {
        edits.push(StaticStylesheetEvaluationEdit {
            start: declaration.span_start,
            end: declaration.span_end,
            replacement: String::new(),
        });
    }
    for fact in variable_facts {
        if !variable_kind.matches_reference(fact.kind) {
            continue;
        }
        let reference_start = parser_text_size_to_usize(fact.range.start().into());
        if static_stylesheet_position_is_inside_declaration(&declarations, reference_start) {
            continue;
        }
        let declaration = declarations.get(fact.name.as_str())?;
        edits.push(StaticStylesheetEvaluationEdit {
            start: reference_start,
            end: parser_text_size_to_usize(fact.range.end().into()),
            replacement: declaration.value.clone(),
        });
    }

    let evaluated_css = apply_static_stylesheet_evaluation_edits(style_source, edits)?;
    if evaluated_css == style_source {
        return None;
    }

    Some(TransformModuleEvaluationV0 {
        evaluator: variable_kind.evaluator_label().to_string(),
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

    fn matches_declaration(self, kind: ParsedVariableFactKind) -> bool {
        matches!(
            (self, kind),
            (Self::Scss, ParsedVariableFactKind::ScssDeclaration)
                | (Self::Less, ParsedVariableFactKind::LessDeclaration)
        )
    }

    fn matches_reference(self, kind: ParsedVariableFactKind) -> bool {
        matches!(
            (self, kind),
            (Self::Scss, ParsedVariableFactKind::ScssReference)
                | (Self::Less, ParsedVariableFactKind::LessReference)
        )
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
}

#[derive(Debug, Clone)]
struct StaticStylesheetEvaluationEdit {
    start: usize,
    end: usize,
    replacement: String,
}

fn collect_static_stylesheet_variable_declarations(
    source: &str,
    variable_facts: &[ParsedVariableFact],
    variable_kind: StaticStylesheetVariableKind,
) -> Option<BTreeMap<String, StaticStylesheetVariableDeclaration>> {
    let mut declarations = BTreeMap::<String, StaticStylesheetVariableDeclaration>::new();
    for fact in variable_facts {
        if !variable_kind.matches_declaration(fact.kind) {
            continue;
        }
        let start = parser_text_size_to_usize(fact.range.start().into());
        let end = parser_text_size_to_usize(fact.range.end().into());
        if !source_position_is_top_level(source, start) {
            return None;
        }
        let declaration = extract_static_stylesheet_variable_declaration(source, start, end)?;
        if !static_stylesheet_variable_value_is_allowed(&declaration.value, variable_kind) {
            return None;
        }
        if declarations.contains_key(fact.name.as_str()) {
            return None;
        }
        declarations.insert(fact.name.clone(), declaration);
    }
    resolve_static_stylesheet_variable_declarations(declarations, variable_kind)
}

fn extract_static_stylesheet_variable_declaration(
    source: &str,
    variable_start: usize,
    variable_end: usize,
) -> Option<StaticStylesheetVariableDeclaration> {
    let after_name = source.get(variable_end..)?;
    let colon_offset = after_name.find(':')?;
    let value_start = variable_end + colon_offset + 1;
    let terminator_offset = source.get(value_start..)?.find(';')?;
    let span_end = value_start + terminator_offset + 1;
    let value = source.get(value_start..span_end - 1)?.trim().to_string();
    Some(StaticStylesheetVariableDeclaration {
        value,
        span_start: variable_start,
        span_end,
    })
}

fn static_stylesheet_variable_value_is_allowed(
    value: &str,
    variable_kind: StaticStylesheetVariableKind,
) -> bool {
    static_stylesheet_literal_value_is_safe(value)
        || parse_static_stylesheet_variable_reference_value(value, variable_kind).is_some()
}

fn static_stylesheet_literal_value_is_safe(value: &str) -> bool {
    let value = value.trim();
    !value.is_empty()
        && !value
            .chars()
            .any(|ch| matches!(ch, '{' | '}' | ';' | '$' | '@' | '!'))
}

fn resolve_static_stylesheet_variable_declarations(
    declarations: BTreeMap<String, StaticStylesheetVariableDeclaration>,
    variable_kind: StaticStylesheetVariableKind,
) -> Option<BTreeMap<String, StaticStylesheetVariableDeclaration>> {
    let mut resolved = BTreeMap::new();
    for name in declarations.keys() {
        let mut stack = BTreeSet::new();
        let value = resolve_static_stylesheet_variable_value(
            name,
            &declarations,
            variable_kind,
            &mut stack,
        )?;
        let mut declaration = declarations.get(name.as_str())?.clone();
        declaration.value = value;
        resolved.insert(name.clone(), declaration);
    }
    Some(resolved)
}

fn resolve_static_stylesheet_variable_value(
    name: &str,
    declarations: &BTreeMap<String, StaticStylesheetVariableDeclaration>,
    variable_kind: StaticStylesheetVariableKind,
    stack: &mut BTreeSet<String>,
) -> Option<String> {
    if !stack.insert(name.to_string()) {
        return None;
    }
    let declaration = declarations.get(name)?;
    let value = declaration.value.trim();
    let resolved = if static_stylesheet_literal_value_is_safe(value) {
        Some(value.to_string())
    } else {
        let target_name = parse_static_stylesheet_variable_reference_value(value, variable_kind)?;
        let target = declarations.get(target_name)?;
        if target.span_end > declaration.span_start {
            None
        } else {
            resolve_static_stylesheet_variable_value(
                target_name,
                declarations,
                variable_kind,
                stack,
            )
        }
    };
    stack.remove(name);
    resolved
}

fn parse_static_stylesheet_variable_reference_value(
    value: &str,
    variable_kind: StaticStylesheetVariableKind,
) -> Option<&str> {
    let value = value.trim();
    let name = value.strip_prefix(variable_kind.reference_prefix())?;
    static_stylesheet_variable_name_is_safe(name).then_some(value)
}

fn static_stylesheet_variable_name_is_safe(name: &str) -> bool {
    !name.is_empty()
        && name
            .chars()
            .all(|ch| ch.is_ascii_alphanumeric() || ch == '_' || ch == '-')
}

fn static_stylesheet_position_is_inside_declaration(
    declarations: &BTreeMap<String, StaticStylesheetVariableDeclaration>,
    position: usize,
) -> bool {
    declarations
        .values()
        .any(|declaration| position >= declaration.span_start && position < declaration.span_end)
}

fn source_position_is_top_level(source: &str, position: usize) -> bool {
    let mut depth = 0usize;
    for byte in source.as_bytes().iter().take(position) {
        match byte {
            b'{' => depth += 1,
            b'}' => depth = depth.saturating_sub(1),
            _ => {}
        }
    }
    depth == 0
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

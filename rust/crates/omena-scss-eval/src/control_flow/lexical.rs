use std::collections::BTreeMap;

use omena_abstract_value::AbstractCssValueV0;
use omena_parser::{
    ParsedSassSymbolFactKind, ParsedVariableFact, ParsedVariableFactKind, StyleDialect,
    collect_style_facts, lex,
};
use omena_syntax::SyntaxKind;

use super::analysis_model::ScssGlobalVariableDeclaration;
use super::call_resolution::canonical_scss_callable_name;
use super::header_values::static_scss_header_abstract_value;
use super::tokens::{declaration_end_token_index, next_non_trivia_token_index};
use super::variables::canonical_scss_variable_name;

pub(super) fn collect_lexical_scss_bindings(
    source: &str,
    dialect: StyleDialect,
) -> LexicalScssBindings {
    let lexed = lex(source, dialect);
    let tokens = lexed.tokens();
    let Some(scopes) = collect_lexical_scss_scopes(source) else {
        return LexicalScssBindings::new(Vec::new());
    };
    let facts = collect_style_facts(source, dialect);
    let mut bindings = LexicalScssBindings::new(scopes);
    for symbol in &facts.sass_symbols {
        match symbol.kind {
            ParsedSassSymbolFactKind::FunctionDeclaration => bindings.push_callable(
                LexicalScssCallableKind::Function,
                symbol.name.as_str(),
                symbol.range.start().into(),
            ),
            ParsedSassSymbolFactKind::MixinDeclaration => bindings.push_callable(
                LexicalScssCallableKind::Mixin,
                symbol.name.as_str(),
                symbol.range.start().into(),
            ),
            ParsedSassSymbolFactKind::FunctionCall
            | ParsedSassSymbolFactKind::MixinInclude
            | ParsedSassSymbolFactKind::VariableDeclaration
            | ParsedSassSymbolFactKind::VariableReference => {}
        }
    }
    for (index, token) in tokens.iter().enumerate() {
        if token.kind != SyntaxKind::ScssVariable {
            continue;
        }
        let Some(colon_index) = next_non_trivia_token_index(tokens, index + 1) else {
            continue;
        };
        if tokens[colon_index].kind != SyntaxKind::Colon {
            continue;
        }
        let value_start = tokens[colon_index].range.end().into();
        let Some(value_end_index) = declaration_end_token_index(tokens, colon_index + 1) else {
            continue;
        };
        let value_end = tokens[value_end_index].range.start().into();
        if let Some(value) = source.get(value_start..value_end).map(str::trim)
            && !value.is_empty()
        {
            let declaration_start = token.range.start().into();
            let Some(scope_id) =
                lexical_scss_scope_for_position(&bindings.scopes, declaration_start)
            else {
                continue;
            };
            bindings.push(
                token.text.as_str(),
                declaration_start,
                scope_id,
                static_scss_header_abstract_value(value),
            );
        }
    }
    bindings
}

pub(super) fn collect_scss_global_variable_declarations(
    source: &str,
    variable_facts: &[ParsedVariableFact],
) -> Vec<ScssGlobalVariableDeclaration> {
    let Some(scopes) = collect_lexical_scss_scopes(source) else {
        return Vec::new();
    };
    variable_facts
        .iter()
        .filter(|fact| fact.kind == ParsedVariableFactKind::ScssDeclaration)
        .filter_map(|fact| {
            let declaration_start = fact.range.start().into();
            let scope_id = lexical_scss_scope_for_position(&scopes, declaration_start)?;
            (scope_id == 0).then(|| ScssGlobalVariableDeclaration {
                name: canonical_scss_variable_name(fact.name.as_str()),
                declaration_start,
            })
        })
        .collect()
}

pub(super) fn scss_global_variable_metadata_exists(
    name: &str,
    position: usize,
    declarations: &[ScssGlobalVariableDeclaration],
) -> Option<bool> {
    let canonical_name = canonical_scss_variable_name(name);
    if declarations.iter().any(|declaration| {
        declaration.name == canonical_name && declaration.declaration_start <= position
    }) {
        return Some(true);
    }
    if declarations.iter().any(|declaration| {
        declaration.name == canonical_name && declaration.declaration_start > position
    }) {
        return None;
    }
    Some(false)
}

pub(super) fn static_scss_metadata_exists_call_may_need_resolution(value: &str) -> bool {
    const NAMES: [&str; 8] = [
        "meta.function-exists(",
        "function-exists(",
        "meta.mixin-exists(",
        "mixin-exists(",
        "meta.variable-exists(",
        "variable-exists(",
        "meta.global-variable-exists(",
        "global-variable-exists(",
    ];
    let lower = value.to_ascii_lowercase();
    NAMES.iter().any(|name| lower.contains(name))
}

fn collect_lexical_scss_scopes(source: &str) -> Option<Vec<LexicalScssScope>> {
    let mut scopes = vec![LexicalScssScope {
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
                scopes.push(LexicalScssScope {
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

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub(super) struct LexicalScssBindings {
    bindings: Vec<LexicalScssBinding>,
    callables: Vec<LexicalScssCallableDeclaration>,
    scopes: Vec<LexicalScssScope>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct LexicalScssBinding {
    name: String,
    declaration_start: usize,
    scope_id: usize,
    value: AbstractCssValueV0,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum LexicalScssCallableKind {
    Function,
    Mixin,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct LexicalScssCallableDeclaration {
    kind: LexicalScssCallableKind,
    name: String,
    declaration_start: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct LexicalScssScope {
    parent_id: Option<usize>,
    body_start: usize,
    end: usize,
}

impl LexicalScssBindings {
    fn new(scopes: Vec<LexicalScssScope>) -> Self {
        Self {
            bindings: Vec::new(),
            callables: Vec::new(),
            scopes,
        }
    }

    fn push(
        &mut self,
        name: &str,
        declaration_start: usize,
        scope_id: usize,
        value: AbstractCssValueV0,
    ) {
        self.bindings.push(LexicalScssBinding {
            name: canonical_scss_variable_name(name),
            declaration_start,
            scope_id,
            value,
        });
    }

    fn push_callable(
        &mut self,
        kind: LexicalScssCallableKind,
        name: &str,
        declaration_start: usize,
    ) {
        self.callables.push(LexicalScssCallableDeclaration {
            kind,
            name: canonical_scss_callable_name(name),
            declaration_start,
        });
    }

    pub(super) fn visible_function_metadata_exists(
        &self,
        name: &str,
        position: usize,
    ) -> Option<bool> {
        self.visible_callable_metadata_exists(LexicalScssCallableKind::Function, name, position)
    }

    pub(super) fn visible_mixin_metadata_exists(
        &self,
        name: &str,
        position: usize,
    ) -> Option<bool> {
        self.visible_callable_metadata_exists(LexicalScssCallableKind::Mixin, name, position)
    }

    fn visible_callable_metadata_exists(
        &self,
        kind: LexicalScssCallableKind,
        name: &str,
        position: usize,
    ) -> Option<bool> {
        let canonical_name = canonical_scss_callable_name(name);
        self.callables
            .iter()
            .any(|callable| {
                callable.kind == kind
                    && callable.name == canonical_name
                    && callable.declaration_start <= position
            })
            .then_some(true)
    }

    pub(super) fn visible_at(&self, position: usize) -> BTreeMap<String, AbstractCssValueV0> {
        let Some(scope_id) = lexical_scss_scope_for_position(&self.scopes, position) else {
            return BTreeMap::new();
        };
        let mut visible = BTreeMap::new();
        for binding in self.bindings.iter() {
            if binding.declaration_start > position {
                continue;
            }
            if lexical_scss_scope_is_ancestor_or_self(&self.scopes, binding.scope_id, scope_id) {
                visible.insert(binding.name.clone(), binding.value.clone());
            } else {
                visible.insert(binding.name.clone(), AbstractCssValueV0::Top);
            }
        }
        visible
    }

    pub(super) fn visible_variable_metadata_exists(
        &self,
        name: &str,
        position: usize,
    ) -> Option<bool> {
        let canonical_name = canonical_scss_variable_name(name);
        let scope_id = lexical_scss_scope_for_position(&self.scopes, position)?;
        if self.bindings.iter().any(|binding| {
            binding.name == canonical_name
                && binding.declaration_start <= position
                && lexical_scss_scope_is_ancestor_or_self(&self.scopes, binding.scope_id, scope_id)
        }) {
            return Some(true);
        }
        if self.bindings.iter().any(|binding| {
            binding.name == canonical_name
                && binding.declaration_start > position
                && lexical_scss_scope_is_ancestor_or_self(&self.scopes, binding.scope_id, scope_id)
        }) {
            return None;
        }
        Some(false)
    }

    pub(super) fn global_variable_metadata_exists(
        &self,
        name: &str,
        position: usize,
    ) -> Option<bool> {
        let canonical_name = canonical_scss_variable_name(name);
        if self.bindings.iter().any(|binding| {
            binding.name == canonical_name
                && binding.scope_id == 0
                && binding.declaration_start <= position
        }) {
            return Some(true);
        }
        if self.bindings.iter().any(|binding| {
            binding.name == canonical_name
                && binding.scope_id == 0
                && binding.declaration_start > position
        }) {
            return None;
        }
        Some(false)
    }
}

fn lexical_scss_scope_for_position(scopes: &[LexicalScssScope], position: usize) -> Option<usize> {
    scopes
        .iter()
        .enumerate()
        .rev()
        .find_map(|(scope_id, scope)| {
            (position >= scope.body_start && position < scope.end).then_some(scope_id)
        })
}

fn lexical_scss_scope_is_ancestor_or_self(
    scopes: &[LexicalScssScope],
    ancestor_id: usize,
    mut scope_id: usize,
) -> bool {
    loop {
        if scope_id == ancestor_id {
            return true;
        }
        let Some(parent_id) = scopes.get(scope_id).and_then(|scope| scope.parent_id) else {
            return false;
        };
        scope_id = parent_id;
    }
}

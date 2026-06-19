use std::collections::BTreeSet;

use omena_parser::LexedToken;

use super::{
    StaticScssFunctionDeclaration, StaticScssFunctionResolutionContext, StaticScssMixinDeclaration,
    StaticScssMixinEvaluationEdits, StaticStylesheetEvaluationEdit, StaticStylesheetScope,
    StaticStylesheetScopedVariableDeclaration, canonical_static_scss_function_name,
    collect_static_scss_mixin_include_calls, extend_static_scss_used_function_dependencies,
    render_static_scss_mixin_include_body, static_scss_mixin_include_is_inside_declaration_body,
    static_scss_mixin_include_is_inside_function_declaration_body,
};

pub(super) fn collect_static_scss_mixin_evaluation_edits(
    source: &str,
    tokens: &[LexedToken],
    function_declarations: &[StaticScssFunctionDeclaration],
    mixin_declarations: &[StaticScssMixinDeclaration],
    scopes: &[StaticStylesheetScope],
    variable_declarations: &[StaticStylesheetScopedVariableDeclaration],
) -> Option<StaticScssMixinEvaluationEdits> {
    let calls = collect_static_scss_mixin_include_calls(source, tokens, mixin_declarations)?;
    if calls.is_empty() {
        return Some(StaticScssMixinEvaluationEdits {
            edits: Vec::new(),
            preserved_raw_include_count: 0,
        });
    }

    let context = StaticScssFunctionResolutionContext {
        declarations: function_declarations,
        mixin_declarations,
        scopes,
        variable_declarations,
        active_functions: &BTreeSet::new(),
    };
    let mut edits = Vec::new();
    let mut used_declaration_names = BTreeSet::new();
    let mut preserved_declaration_names = BTreeSet::new();
    let mut used_function_declaration_names = BTreeSet::new();
    let mut preserved_raw_include_count = 0usize;
    for call in calls.iter().filter(|call| {
        !static_scss_mixin_include_is_inside_declaration_body(call, mixin_declarations)
            && !static_scss_mixin_include_is_inside_function_declaration_body(
                call,
                function_declarations,
            )
    }) {
        let Some(declaration) = mixin_declarations.iter().find(|declaration| {
            canonical_static_scss_function_name(declaration.name.as_str())
                == canonical_static_scss_function_name(call.name.as_str())
        }) else {
            continue;
        };
        let Some(rendered) = render_static_scss_mixin_include_body(
            source,
            tokens,
            declaration,
            call,
            call.start,
            context,
        ) else {
            preserved_raw_include_count += 1;
            preserved_declaration_names.insert(canonical_static_scss_function_name(
                declaration.name.as_str(),
            ));
            continue;
        };
        used_declaration_names.extend(rendered.used_mixin_declaration_names);
        used_function_declaration_names.extend(rendered.used_function_declaration_names);
        edits.push(StaticStylesheetEvaluationEdit {
            start: call.start,
            end: call.end,
            replacement: rendered.body,
        });
    }

    for declaration in mixin_declarations.iter().filter(|declaration| {
        let canonical_name = canonical_static_scss_function_name(declaration.name.as_str());
        used_declaration_names.contains(&canonical_name)
            && !preserved_declaration_names.contains(&canonical_name)
    }) {
        edits.push(StaticStylesheetEvaluationEdit {
            start: declaration.span_start,
            end: declaration.span_end,
            replacement: String::new(),
        });
    }
    extend_static_scss_used_function_dependencies(
        &mut used_function_declaration_names,
        function_declarations,
    );
    for declaration in function_declarations.iter().filter(|declaration| {
        used_function_declaration_names.contains(&canonical_static_scss_function_name(
            declaration.name.as_str(),
        ))
    }) {
        edits.push(StaticStylesheetEvaluationEdit {
            start: declaration.span_start,
            end: declaration.span_end,
            replacement: String::new(),
        });
    }

    Some(StaticScssMixinEvaluationEdits {
        edits,
        preserved_raw_include_count,
    })
}

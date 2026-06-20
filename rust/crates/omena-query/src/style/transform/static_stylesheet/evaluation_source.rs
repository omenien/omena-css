use super::*;
use omena_query_transform_runner::{
    TransformImportInlineV0, TransformLessInlineLiteralPlaceholderV0, inline_css_imports,
    inline_css_imports_for_static_module_evaluation,
    materialize_transform_module_evaluation_native_edits,
};
use std::borrow::Cow;

pub(super) struct StaticModuleEvaluationSource<'a> {
    pub(super) source: Cow<'a, str>,
    pub(super) less_inline_literal_placeholders: Vec<TransformLessInlineLiteralPlaceholderV0>,
}

pub(super) fn derive_import_aware_static_stylesheet_module_evaluation_source<'a>(
    style_source: &'a str,
    dialect: OmenaParserStyleDialect,
    import_inlines: &[TransformImportInlineV0],
) -> StaticModuleEvaluationSource<'a> {
    if import_inlines.is_empty() {
        return StaticModuleEvaluationSource {
            source: Cow::Borrowed(style_source),
            less_inline_literal_placeholders: Vec::new(),
        };
    }
    let (inlined_source, mutation_count, less_inline_literal_placeholders) = if dialect
        == OmenaParserStyleDialect::Less
    {
        let (inlined_source, mutation_count, placeholders) =
            inline_css_imports_for_static_module_evaluation(style_source, dialect, import_inlines);
        (inlined_source, mutation_count, placeholders)
    } else {
        let (inlined_source, mutation_count) =
            inline_css_imports(style_source, dialect, import_inlines);
        (inlined_source, mutation_count, Vec::new())
    };
    if mutation_count == 0 {
        StaticModuleEvaluationSource {
            source: Cow::Borrowed(style_source),
            less_inline_literal_placeholders,
        }
    } else {
        StaticModuleEvaluationSource {
            source: Cow::Owned(inlined_source),
            less_inline_literal_placeholders,
        }
    }
}

pub(super) fn static_stylesheet_module_system_evaluator_label(
    dialect: OmenaParserStyleDialect,
) -> &'static str {
    match dialect {
        OmenaParserStyleDialect::Scss | OmenaParserStyleDialect::Sass => {
            "omena-query-static-scss-module-system-evaluator"
        }
        OmenaParserStyleDialect::Less => "omena-query-static-less-module-system-evaluator",
        OmenaParserStyleDialect::Css => "omena-query-static-css-module-system-evaluator",
    }
}

pub(super) fn static_stylesheet_module_output_css_from_evaluation(
    input_css: &str,
    evaluation: TransformModuleEvaluationV0,
) -> Option<String> {
    if !evaluation.may_consume_native_product_output() {
        return None;
    }
    if let Some(native_edit_output) = evaluation.native_edit_output {
        return Some(native_edit_output);
    }
    if let Some(native_css) =
        materialize_transform_module_evaluation_native_edits(input_css, &evaluation.native_edits)
        && native_css == evaluation.evaluated_css
    {
        return Some(native_css);
    }
    None
}

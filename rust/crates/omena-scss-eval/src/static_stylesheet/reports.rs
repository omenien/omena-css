use omena_abstract_value::{AbstractCssValueV0, abstract_css_value_from_text};
use omena_parser::StyleDialect;

use crate::{
    abstract_css_value_kind, abstract_css_value_reflected_in_legacy_css,
    summarize_omena_scss_eval_oracle,
};

use super::{
    STATIC_STYLESHEET_VALUE_RESOLUTION_FUEL_LIMIT, dialect_label,
    edits::{
        apply_normalized_static_stylesheet_evaluation_edits,
        normalize_static_stylesheet_evaluation_edits,
    },
    model::{
        OmenaScssEvalResolvedReplacementV0, OmenaScssEvalStaticStylesheetEvaluationV0,
        OmenaScssEvalStaticStylesheetNativeEditV0, OmenaScssEvalStaticValueResolutionReportV0,
        OmenaScssEvalStaticValueResolutionV0, StaticStylesheetEvaluationEdit,
        StaticStylesheetVariableKind,
    },
    summarize_static_stylesheet_value_resolution,
    value_resolution_model::render_static_abstract_value,
};

pub(super) fn build_static_stylesheet_evaluation_report(
    style_source: &str,
    dialect: StyleDialect,
    variable_kind: StaticStylesheetVariableKind,
    evaluated_css: String,
    native_edit_source: Vec<StaticStylesheetEvaluationEdit>,
    resolved_replacements: Vec<OmenaScssEvalResolvedReplacementV0>,
) -> Option<OmenaScssEvalStaticStylesheetEvaluationV0> {
    let value_resolution = summarize_static_stylesheet_value_resolution(style_source, dialect)?;
    build_static_stylesheet_evaluation_report_with_value_resolution(
        style_source,
        dialect,
        variable_kind,
        evaluated_css,
        native_edit_source,
        resolved_replacements,
        value_resolution,
    )
}

pub(super) fn build_static_stylesheet_preserved_evaluation_report_if_explained(
    style_source: &str,
    dialect: StyleDialect,
    variable_kind: StaticStylesheetVariableKind,
) -> Option<OmenaScssEvalStaticStylesheetEvaluationV0> {
    let value_resolution = summarize_static_stylesheet_value_resolution(style_source, dialect)?;
    if value_resolution.raw_count == 0 && value_resolution.top_count == 0 {
        return None;
    }
    build_static_stylesheet_evaluation_report_with_value_resolution(
        style_source,
        dialect,
        variable_kind,
        style_source.to_string(),
        Vec::new(),
        Vec::new(),
        value_resolution,
    )
}

fn build_static_stylesheet_evaluation_report_with_value_resolution(
    style_source: &str,
    dialect: StyleDialect,
    variable_kind: StaticStylesheetVariableKind,
    evaluated_css: String,
    native_edit_source: Vec<StaticStylesheetEvaluationEdit>,
    resolved_replacements: Vec<OmenaScssEvalResolvedReplacementV0>,
    value_resolution: OmenaScssEvalStaticValueResolutionReportV0,
) -> Option<OmenaScssEvalStaticStylesheetEvaluationV0> {
    let oracle = summarize_omena_scss_eval_oracle(style_source, dialect, evaluated_css.as_str());
    if !oracle.all_legacy_declaration_values_preserved {
        return None;
    }
    let native_replacement_legacy_reflection_count =
        count_native_replacements_reflected_in_legacy_css(
            resolved_replacements.as_slice(),
            evaluated_css.as_str(),
            dialect,
        );
    let native_replacement_legacy_unreflected_count = resolved_replacements
        .len()
        .saturating_sub(native_replacement_legacy_reflection_count);
    let normalized_native_edit_source =
        normalize_static_stylesheet_evaluation_edits(style_source, native_edit_source)?;
    let native_edit_output = apply_normalized_static_stylesheet_evaluation_edits(
        style_source,
        &normalized_native_edit_source,
    );
    let native_edit_output_matches_evaluated_css = native_edit_output == evaluated_css;
    let native_edits = build_static_stylesheet_native_edits(
        normalized_native_edit_source,
        resolved_replacements.as_slice(),
    );
    let native_value_edit_count = native_edits
        .iter()
        .filter(|edit| edit.edit_kind == "valueReplacement")
        .count();
    let native_structural_edit_count = native_edits.len().saturating_sub(native_value_edit_count);
    Some(OmenaScssEvalStaticStylesheetEvaluationV0 {
        schema_version: "0",
        product: "omena-scss-eval.static-stylesheet-evaluation",
        evaluator: variable_kind.evaluator_label(),
        dialect: dialect_label(dialect),
        product_output_source: "nativeEditOutput",
        legacy_output_retained_as_oracle: true,
        legacy_output_consumed_until_cutover: false,
        replacement_count: resolved_replacements.len(),
        native_replacement_legacy_reflection_count,
        native_replacement_legacy_unreflected_count,
        native_edit_count: native_edits.len(),
        native_value_edit_count,
        native_structural_edit_count,
        native_edit_output_matches_evaluated_css,
        resolved_replacements,
        native_edits,
        value_resolution,
        native_edit_output,
        evaluated_css,
        oracle,
    })
}

fn count_native_replacements_reflected_in_legacy_css(
    replacements: &[OmenaScssEvalResolvedReplacementV0],
    evaluated_css: &str,
    dialect: StyleDialect,
) -> usize {
    replacements
        .iter()
        .filter(|replacement| {
            replacement
                .rendered_value
                .as_deref()
                .is_some_and(|rendered| {
                    abstract_css_value_reflected_in_legacy_css(
                        evaluated_css,
                        dialect,
                        rendered,
                        &replacement.abstract_value,
                    )
                })
        })
        .count()
}

fn build_static_stylesheet_native_edits(
    edits: Vec<StaticStylesheetEvaluationEdit>,
    replacements: &[OmenaScssEvalResolvedReplacementV0],
) -> Vec<OmenaScssEvalStaticStylesheetNativeEditV0> {
    edits
        .into_iter()
        .map(|edit| {
            let value_replacement =
                native_edit_value_replacement_for_static_edit(&edit, replacements);
            let edit_kind = value_replacement
                .map(|_| "valueReplacement")
                .unwrap_or_else(|| {
                    if edit.replacement.is_empty() {
                        "structuralRemoval"
                    } else {
                        "structuralReplacement"
                    }
                });
            OmenaScssEvalStaticStylesheetNativeEditV0 {
                start: edit.start,
                end: edit.end,
                replacement: edit.replacement,
                edit_kind,
                abstract_value: value_replacement
                    .map(|replacement| replacement.abstract_value.clone()),
                abstract_value_kind: value_replacement
                    .map(|replacement| replacement.abstract_value_kind),
            }
        })
        .collect()
}

fn native_edit_value_replacement_for_static_edit<'a>(
    edit: &StaticStylesheetEvaluationEdit,
    replacements: &'a [OmenaScssEvalResolvedReplacementV0],
) -> Option<&'a OmenaScssEvalResolvedReplacementV0> {
    replacements.iter().find(|replacement| {
        replacement.start == edit.start
            && replacement.end == edit.end
            && (replacement.text == edit.replacement
                || replacement.rendered_value.as_deref() == Some(edit.replacement.as_str()))
    })
}

pub(super) fn resolved_replacement_value(
    name: &str,
    start: usize,
    end: usize,
    text: &str,
) -> OmenaScssEvalResolvedReplacementV0 {
    let abstract_value = abstract_css_value_from_text(text);
    OmenaScssEvalResolvedReplacementV0 {
        name: name.to_string(),
        start,
        end,
        text: text.to_string(),
        rendered_value: render_static_abstract_value(&abstract_value),
        abstract_value_kind: abstract_css_value_kind(&abstract_value),
        abstract_value,
    }
}

pub(super) fn build_static_value_resolution_report(
    dialect: &'static str,
    values: Vec<OmenaScssEvalStaticValueResolutionV0>,
) -> OmenaScssEvalStaticValueResolutionReportV0 {
    let resolved_count = values
        .iter()
        .filter(|value| value.outcome == "resolved")
        .count();
    let raw_count = values
        .iter()
        .filter(|value| matches!(value.abstract_value, AbstractCssValueV0::Raw { .. }))
        .count();
    let top_count = values
        .iter()
        .filter(|value| matches!(value.abstract_value, AbstractCssValueV0::Top))
        .count();
    let cycle_count = values
        .iter()
        .filter(|value| value.reason == "cycle")
        .count();
    let fuel_exhausted_count = values
        .iter()
        .filter(|value| value.reason == "fuelExhausted")
        .count();
    let unresolved_reference_count = values
        .iter()
        .filter(|value| value.reason == "unresolvedReference")
        .count();
    let unsupported_dynamic_count = values
        .iter()
        .filter(|value| value.reason == "unsupportedDynamic")
        .count();
    OmenaScssEvalStaticValueResolutionReportV0 {
        schema_version: "0",
        product: "omena-scss-eval.static-value-resolution",
        mode: "oracleOnly",
        dialect,
        fuel_limit: STATIC_STYLESHEET_VALUE_RESOLUTION_FUEL_LIMIT,
        reference_count: values.len(),
        resolved_count,
        raw_count,
        top_count,
        cycle_count,
        fuel_exhausted_count,
        unresolved_reference_count,
        unsupported_dynamic_count,
        values,
    }
}

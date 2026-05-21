use crate::{
    TransformExecutionContextV0, TransformImportInlineV0,
    execute_transform_passes_on_source_with_dialect_and_context,
};
use omena_parser::StyleDialect;
use omena_transform_cst::TransformPassKind;

#[test]
fn execution_runtime_inlines_imports_from_explicit_replacements() {
    let source = r#"@import "./tokens.css"; @import url(./theme.css); @import "./conditional.css" layer(theme) supports(display: grid) screen and (min-width: 40rem); .button { color: var(--brand); }"#;
    let context = TransformExecutionContextV0 {
        import_inlines: vec![
            TransformImportInlineV0 {
                import_source: "./tokens.css".to_string(),
                replacement_css: r#":root { --brand: red; }"#.to_string(),
            },
            TransformImportInlineV0 {
                import_source: "./theme.css".to_string(),
                replacement_css: r#"@media screen { .theme { color: blue; } }"#.to_string(),
            },
            TransformImportInlineV0 {
                import_source: "./conditional.css".to_string(),
                replacement_css: r#".conditional { color: green; }"#.to_string(),
            },
        ],
        ..TransformExecutionContextV0::default()
    };
    let execution = execute_transform_passes_on_source_with_dialect_and_context(
        source,
        StyleDialect::Css,
        &[TransformPassKind::ImportInline, TransformPassKind::PrintCss],
        &context,
    );

    assert_eq!(execution.mutation_count, 3);
    assert_eq!(
        execution.output_css,
        r#":root { --brand: red; } @media screen { .theme { color: blue; } } @media screen and (min-width: 40rem) { @supports (display: grid) { @layer theme { .conditional { color: green; } } } } .button { color: var(--brand); }"#
    );
    assert_eq!(
        execution.css_import_inlines,
        vec![
            TransformImportInlineV0 {
                import_source: "./tokens.css".to_string(),
                replacement_css: r#":root { --brand: red; }"#.to_string(),
            },
            TransformImportInlineV0 {
                import_source: "./theme.css".to_string(),
                replacement_css: r#"@media screen { .theme { color: blue; } }"#.to_string(),
            },
            TransformImportInlineV0 {
                import_source: "./conditional.css".to_string(),
                replacement_css: r#".conditional { color: green; }"#.to_string(),
            },
        ]
    );
    assert_eq!(
        execution.executed_pass_ids,
        vec!["import-inline", "print-css"]
    );
}

#[test]
fn execution_runtime_inlines_less_imports_with_options() {
    let source = r#"@import (reference) "./tokens.less"; .button { color: @brand; }"#;
    let context = TransformExecutionContextV0 {
        import_inlines: vec![TransformImportInlineV0 {
            import_source: "./tokens.less".to_string(),
            replacement_css: r#"@brand: red; .base { color: @brand; }"#.to_string(),
        }],
        ..TransformExecutionContextV0::default()
    };
    let execution = execute_transform_passes_on_source_with_dialect_and_context(
        source,
        StyleDialect::Less,
        &[TransformPassKind::ImportInline, TransformPassKind::PrintCss],
        &context,
    );

    assert_eq!(execution.mutation_count, 1);
    assert_eq!(
        execution.output_css,
        r#"@brand: red; .button { color: @brand; }"#
    );
    assert_eq!(
        execution.executed_pass_ids,
        vec!["import-inline", "print-css"]
    );
}

#[test]
fn execution_runtime_inlines_less_imports_once_by_default() {
    let source =
        r#"@import "./tokens.less"; @import (once) "./tokens.less"; .button { color: @brand; }"#;
    let context = TransformExecutionContextV0 {
        import_inlines: vec![TransformImportInlineV0 {
            import_source: "./tokens.less".to_string(),
            replacement_css: r#"@brand: red; .base { color: @brand; }"#.to_string(),
        }],
        ..TransformExecutionContextV0::default()
    };
    let execution = execute_transform_passes_on_source_with_dialect_and_context(
        source,
        StyleDialect::Less,
        &[TransformPassKind::ImportInline, TransformPassKind::PrintCss],
        &context,
    );

    assert_eq!(execution.mutation_count, 2);
    assert_eq!(
        execution.output_css,
        r#"@brand: red; .base { color: @brand; }  .button { color: @brand; }"#
    );
    assert_eq!(
        execution.executed_pass_ids,
        vec!["import-inline", "print-css"]
    );
}

#[test]
fn execution_runtime_honors_less_multiple_imports() {
    let source = r#"@import "./tokens.less"; @import (multiple) "./tokens.less"; .button { color: @brand; }"#;
    let context = TransformExecutionContextV0 {
        import_inlines: vec![TransformImportInlineV0 {
            import_source: "./tokens.less".to_string(),
            replacement_css: r#"@brand: red; .base { color: @brand; }"#.to_string(),
        }],
        ..TransformExecutionContextV0::default()
    };
    let execution = execute_transform_passes_on_source_with_dialect_and_context(
        source,
        StyleDialect::Less,
        &[TransformPassKind::ImportInline, TransformPassKind::PrintCss],
        &context,
    );

    assert_eq!(execution.mutation_count, 2);
    assert_eq!(
        execution.output_css,
        r#"@brand: red; .base { color: @brand; } @brand: red; .base { color: @brand; } .button { color: @brand; }"#
    );
    assert_eq!(
        execution.executed_pass_ids,
        vec!["import-inline", "print-css"]
    );
}

#[test]
fn execution_runtime_removes_missing_optional_less_imports() {
    let source = r#"@import (optional) "./missing.less"; .button { color: red; }"#;
    let execution = execute_transform_passes_on_source_with_dialect_and_context(
        source,
        StyleDialect::Less,
        &[TransformPassKind::ImportInline, TransformPassKind::PrintCss],
        &TransformExecutionContextV0::default(),
    );

    assert_eq!(execution.mutation_count, 1);
    assert_eq!(execution.output_css, r#" .button { color: red; }"#);
    assert_eq!(
        execution.executed_pass_ids,
        vec!["import-inline", "print-css"]
    );
}

#[test]
fn execution_runtime_preserves_less_inline_imports_as_literal_css() {
    let source = r#"@import (inline) "./tokens.less"; .button { color: blue; }"#;
    let context = TransformExecutionContextV0 {
        import_inlines: vec![TransformImportInlineV0 {
            import_source: "./tokens.less".to_string(),
            replacement_css: r#"@brand: red; .base { color: @brand; }"#.to_string(),
        }],
        ..TransformExecutionContextV0::default()
    };
    let execution = execute_transform_passes_on_source_with_dialect_and_context(
        source,
        StyleDialect::Less,
        &[TransformPassKind::ImportInline, TransformPassKind::PrintCss],
        &context,
    );

    assert_eq!(execution.mutation_count, 1);
    assert_eq!(
        execution.output_css,
        r#"@brand: red; .base { color: @brand; } .button { color: blue; }"#
    );
    assert_eq!(
        execution.executed_pass_ids,
        vec!["import-inline", "print-css"]
    );
}

#[test]
fn execution_runtime_preserves_less_css_imports_as_css_imports() {
    let source = r#"@import (css) "./tokens.less" screen; .button { color: blue; }"#;
    let context = TransformExecutionContextV0 {
        import_inlines: vec![TransformImportInlineV0 {
            import_source: "./tokens.less".to_string(),
            replacement_css: r#"@brand: red; .base { color: @brand; }"#.to_string(),
        }],
        ..TransformExecutionContextV0::default()
    };
    let execution = execute_transform_passes_on_source_with_dialect_and_context(
        source,
        StyleDialect::Less,
        &[TransformPassKind::ImportInline, TransformPassKind::PrintCss],
        &context,
    );

    assert_eq!(execution.mutation_count, 1);
    assert_eq!(
        execution.output_css,
        r#"@import "./tokens.less" screen; .button { color: blue; }"#
    );
    assert_eq!(
        execution.executed_pass_ids,
        vec!["import-inline", "print-css"]
    );
}

#[test]
fn execution_runtime_leaves_unknown_less_import_options_untouched() {
    let source = r#"@import (plugin) "./tokens.less"; .button { color: blue; }"#;
    let context = TransformExecutionContextV0 {
        import_inlines: vec![TransformImportInlineV0 {
            import_source: "./tokens.less".to_string(),
            replacement_css: r#"@brand: red;"#.to_string(),
        }],
        ..TransformExecutionContextV0::default()
    };
    let execution = execute_transform_passes_on_source_with_dialect_and_context(
        source,
        StyleDialect::Less,
        &[TransformPassKind::ImportInline, TransformPassKind::PrintCss],
        &context,
    );

    assert_eq!(execution.mutation_count, 0);
    assert_eq!(execution.output_css, source);
    assert_eq!(
        execution.executed_pass_ids,
        vec!["import-inline", "print-css"]
    );
}

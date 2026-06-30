use super::execute_transform_passes_on_source_with_closed_world_context;
use crate::{TransformExecutionContextV0, execute_transform_passes_on_source};
use omena_parser::StyleDialect;
use omena_transform_cst::TransformPassKind;

#[test]
fn execution_runtime_keeps_keyframe_tree_shaking_planned_without_closed_world_context() {
    let source = r#"@keyframes unused { to { opacity: 1; } } .btn { color: red; }"#;
    let execution = execute_transform_passes_on_source(
        source,
        &[
            TransformPassKind::TreeShakeKeyframes,
            TransformPassKind::PrintCss,
        ],
    );

    assert_eq!(execution.output_css, source);
    assert_eq!(execution.mutation_count, 0);
    assert_eq!(execution.executed_pass_ids, vec!["print-css"]);
    assert_eq!(
        execution.planned_only_pass_ids,
        vec!["tree-shake-keyframes"]
    );
}

#[test]
fn execution_runtime_tree_shakes_keyframes_with_closed_world_context() {
    let source = r#"@-webkit-keyframes fade { to { opacity: 1; } } @keyframes fade { to { opacity: 1; } } @-webkit-keyframes spin { to { transform: rotate(1turn); } } @keyframes spin { to { transform: rotate(1turn); } } @-webkit-keyframes dead { to { opacity: 0; } } @keyframes dead { to { opacity: 0; } } @keyframes ghost { to { opacity: .5; } } .btn { animation: 1s ease fade; } .dead-ref { animation: ghost 1s ease; }"#;
    let context = TransformExecutionContextV0 {
        reachable_class_names: vec!["btn".to_string()],
        reachable_keyframe_names: vec!["spin".to_string()],
        ..TransformExecutionContextV0::default()
    };
    let execution = execute_transform_passes_on_source_with_closed_world_context(
        source,
        StyleDialect::Css,
        &[
            TransformPassKind::TreeShakeKeyframes,
            TransformPassKind::PrintCss,
        ],
        &context,
    );

    assert_eq!(execution.mutation_count, 3);
    assert_eq!(
        execution.output_css,
        r#"@-webkit-keyframes fade { to { opacity: 1; } } @keyframes fade { to { opacity: 1; } } @-webkit-keyframes spin { to { transform: rotate(1turn); } } @keyframes spin { to { transform: rotate(1turn); } }    .btn { animation: 1s ease fade; } .dead-ref { animation: ghost 1s ease; }"#
    );
    assert_eq!(
        execution.executed_pass_ids,
        vec!["tree-shake-keyframes", "print-css"]
    );
    assert_eq!(execution.semantic_removals.len(), 3);
    assert!(
        execution
            .semantic_removals
            .iter()
            .all(|removal| removal.symbol_kind == "keyframes")
    );
    assert!(
        execution
            .semantic_removals
            .iter()
            .any(|removal| removal.name == "dead" && removal.pass_id == "tree-shake-keyframes")
    );
    assert!(
        execution.semantic_removals.iter().any(|removal| {
            removal.name == "ghost" && removal.pass_id == "tree-shake-keyframes"
        })
    );
}

#[test]
fn execution_runtime_tree_shakes_nested_keyframes_with_closed_world_context() {
    let source = r#"@media screen { @keyframes spin { to { opacity: 1; } } @keyframes ghost { to { opacity: 0; } } } @supports (display: grid) { @-webkit-keyframes dead { to { opacity: .5; } } } .btn { animation: spin 1s; }"#;
    let context = TransformExecutionContextV0 {
        reachable_class_names: vec!["btn".to_string()],
        ..TransformExecutionContextV0::default()
    };
    let execution = execute_transform_passes_on_source_with_closed_world_context(
        source,
        StyleDialect::Css,
        &[
            TransformPassKind::TreeShakeKeyframes,
            TransformPassKind::PrintCss,
        ],
        &context,
    );

    assert_eq!(execution.mutation_count, 2);
    assert_eq!(
        execution.output_css,
        r#"@media screen { @keyframes spin { to { opacity: 1; } }  } @supports (display: grid) {  } .btn { animation: spin 1s; }"#
    );
    assert!(
        execution
            .semantic_removals
            .iter()
            .any(|removal| removal.name == "ghost")
    );
    assert!(
        execution
            .semantic_removals
            .iter()
            .any(|removal| removal.name == "dead")
    );
}

#[test]
fn execution_runtime_tree_shakes_quoted_keyframes_with_closed_world_context() {
    let source = r#"@keyframes "slide" { to { opacity: 1; } } @keyframes "fade in" { to { opacity: 1; } } @keyframes "ghost" { to { opacity: 0; } } .btn { animation-name: "slide"; } .alt { animation: "slide" 1s ease; } .space { animation: "fade in" 1s ease; }"#;
    let context = TransformExecutionContextV0 {
        ..TransformExecutionContextV0::default()
    };
    let execution = execute_transform_passes_on_source_with_closed_world_context(
        source,
        StyleDialect::Css,
        &[
            TransformPassKind::TreeShakeKeyframes,
            TransformPassKind::PrintCss,
        ],
        &context,
    );

    assert_eq!(execution.mutation_count, 1);
    assert_eq!(
        execution.output_css,
        r#"@keyframes "slide" { to { opacity: 1; } } @keyframes "fade in" { to { opacity: 1; } }  .btn { animation-name: "slide"; } .alt { animation: "slide" 1s ease; } .space { animation: "fade in" 1s ease; }"#
    );
    assert_eq!(
        execution.executed_pass_ids,
        vec!["tree-shake-keyframes", "print-css"]
    );
}

#[test]
fn execution_runtime_tree_shakes_escaped_keyframes_with_closed_world_context() {
    let source = r#"@keyframes spin\:fast { to { opacity: 1; } } @keyframes hex\3A fast { to { opacity: .5; } } @keyframes dead { to { opacity: 0; } } .btn { animation: spin\:fast 1s ease; } .dead-ref { animation: dead 1s ease; }"#;
    let context = TransformExecutionContextV0 {
        reachable_class_names: vec!["btn".to_string()],
        reachable_keyframe_names: vec!["hex:fast".to_string()],
        ..TransformExecutionContextV0::default()
    };
    let execution = execute_transform_passes_on_source_with_closed_world_context(
        source,
        StyleDialect::Css,
        &[
            TransformPassKind::TreeShakeKeyframes,
            TransformPassKind::PrintCss,
        ],
        &context,
    );

    assert!(execution.output_css.contains("@keyframes spin\\:fast"));
    assert!(execution.output_css.contains("@keyframes hex\\3A fast"));
    assert!(!execution.output_css.contains("@keyframes dead"));
    assert!(
        execution
            .output_css
            .contains(".btn { animation: spin\\:fast 1s ease; }")
    );
    assert_eq!(execution.mutation_count, 1);
    assert_eq!(
        execution.executed_pass_ids,
        vec!["tree-shake-keyframes", "print-css"]
    );
    assert!(
        execution
            .semantic_removals
            .iter()
            .any(|removal| { removal.name == "dead" && removal.pass_id == "tree-shake-keyframes" })
    );
}

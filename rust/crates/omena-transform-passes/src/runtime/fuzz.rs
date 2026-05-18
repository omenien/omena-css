use omena_parser::{StyleDialect, lex};
use omena_transform_cst::{
    TRANSFORM_PASS_CATALOG_LEN, TransformPassKind, all_transform_pass_kinds,
};

use crate::{
    TransformCascadeSafetyFuzzCaseV0, TransformCascadeSafetyFuzzResultV0,
    TransformClassNameRewriteV0, TransformDesignTokenRouteV0, TransformExecutionContextV0,
    TransformFuzzSeedReportV0, execute_transform_passes_on_source_with_dialect_and_context,
};

pub fn run_transform_cascade_safe_fuzz_case(
    case: TransformCascadeSafetyFuzzCaseV0,
) -> TransformCascadeSafetyFuzzResultV0 {
    let pass_count = case.pass_count.clamp(1, TRANSFORM_PASS_CATALOG_LEN);
    let source = generated_transform_fuzz_source(case.seed);
    let requested = generated_transform_fuzz_passes(case.seed, pass_count);
    let context = generated_transform_fuzz_context(case.seed);
    let execution = execute_transform_passes_on_source_with_dialect_and_context(
        &source,
        StyleDialect::Css,
        &requested,
        &context,
    );
    let lexed_output = lex(&execution.output_css, StyleDialect::Css);
    let requested_pass_ids = requested.iter().map(|pass| pass.id()).collect::<Vec<_>>();
    let output_byte_len = execution.output_css.len();
    let output_token_count = lexed_output.tokens().len();
    let output_error_count = lexed_output.errors().len();
    let provenance_node_count = execution.provenance_derivation_forest.node_count;
    let passed = execution.pass_plan.violated_dag_edge_count == 0
        && output_error_count == 0
        && output_byte_len <= source.len().saturating_mul(4).saturating_add(256)
        && provenance_node_count == execution.outcomes.len()
        && execution.provenance_preserved
            == execution
                .outcomes
                .iter()
                .all(|outcome| outcome.provenance_preserved);

    TransformCascadeSafetyFuzzResultV0 {
        seed: case.seed,
        pass_count,
        requested_pass_ids,
        executed_pass_ids: execution.executed_pass_ids,
        output_byte_len,
        output_token_count,
        output_error_count,
        provenance_node_count,
        passed,
    }
}

pub fn run_transform_fuzz_seed_corpus() -> TransformFuzzSeedReportV0 {
    let seeds = [1, 2, 3, 5, 8, 13, 21, 34, 55, 89, 144, 233];
    let results = seeds
        .into_iter()
        .enumerate()
        .map(|(index, seed)| {
            run_transform_cascade_safe_fuzz_case(TransformCascadeSafetyFuzzCaseV0 {
                seed,
                pass_count: index + 1,
            })
        })
        .collect::<Vec<_>>();
    let passed_count = results.iter().filter(|result| result.passed).count();
    let case_count = results.len();

    TransformFuzzSeedReportV0 {
        schema_version: "0",
        product: "omena-transform-passes.fuzz-seed-corpus",
        case_count,
        passed_count,
        failed_count: case_count - passed_count,
        results,
    }
}

fn generated_transform_fuzz_source(seed: u64) -> String {
    let mut state = seed ^ 0xe703_7ed1_a0b4_28db;
    let spacing = if fuzz_transform_next(&mut state).is_multiple_of(2) {
        "  "
    } else {
        "\n  "
    };
    let color = ["red", "blue", "oklch(60% 0.2 20)", "#ff0000"]
        [(fuzz_transform_next(&mut state) % 4) as usize];
    let margin = fuzz_transform_next(&mut state) % 24;
    format!(
        "/* fuzz:{seed} */\n.button-{seed} {{{spacing}--brand: {color};{spacing}color: var(--brand, red);{spacing}margin: {margin}.0px;{spacing}&__icon {{ color: var(--brand); }}\n}}\n@media (min-width: 40rem) {{ .button-{seed} {{ margin: {margin}px; }} }}\n"
    )
}

fn generated_transform_fuzz_passes(seed: u64, pass_count: usize) -> Vec<TransformPassKind> {
    let all_passes = all_transform_pass_kinds();
    let mut state = seed;
    let mut passes = Vec::new();
    for _ in 0..pass_count {
        let index = (fuzz_transform_next(&mut state) % all_passes.len() as u64) as usize;
        let pass = all_passes[index];
        if !passes.contains(&pass) {
            passes.push(pass);
        }
    }
    if passes.is_empty() {
        passes.push(TransformPassKind::PrintCss);
    }
    passes
}

fn generated_transform_fuzz_context(seed: u64) -> TransformExecutionContextV0 {
    let class_name = format!("button-{seed}");
    TransformExecutionContextV0 {
        closed_style_world: seed.is_multiple_of(2),
        reachable_class_names: vec![class_name.clone(), format!("{class_name}__icon")],
        reachable_keyframe_names: vec![format!("fade-{seed}")],
        reachable_value_names: vec![format!("spacing-{seed}")],
        reachable_custom_property_names: vec!["--brand".to_string()],
        class_name_rewrites: vec![TransformClassNameRewriteV0 {
            original_name: class_name,
            rewritten_name: format!("button-{seed}_hash"),
        }],
        design_token_routes: vec![TransformDesignTokenRouteV0 {
            token_name: "--brand".to_string(),
            routed_value: "var(--brand)".to_string(),
        }],
        ..TransformExecutionContextV0::default()
    }
}

fn fuzz_transform_next(state: &mut u64) -> u64 {
    *state = state
        .wrapping_mul(6_364_136_223_846_793_005)
        .wrapping_add(1_442_695_040_888_963_407);
    *state
}

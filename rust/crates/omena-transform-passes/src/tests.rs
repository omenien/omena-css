use super::{
    TransformCssModuleValueResolutionV0, TransformExecutionContextV0, TransformExecutionSummaryV0,
    TransformPassRuntimeStatus, execute_transform_passes_on_source,
    execute_transform_passes_on_source_with_dialect,
    execute_transform_passes_on_source_with_dialect_and_context,
    execute_transform_passes_on_source_with_dialect_context_and_closed_world_bundle,
};
#[cfg(feature = "lawvere-trace")]
use super::{
    evaluate_lawvere_reorderability_with_differential_corpus,
    execute_transform_passes_on_source_with_lawvere_trace,
    plan_transform_passes_parallel_lawvere_layers,
};
use crate::domains::css_modules_classes::local_css_module_composes_resolutions_with_lexer;
use omena_parser::{
    ClosedWorldBundleV0, ClosedWorldLinkedModuleV0, ConfigurationHashV0, ModuleIdV0,
    ModuleInstanceKeyV0, StyleDialect,
};
use omena_transform_cst::TransformPassKind;
use std::collections::BTreeSet;

mod class_hashing;
mod design_tokens;
mod import_inline;
mod module_evaluation;
mod nesting_layers;
mod rule_optimization;
mod runtime_boundary;
mod runtime_smoke;
mod scalar_normalization;
mod selector_structure;
mod shorthand_combining;
mod static_conditionals;
mod static_resolution;
mod token_normalization;
mod tree_shake_classes;
mod tree_shake_custom_properties;
mod tree_shake_keyframes;
mod tree_shake_values;
mod value_lowering;

fn test_closed_world_bundle_from_context(
    source: &str,
    dialect: StyleDialect,
    context: &TransformExecutionContextV0,
) -> ClosedWorldBundleV0 {
    let instance = ModuleInstanceKeyV0::new(
        ModuleIdV0::new("omena-transform-passes.test.current"),
        ConfigurationHashV0::none(),
    );
    let mut class_names = context
        .reachable_class_names
        .iter()
        .cloned()
        .collect::<BTreeSet<_>>();
    let mut resolutions = context.css_module_composes_resolutions.clone();
    resolutions.extend(local_css_module_composes_resolutions_with_lexer(
        source, dialect,
    ));
    let mut changed = true;
    while changed {
        changed = false;
        for resolution in &resolutions {
            if class_names.contains(&resolution.local_class_name) {
                for exported in &resolution.exported_class_names {
                    changed |= class_names.insert(exported.clone());
                }
            }
        }
    }

    let mut module = ClosedWorldLinkedModuleV0::new(instance.clone());
    for name in class_names {
        module = module.with_class_name(name);
    }
    for name in &context.reachable_keyframe_names {
        module = module.with_keyframe_name(name.clone());
    }
    for name in &context.reachable_value_names {
        module = module.with_value_name(name.clone());
    }
    for name in &context.reachable_custom_property_names {
        module = module.with_custom_property_name(name.clone());
    }

    closed_world_bundle_or_abort(ClosedWorldBundleV0::try_from_linked_modules(
        vec![instance],
        vec![module],
    ))
}

#[allow(clippy::panic)]
fn closed_world_bundle_or_abort(
    bundle: Result<ClosedWorldBundleV0, omena_parser::ClosedWorldBundleBuildErrorV0>,
) -> ClosedWorldBundleV0 {
    match bundle {
        Ok(bundle) => bundle,
        Err(err) => panic!("test closed-world bundle should be constructible: {err:?}"),
    }
}

fn execute_transform_passes_on_source_with_closed_world_context(
    source: &str,
    dialect: StyleDialect,
    requested: &[TransformPassKind],
    context: &TransformExecutionContextV0,
) -> TransformExecutionSummaryV0 {
    let bundle = test_closed_world_bundle_from_context(source, dialect, context);
    execute_transform_passes_on_source_with_dialect_context_and_closed_world_bundle(
        source, dialect, requested, context, &bundle,
    )
}

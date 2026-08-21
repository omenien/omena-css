use omena_benchmarks::bundler_productization_corpus;
use omena_bundler::{
    EmissionOrderingPolicyV0, TransformBundleLinkErrorV0, TransformBundleLinkOptionsV0,
    TransformBundleModuleInputV0,
    link_omena_transform_bundle_projection_with_emission_items_and_resolved_dependencies_and_options,
    project_omena_transform_bundle_linker_and_emission_items,
};
use omena_parser::StyleDialect;
use serde::Serialize;

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
struct DialectCycleRealCorpusRefusalDiffV0 {
    schema_version: &'static str,
    product: &'static str,
    comparison_baseline: &'static str,
    current_policy: &'static str,
    input_count: usize,
    module_count: usize,
    newly_refused_input_count: usize,
    input_ids: Vec<String>,
    newly_refused_input_ids: Vec<String>,
}

fn summarize_dialect_cycle_real_corpus_refusal_diff_v0()
-> Result<DialectCycleRealCorpusRefusalDiffV0, String> {
    let samples = bundler_productization_corpus();
    let mut module_count = 0;
    let mut newly_refused_input_ids = Vec::new();
    for sample in &samples {
        let mut modules = vec![TransformBundleModuleInputV0::new(
            sample.path,
            sample.source.clone(),
            sample.dialect,
        )];
        if sample.name == "next-dashboard-shell-scss" {
            modules.push(TransformBundleModuleInputV0::new(
                "tokens.scss",
                "$accent: #0f766e;",
                StyleDialect::Scss,
            ));
        }
        module_count += modules.len();
        let projections = project_omena_transform_bundle_linker_and_emission_items(&modules, &[]);
        match link_omena_transform_bundle_projection_with_emission_items_and_resolved_dependencies_and_options(
            &[sample.path],
            projections.linker_projection(),
            projections.emission_item_projection(),
            &[],
            &[],
            TransformBundleLinkOptionsV0::default()
                .with_emission_ordering_policy(EmissionOrderingPolicyV0::ImportOrderPreserving),
        ) {
            Ok(_) => {}
            Err(TransformBundleLinkErrorV0::UnsupportedDialectEmissionCycle { .. }) => {
                newly_refused_input_ids.push(sample.name.to_string());
            }
            Err(error) => {
                return Err(format!(
                    "real corpus fixture {} failed outside the dialect-cycle boundary: {error:?}",
                    sample.name
                ));
            }
        }
    }
    Ok(DialectCycleRealCorpusRefusalDiffV0 {
        schema_version: "0",
        product: "omena-diff-test.dialect-cycle-real-corpus-refusal-diff",
        comparison_baseline: "moduleIdentityTieBreakForImportCycles",
        current_policy: "unsupportedDialectEmissionCycleFailsClosed",
        input_count: samples.len(),
        module_count,
        newly_refused_input_count: newly_refused_input_ids.len(),
        input_ids: samples
            .into_iter()
            .map(|sample| sample.name.to_string())
            .collect(),
        newly_refused_input_ids,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dialect_cycle_real_corpus_refusal_diff_is_measured() -> Result<(), String> {
        let report = summarize_dialect_cycle_real_corpus_refusal_diff_v0()?;
        assert!(report.input_count > 0);
        assert!(report.module_count > report.input_count);
        assert_eq!(report.newly_refused_input_count, 0);
        eprintln!(
            "DIALECT_CYCLE_REAL_CORPUS_DIFF={}",
            serde_json::to_string(&report).map_err(|error| error.to_string())?
        );
        Ok(())
    }
}

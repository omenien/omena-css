use omena_benchmarks::bundler_productization_corpus;
use omena_bundler::{
    EmissionOrderingPolicyV0, TransformBundleDependencyResolutionV0, TransformBundleLinkErrorV0,
    TransformBundleLinkOptionsV0, TransformBundleModuleInputV0,
    TransformBundleResolvedDependencyV0,
    link_omena_transform_bundle_projection_with_emission_items_and_resolved_dependencies_and_options,
    project_omena_transform_bundle_linker_and_emission_items,
};
use omena_parser::StyleDialect;
use serde::Serialize;

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
struct DialectCycleCorpusRefusalMeasurementV1 {
    schema_version: &'static str,
    product: &'static str,
    comparison_baseline: &'static str,
    comparison_baseline_executed: bool,
    current_policy: &'static str,
    product_corpus_input_count: usize,
    edge_fixture_input_count: usize,
    input_count: usize,
    module_count: usize,
    dependency_edge_count: usize,
    edge_bearing_input_count: usize,
    known_acyclic_edge_input_count: usize,
    known_cyclic_edge_input_count: usize,
    current_policy_refused_input_count: usize,
    input_ids: Vec<String>,
    current_policy_refused_input_ids: Vec<String>,
}

#[derive(Debug, Clone)]
struct DialectCycleCorpusInputV1 {
    id: String,
    entrypoint_paths: Vec<String>,
    modules: Vec<TransformBundleModuleInputV0>,
    known_cycle_shape: Option<bool>,
}

fn dialect_cycle_corpus_inputs_v1() -> (usize, Vec<DialectCycleCorpusInputV1>) {
    let samples = bundler_productization_corpus();
    let product_corpus_input_count = samples.len();
    let mut inputs = samples
        .into_iter()
        .map(|sample| DialectCycleCorpusInputV1 {
            id: sample.name.to_string(),
            entrypoint_paths: vec![sample.path.to_string()],
            modules: vec![TransformBundleModuleInputV0::new(
                sample.path,
                sample.source,
                sample.dialect,
            )],
            known_cycle_shape: None,
        })
        .collect::<Vec<_>>();
    inputs.push(DialectCycleCorpusInputV1 {
        id: "css-import-acyclic-edge".to_string(),
        entrypoint_paths: vec!["fixtures/acyclic/app.css".to_string()],
        modules: vec![
            TransformBundleModuleInputV0::new(
                "fixtures/acyclic/app.css",
                "@import \"./tokens.css\"; .app { color: var(--brand); }",
                StyleDialect::Css,
            ),
            TransformBundleModuleInputV0::new(
                "fixtures/acyclic/tokens.css",
                ":root { --brand: #0f766e; }",
                StyleDialect::Css,
            ),
        ],
        known_cycle_shape: Some(false),
    });
    inputs.push(DialectCycleCorpusInputV1 {
        id: "scss-use-cycle-edge".to_string(),
        entrypoint_paths: vec!["fixtures/cyclic/a.scss".to_string()],
        modules: vec![
            TransformBundleModuleInputV0::new(
                "fixtures/cyclic/a.scss",
                "@use \"./b.scss\"; .a { color: red; }",
                StyleDialect::Scss,
            ),
            TransformBundleModuleInputV0::new(
                "fixtures/cyclic/b.scss",
                "@use \"./a.scss\"; .b { color: blue; }",
                StyleDialect::Scss,
            ),
        ],
        known_cycle_shape: Some(true),
    });
    (product_corpus_input_count, inputs)
}

fn summarize_dialect_cycle_corpus_refusal_measurement_v1()
-> Result<DialectCycleCorpusRefusalMeasurementV1, String> {
    let (product_corpus_input_count, inputs) = dialect_cycle_corpus_inputs_v1();
    let mut module_count = 0;
    let mut dependency_edge_count = 0;
    let mut edge_bearing_input_count = 0;
    let mut current_policy_refused_input_ids = Vec::new();
    for input in &inputs {
        module_count += input.modules.len();
        let projections =
            project_omena_transform_bundle_linker_and_emission_items(&input.modules, &[]);
        let input_dependency_edge_count = projections
            .linker_projection()
            .inputs()
            .iter()
            .map(|linker_input| linker_input.dependency_edges.len())
            .sum::<usize>();
        dependency_edge_count += input_dependency_edge_count;
        edge_bearing_input_count += usize::from(input_dependency_edge_count > 0);
        let mut resolved_dependencies = Vec::new();
        for source in projections.linker_projection().inputs() {
            for edge in &source.dependency_edges {
                let relative_target = edge
                    .import_source
                    .strip_prefix("./")
                    .unwrap_or(edge.import_source.as_str());
                let target_path = std::path::Path::new(source.source_path.as_str())
                    .parent()
                    .unwrap_or_else(|| std::path::Path::new(""))
                    .join(relative_target)
                    .to_string_lossy()
                    .into_owned();
                let target = projections
                    .linker_projection()
                    .inputs()
                    .iter()
                    .find(|candidate| candidate.source_path == target_path)
                    .ok_or_else(|| {
                        format!(
                            "dialect-cycle corpus fixture {} lacks resolved target {} for {}",
                            input.id, target_path, edge.import_source
                        )
                    })?;
                resolved_dependencies.push(TransformBundleResolvedDependencyV0::new(
                    source.instance.clone(),
                    edge.kind,
                    edge.import_source.clone(),
                    edge.import_ordinal,
                    TransformBundleDependencyResolutionV0::attempted(
                        vec!["dialectCycleCorpusImportGraph"],
                        "dialectCycleCorpusImportGraph",
                        1,
                        Some(target.instance.clone()),
                    ),
                ));
            }
        }
        match link_omena_transform_bundle_projection_with_emission_items_and_resolved_dependencies_and_options(
            &input.entrypoint_paths,
            projections.linker_projection(),
            projections.emission_item_projection(),
            &resolved_dependencies,
            &[],
            TransformBundleLinkOptionsV0::default()
                .with_emission_ordering_policy(EmissionOrderingPolicyV0::ImportOrderPreserving),
        ) {
            Ok(_) => {}
            Err(TransformBundleLinkErrorV0::UnsupportedDialectEmissionCycle { .. }) => {
                current_policy_refused_input_ids.push(input.id.clone());
            }
            Err(error) => {
                return Err(format!(
                    "dialect-cycle corpus fixture {} failed outside the dialect-cycle boundary: {error:?}",
                    input.id
                ));
            }
        }
    }
    Ok(DialectCycleCorpusRefusalMeasurementV1 {
        schema_version: "1",
        product: "omena-diff-test.dialect-cycle-corpus-refusal-measurement",
        comparison_baseline: "historicalModuleIdentityTieBreakReferenceOnly",
        comparison_baseline_executed: false,
        current_policy: "unsupportedDialectEmissionCycleFailsClosed",
        product_corpus_input_count,
        edge_fixture_input_count: inputs.len().saturating_sub(product_corpus_input_count),
        input_count: inputs.len(),
        module_count,
        dependency_edge_count,
        edge_bearing_input_count,
        known_acyclic_edge_input_count: inputs
            .iter()
            .filter(|input| input.known_cycle_shape == Some(false))
            .count(),
        known_cyclic_edge_input_count: inputs
            .iter()
            .filter(|input| input.known_cycle_shape == Some(true))
            .count(),
        current_policy_refused_input_count: current_policy_refused_input_ids.len(),
        input_ids: inputs.into_iter().map(|input| input.id).collect(),
        current_policy_refused_input_ids,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dialect_cycle_corpus_refusal_measurement_is_edge_bearing() -> Result<(), String> {
        let report = summarize_dialect_cycle_corpus_refusal_measurement_v1()?;
        assert_eq!(report.product_corpus_input_count, 3);
        assert_eq!(report.edge_fixture_input_count, 2);
        assert_eq!(report.input_count, 5);
        assert!(report.module_count > report.input_count);
        assert_eq!(report.dependency_edge_count, 3);
        assert_eq!(report.edge_bearing_input_count, 2);
        assert_eq!(report.known_acyclic_edge_input_count, 1);
        assert_eq!(report.known_cyclic_edge_input_count, 1);
        assert!(!report.comparison_baseline_executed);
        assert_eq!(
            report.current_policy_refused_input_ids,
            vec!["scss-use-cycle-edge"]
        );
        assert_eq!(report.current_policy_refused_input_count, 1);
        eprintln!(
            "DIALECT_CYCLE_CORPUS_MEASUREMENT={}",
            serde_json::to_string(&report).map_err(|error| error.to_string())?
        );
        Ok(())
    }
}

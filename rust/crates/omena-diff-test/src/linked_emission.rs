use std::collections::{BTreeMap, BTreeSet};

use omena_benchmarks::bundler_productization_corpus;
use omena_bundler::{TransformBundleModuleInputV0, link_omena_transform_bundle_modules};
use omena_parser::{
    ParsedStyleFacts, StyleDialect, collect_style_facts, summarize_omena_parser_style_facts,
};
use omena_query::{
    OmenaQueryBundleEmissionPathV0, OmenaQueryBundlePlanInputV0, OmenaQueryConsumerBuildOptionsV0,
    OmenaQueryStyleResolutionInputsV0, OmenaQueryStyleSourceInputV0,
    OmenaQueryTransformExecutionContextV0, compare_omena_query_transform_css_semantics_v0,
    run_omena_query_bundle_with_semantic_inputs_and_options,
};
use serde::Serialize;
use sha2::{Digest, Sha256};

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
/// Controlled output perturbations used to exercise linked-emission classification.
pub enum LinkedEmissionByteDifferentialPerturbationV0 {
    #[default]
    None,
    AddUnexpectedRule,
    CollapseToLegacyBytes,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
/// Classification of the byte difference between legacy and linked emission.
pub enum LinkedEmissionByteDifferenceClassV0 {
    Equivalent,
    Expected,
    Unexpected,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize)]
#[serde(rename_all = "camelCase")]
/// Known reason that linked emission may differ from legacy output bytes.
pub enum LinkedEmissionByteDifferenceReasonV0 {
    GlobalModuleOrder,
    EntryInterleaveCollapse,
    PerModuleGrouping,
    SharedImportSingleEmission,
    FormattingNormalization,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
/// Byte and semantic comparison for one linked-emission fixture.
pub struct LinkedEmissionByteDifferentialCaseV0 {
    pub fixture_id: String,
    pub module_count: usize,
    pub legacy_emission_path: &'static str,
    pub linked_emission_path: &'static str,
    pub legacy_sha256: String,
    pub linked_sha256: String,
    pub legacy_byte_len: usize,
    pub linked_byte_len: usize,
    pub byte_equal: bool,
    pub semantic_preserved: bool,
    pub semantic_mismatch_count: usize,
    pub authoritative_marker_order: Vec<String>,
    pub legacy_marker_order: Vec<String>,
    pub linked_marker_order: Vec<String>,
    pub linked_modules_emitted_once: bool,
    pub difference_class: LinkedEmissionByteDifferenceClassV0,
    pub reasons: Vec<LinkedEmissionByteDifferenceReasonV0>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
/// Aggregate linked-emission differential results across the shared corpus.
pub struct LinkedEmissionByteDifferentialReportV0 {
    pub schema_version: &'static str,
    pub product: &'static str,
    pub fixture_count: usize,
    pub equivalent_count: usize,
    pub expected_divergence_count: usize,
    pub unexpected_divergence_count: usize,
    pub total_divergence_count: usize,
    pub cases: Vec<LinkedEmissionByteDifferentialCaseV0>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
/// Coverage of one independently enumerated linked-emission shape.
pub struct LinkedEmissionCoverageShapeV0 {
    pub shape_class: String,
    pub fixture_ids: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
/// A linked-emission shape that has no exercising fixture yet.
pub struct LinkedEmissionNotCoveredShapeV0 {
    pub shape_class: String,
    pub reentry: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
/// Per-fixture visibility of modules to the class-marker order oracle.
pub struct LinkedEmissionFixtureObservabilityV0 {
    pub fixture_id: String,
    pub module_count: usize,
    pub marker_observable_module_count: usize,
    pub blind_spot_module_count: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
/// A module that the class-marker order oracle cannot observe.
pub struct LinkedEmissionMarkerBlindSpotV0 {
    pub fixture_id: String,
    pub module_path: String,
    pub shape_classes: Vec<String>,
    pub emission_plan_entry_count: usize,
    pub fact_categories: Vec<&'static str>,
    pub output_bytes_differ: bool,
    pub marker_orders_agree: bool,
    pub linked_marker_order_matches_authority: bool,
    pub semantic_difference_observed: bool,
    pub difference_reason_observed: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
/// An import-order witness that exercises a module absent from the selector-only plan.
pub struct LinkedEmissionPlacementWitnessV0 {
    pub witness_id: String,
    pub selectorless_module_paths: Vec<String>,
    pub emission_plan_entry_count: usize,
    pub output_bytes_differ: bool,
    pub marker_orders_agree: bool,
    pub linked_marker_order_matches_authority: bool,
    pub semantic_difference_observed: bool,
    pub difference_reason_observed: bool,
    pub import_graph_winner: String,
    pub legacy_winner: String,
    pub linked_winner: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
/// Machine-derived coverage over the independently enumerated bundle-shape population.
pub struct LinkedEmissionCoverageCensusV0 {
    pub schema_version: &'static str,
    pub product: &'static str,
    pub coverage_scope: &'static str,
    pub full_corpus_coverage: bool,
    pub population_count: usize,
    pub covered_shape_count: usize,
    pub not_covered_shape_count: usize,
    pub fixture_count: usize,
    pub module_count: usize,
    pub marker_observable_module_count: usize,
    pub blind_spot_module_count: usize,
    pub shapes: Vec<LinkedEmissionCoverageShapeV0>,
    pub not_covered: Vec<LinkedEmissionNotCoveredShapeV0>,
    pub fixture_observability: Vec<LinkedEmissionFixtureObservabilityV0>,
    pub blind_spots: Vec<LinkedEmissionMarkerBlindSpotV0>,
    pub placement_witnesses: Vec<LinkedEmissionPlacementWitnessV0>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
/// Repository-internal stdout contract for linked-emission differential consumers.
pub struct LinkedEmissionByteDifferentialEnvelopeV0 {
    pub schema_version: &'static str,
    pub product: &'static str,
    pub report: LinkedEmissionByteDifferentialReportV0,
    pub census: LinkedEmissionCoverageCensusV0,
}

#[derive(Debug, Clone, Copy)]
struct LinkedEmissionCoverageShapeDefinitionV0 {
    shape_class: &'static str,
    reentry: &'static str,
}

#[derive(Debug, Clone)]
struct LinkedEmissionFixtureModuleV0 {
    path: String,
    source: String,
    dialect: StyleDialect,
    marker_names: Vec<String>,
}

#[derive(Debug, Clone)]
struct LinkedEmissionFixtureV0 {
    id: String,
    entry_path: String,
    shape_classes: Vec<&'static str>,
    modules: Vec<LinkedEmissionFixtureModuleV0>,
}

#[derive(Debug)]
struct LinkedEmissionFixtureAnalysisV0 {
    case: LinkedEmissionByteDifferentialCaseV0,
    linked_order: omena_bundler::LinkedStylesheetV0,
    legacy_css: String,
    linked_css: String,
}

#[derive(Debug)]
struct LinkedEmissionPlacementWitnessDefinitionV0 {
    fixture: LinkedEmissionFixtureV0,
    selectorless_module_paths: &'static [&'static str],
    import_graph_winner: &'static str,
}

const LINKED_EMISSION_COVERAGE_POPULATION_V0: &[LinkedEmissionCoverageShapeDefinitionV0] = &[
    LinkedEmissionCoverageShapeDefinitionV0 {
        shape_class: "css-import-order",
        reentry: "retain the CSS import-order differential fixture",
    },
    LinkedEmissionCoverageShapeDefinitionV0 {
        shape_class: "scss-import-order",
        reentry: "retain the SCSS import-order differential fixture",
    },
    LinkedEmissionCoverageShapeDefinitionV0 {
        shape_class: "less-import-order",
        reentry: "retain the Less import-order differential fixture",
    },
    LinkedEmissionCoverageShapeDefinitionV0 {
        shape_class: "shared-import-diamond",
        reentry: "retain a shared-import diamond with single-emission evidence",
    },
    LinkedEmissionCoverageShapeDefinitionV0 {
        shape_class: "large-product-corpus",
        reentry: "retain a fixture sourced from the bundler productization corpus",
    },
    LinkedEmissionCoverageShapeDefinitionV0 {
        shape_class: "element-only-reset",
        reentry: "add a linked module whose emitted rules use only element selectors",
    },
    LinkedEmissionCoverageShapeDefinitionV0 {
        shape_class: "bare-layer-statement",
        reentry: "add a linked module containing a bare cascade-layer statement",
    },
    LinkedEmissionCoverageShapeDefinitionV0 {
        shape_class: "font-face-only",
        reentry: "add a linked module containing only a font-face rule",
    },
    LinkedEmissionCoverageShapeDefinitionV0 {
        shape_class: "configured-sass-module-instance",
        reentry: "add two configured instances of one Sass module to the differential corpus",
    },
    LinkedEmissionCoverageShapeDefinitionV0 {
        shape_class: "tsconfig-path-alias-import",
        reentry: "add a linked fixture resolved through a tsconfig path mapping",
    },
    LinkedEmissionCoverageShapeDefinitionV0 {
        shape_class: "package-export-import",
        reentry: "add a linked fixture resolved through package exports",
    },
    LinkedEmissionCoverageShapeDefinitionV0 {
        shape_class: "code-split-entry-closure",
        reentry: "add multiple bundle entries with independently measured dependency closures",
    },
];

/// Compares legacy and linked emission for the shared corpus.
///
/// The optional perturbation lets callers confirm that unexpected byte changes
/// remain distinguishable from the documented linked-order differences.
pub fn summarize_linked_emission_byte_differential_v0(
    perturbation: LinkedEmissionByteDifferentialPerturbationV0,
) -> Result<LinkedEmissionByteDifferentialReportV0, String> {
    summarize_linked_emission_byte_differential_envelope_v0(perturbation)
        .map(|envelope| envelope.report)
}

/// Produces the differential report and its live coverage census in one run.
pub fn summarize_linked_emission_byte_differential_envelope_v0(
    perturbation: LinkedEmissionByteDifferentialPerturbationV0,
) -> Result<LinkedEmissionByteDifferentialEnvelopeV0, String> {
    let fixtures = linked_emission_fixtures_v0();
    let analyses = fixtures
        .iter()
        .enumerate()
        .map(|(index, fixture)| {
            let case_perturbation = match perturbation {
                LinkedEmissionByteDifferentialPerturbationV0::AddUnexpectedRule if index == 0 => {
                    perturbation
                }
                LinkedEmissionByteDifferentialPerturbationV0::CollapseToLegacyBytes => perturbation,
                _ => LinkedEmissionByteDifferentialPerturbationV0::None,
            };
            analyze_linked_emission_fixture_v0(fixture, case_perturbation)
        })
        .collect::<Result<Vec<_>, _>>()?;
    let census = summarize_linked_emission_coverage_census_v0(&fixtures, &analyses)?;
    let cases = analyses
        .into_iter()
        .map(|analysis| analysis.case)
        .collect::<Vec<_>>();
    let equivalent_count = cases
        .iter()
        .filter(|case| case.difference_class == LinkedEmissionByteDifferenceClassV0::Equivalent)
        .count();
    let expected_divergence_count = cases
        .iter()
        .filter(|case| case.difference_class == LinkedEmissionByteDifferenceClassV0::Expected)
        .count();
    let unexpected_divergence_count = cases
        .iter()
        .filter(|case| case.difference_class == LinkedEmissionByteDifferenceClassV0::Unexpected)
        .count();

    let report = LinkedEmissionByteDifferentialReportV0 {
        schema_version: "0",
        product: "omena-diff-test.linked-emission-byte-differential",
        fixture_count: cases.len(),
        equivalent_count,
        expected_divergence_count,
        unexpected_divergence_count,
        total_divergence_count: expected_divergence_count + unexpected_divergence_count,
        cases,
    };

    Ok(LinkedEmissionByteDifferentialEnvelopeV0 {
        schema_version: "0",
        product: "omena-diff-test.linked-emission-byte-differential-envelope",
        report,
        census,
    })
}

fn summarize_linked_emission_coverage_census_v0(
    fixtures: &[LinkedEmissionFixtureV0],
    analyses: &[LinkedEmissionFixtureAnalysisV0],
) -> Result<LinkedEmissionCoverageCensusV0, String> {
    if fixtures.len() != analyses.len() {
        return Err(format!(
            "linked-emission fixture/analysis count mismatch: {} fixtures, {} analyses",
            fixtures.len(),
            analyses.len()
        ));
    }
    let mut population = BTreeMap::new();
    for definition in LINKED_EMISSION_COVERAGE_POPULATION_V0 {
        if population
            .insert(definition.shape_class, definition.reentry)
            .is_some()
        {
            return Err(format!(
                "duplicate linked-emission shape population row {}",
                definition.shape_class
            ));
        }
    }

    let mut fixtures_by_shape = population
        .keys()
        .map(|shape_class| (*shape_class, BTreeSet::new()))
        .collect::<BTreeMap<_, _>>();
    let mut fixture_ids = BTreeSet::new();
    let mut fixture_observability = Vec::new();
    let mut blind_spots = Vec::new();
    let mut module_count = 0usize;
    let mut marker_observable_module_count = 0usize;

    for (fixture, analysis) in fixtures.iter().zip(analyses) {
        if fixture.id != analysis.case.fixture_id {
            return Err(format!(
                "linked-emission fixture/analysis id mismatch: {} != {}",
                fixture.id, analysis.case.fixture_id
            ));
        }
        if !fixture_ids.insert(fixture.id.as_str()) {
            return Err(format!(
                "duplicate linked-emission fixture id {}",
                fixture.id
            ));
        }
        if fixture.shape_classes.is_empty() {
            return Err(format!(
                "linked-emission fixture {} has no enumerated shape class",
                fixture.id
            ));
        }
        for shape_class in &fixture.shape_classes {
            let Some(shape_fixtures) = fixtures_by_shape.get_mut(shape_class) else {
                return Err(format!(
                    "linked-emission fixture {} names unknown shape class {}",
                    fixture.id, shape_class
                ));
            };
            shape_fixtures.insert(fixture.id.clone());
        }

        let fixture_marker_observable_module_count = fixture
            .modules
            .iter()
            .filter(|module| !module.marker_names.is_empty())
            .count();
        for module in fixture
            .modules
            .iter()
            .filter(|module| module.marker_names.is_empty())
        {
            let emission_plan_entry_count = analysis
                .linked_order
                .global_rule_order
                .rules
                .iter()
                .filter(|rule| rule.module_instance.module().as_str() == module.path)
                .count();
            let facts = collect_style_facts(module.source.as_str(), module.dialect);
            blind_spots.push(LinkedEmissionMarkerBlindSpotV0 {
                fixture_id: fixture.id.clone(),
                module_path: module.path.clone(),
                shape_classes: fixture
                    .shape_classes
                    .iter()
                    .map(|shape_class| (*shape_class).to_string())
                    .collect(),
                emission_plan_entry_count,
                fact_categories: populated_fact_categories_v0(&facts),
                output_bytes_differ: !analysis.case.byte_equal,
                marker_orders_agree: analysis.case.legacy_marker_order
                    == analysis.case.linked_marker_order,
                linked_marker_order_matches_authority: analysis.case.linked_marker_order
                    == analysis.case.authoritative_marker_order,
                semantic_difference_observed: analysis.case.semantic_mismatch_count > 0,
                difference_reason_observed: !analysis.case.reasons.is_empty(),
            });
        }
        module_count += fixture.modules.len();
        marker_observable_module_count += fixture_marker_observable_module_count;
        fixture_observability.push(LinkedEmissionFixtureObservabilityV0 {
            fixture_id: fixture.id.clone(),
            module_count: fixture.modules.len(),
            marker_observable_module_count: fixture_marker_observable_module_count,
            blind_spot_module_count: fixture.modules.len() - fixture_marker_observable_module_count,
        });
    }

    let shapes = fixtures_by_shape
        .iter()
        .map(|(shape_class, fixture_ids)| LinkedEmissionCoverageShapeV0 {
            shape_class: (*shape_class).to_string(),
            fixture_ids: fixture_ids.iter().cloned().collect(),
        })
        .collect::<Vec<_>>();
    let not_covered = fixtures_by_shape
        .iter()
        .filter(|(_, fixture_ids)| fixture_ids.is_empty())
        .map(|(shape_class, _)| LinkedEmissionNotCoveredShapeV0 {
            shape_class: (*shape_class).to_string(),
            reentry: population
                .get(shape_class)
                .copied()
                .unwrap_or_default()
                .to_string(),
        })
        .collect::<Vec<_>>();
    let full_corpus_coverage =
        not_covered.is_empty() && shapes.iter().all(|shape| !shape.fixture_ids.is_empty());
    let placement_witnesses = linked_emission_placement_witness_definitions_v0()
        .iter()
        .map(summarize_linked_emission_placement_witness_v0)
        .collect::<Result<Vec<_>, _>>()?;

    Ok(LinkedEmissionCoverageCensusV0 {
        schema_version: "0",
        product: "omena-diff-test.linked-emission-coverage-census",
        coverage_scope: if full_corpus_coverage {
            "fullCorpus"
        } else {
            "boundedMultiModuleFixtures"
        },
        full_corpus_coverage,
        population_count: shapes.len(),
        covered_shape_count: shapes.len() - not_covered.len(),
        not_covered_shape_count: not_covered.len(),
        fixture_count: fixtures.len(),
        module_count,
        marker_observable_module_count,
        blind_spot_module_count: blind_spots.len(),
        shapes,
        not_covered,
        fixture_observability,
        blind_spots,
        placement_witnesses,
    })
}

fn analyze_linked_emission_fixture_v0(
    fixture: &LinkedEmissionFixtureV0,
    perturbation: LinkedEmissionByteDifferentialPerturbationV0,
) -> Result<LinkedEmissionFixtureAnalysisV0, String> {
    let style_sources = fixture
        .modules
        .iter()
        .map(|module| OmenaQueryStyleSourceInputV0 {
            style_path: module.path.clone(),
            style_source: module.source.clone(),
        })
        .collect::<Vec<_>>();
    let pass_ids = vec!["import-inline".to_string(), "print-css".to_string()];
    let context = OmenaQueryTransformExecutionContextV0::default();
    let resolution_inputs = OmenaQueryStyleResolutionInputsV0::default();
    let run = |emission_path| {
        run_omena_query_bundle_with_semantic_inputs_and_options(
            OmenaQueryBundlePlanInputV0 {
                target_style_path: &fixture.entry_path,
                style_sources: &style_sources,
                source_map_sources: &style_sources,
                requested_pass_ids: &pass_ids,
                context: &context,
                resolution_inputs: &resolution_inputs,
                asset_rewrites: Vec::new(),
                bundle_entry_style_paths: &[],
            },
            &[],
            &OmenaQueryConsumerBuildOptionsV0 {
                bundle_emission_path: emission_path,
                ..OmenaQueryConsumerBuildOptionsV0::default()
            },
        )
    };
    let legacy = run(OmenaQueryBundleEmissionPathV0::ImportInlineLegacy)?;
    let linked = run(OmenaQueryBundleEmissionPathV0::LinkedOrder)?;
    let legacy_css = legacy.artifact.output_css;
    let mut linked_css = linked.artifact.output_css;
    match perturbation {
        LinkedEmissionByteDifferentialPerturbationV0::None => {}
        LinkedEmissionByteDifferentialPerturbationV0::AddUnexpectedRule => {
            linked_css.push_str("\n.injected-unexpected-rule { color: magenta; }");
        }
        LinkedEmissionByteDifferentialPerturbationV0::CollapseToLegacyBytes => {
            linked_css.clone_from(&legacy_css);
        }
    }

    let marker_names = fixture
        .modules
        .iter()
        .flat_map(|module| module.marker_names.iter().cloned())
        .collect::<BTreeSet<_>>();
    let linker_modules = fixture
        .modules
        .iter()
        .map(|module| {
            TransformBundleModuleInputV0::new(
                module.path.clone(),
                module.source.clone(),
                module.dialect,
            )
        })
        .collect::<Vec<_>>();
    let linked_order = link_omena_transform_bundle_modules(
        std::slice::from_ref(&fixture.entry_path),
        &linker_modules,
    )
    .map_err(|error| format!("fixture {} could not be linked: {error:?}", fixture.id))?;
    let authoritative_marker_order = linked_order
        .global_rule_order
        .rules
        .iter()
        .filter(|rule| marker_names.contains(&rule.selector_name))
        .map(|rule| rule.selector_name.clone())
        .collect::<Vec<_>>();
    let legacy_marker_order = output_marker_order_v0(&legacy_css, &marker_names);
    let linked_marker_order = output_marker_order_v0(&linked_css, &marker_names);
    let linked_modules_emitted_once = marker_names.iter().all(|marker| {
        linked_marker_order
            .iter()
            .filter(|candidate| *candidate == marker)
            .count()
            == 1
    });
    let semantic =
        compare_omena_query_transform_css_semantics_v0(&legacy_css, &linked_css, StyleDialect::Css);
    let byte_equal = legacy_css == linked_css;
    let reasons = derive_difference_reasons_v0(
        fixture,
        &linked_order,
        &legacy_css,
        &linked_css,
        &authoritative_marker_order,
        &legacy_marker_order,
        &linked_marker_order,
    );
    let difference_class = if byte_equal {
        LinkedEmissionByteDifferenceClassV0::Equivalent
    } else if semantic.preserved
        && linked_modules_emitted_once
        && linked_marker_order == authoritative_marker_order
        && !reasons.is_empty()
    {
        LinkedEmissionByteDifferenceClassV0::Expected
    } else {
        LinkedEmissionByteDifferenceClassV0::Unexpected
    };

    let case = LinkedEmissionByteDifferentialCaseV0 {
        fixture_id: fixture.id.clone(),
        module_count: fixture.modules.len(),
        legacy_emission_path: legacy.artifact.emission_path.as_wire_label(),
        linked_emission_path: linked.artifact.emission_path.as_wire_label(),
        legacy_sha256: sha256_hex_v0(&legacy_css),
        linked_sha256: sha256_hex_v0(&linked_css),
        legacy_byte_len: legacy_css.len(),
        linked_byte_len: linked_css.len(),
        byte_equal,
        semantic_preserved: semantic.preserved,
        semantic_mismatch_count: semantic.mismatch_count,
        authoritative_marker_order,
        legacy_marker_order,
        linked_marker_order,
        linked_modules_emitted_once,
        difference_class,
        reasons,
    };
    Ok(LinkedEmissionFixtureAnalysisV0 {
        case,
        linked_order,
        legacy_css,
        linked_css,
    })
}

fn populated_fact_categories_v0(facts: &ParsedStyleFacts) -> Vec<&'static str> {
    let mut categories = Vec::new();
    if !facts.selectors.is_empty() {
        categories.push("selectors");
    }
    if !facts.variables.is_empty() {
        categories.push("variables");
    }
    if !facts.sass_symbols.is_empty() {
        categories.push("sassSymbols");
    }
    if !facts.sass_includes.is_empty() {
        categories.push("sassIncludes");
    }
    if !facts.sass_module_edges.is_empty() {
        categories.push("sassModuleEdges");
    }
    if !facts.sass_placeholder_definitions.is_empty() {
        categories.push("sassPlaceholderDefinitions");
    }
    if !facts.extend_targets.is_empty() {
        categories.push("extendTargets");
    }
    if !facts.animations.is_empty() {
        categories.push("animations");
    }
    if !facts.css_module_values.is_empty() {
        categories.push("cssModuleValues");
    }
    if !facts.css_module_value_import_edges.is_empty() {
        categories.push("cssModuleValueImportEdges");
    }
    if !facts.css_module_value_definition_edges.is_empty() {
        categories.push("cssModuleValueDefinitionEdges");
    }
    if !facts.css_module_composes.is_empty() {
        categories.push("cssModuleComposes");
    }
    if !facts.css_module_composes_edges.is_empty() {
        categories.push("cssModuleComposesEdges");
    }
    if !facts.icss.is_empty() {
        categories.push("icss");
    }
    if !facts.icss_import_edges.is_empty() {
        categories.push("icssImportEdges");
    }
    if !facts.icss_export_edges.is_empty() {
        categories.push("icssExportEdges");
    }
    if !facts.at_rules.is_empty() {
        categories.push("atRules");
    }
    categories
}

fn summarize_linked_emission_placement_witness_v0(
    definition: &LinkedEmissionPlacementWitnessDefinitionV0,
) -> Result<LinkedEmissionPlacementWitnessV0, String> {
    let analysis = analyze_linked_emission_fixture_v0(
        &definition.fixture,
        LinkedEmissionByteDifferentialPerturbationV0::None,
    )?;
    let selectorless_module_paths = definition
        .selectorless_module_paths
        .iter()
        .map(|path| (*path).to_string())
        .collect::<Vec<_>>();
    if selectorless_module_paths.is_empty() {
        return Err(format!(
            "{} has no independently designated selector-less module",
            definition.fixture.id
        ));
    }
    for path in &selectorless_module_paths {
        if !definition
            .fixture
            .modules
            .iter()
            .any(|module| module.path == *path)
        {
            return Err(format!(
                "{} designates an absent selector-less module: {path}",
                definition.fixture.id
            ));
        }
    }
    let emission_plan_entry_count = analysis
        .linked_order
        .global_rule_order
        .rules
        .iter()
        .filter(|rule| {
            selectorless_module_paths
                .iter()
                .any(|path| path == rule.module_instance.module().as_str())
        })
        .count();
    let legacy_winner =
        linked_emission_winner_v0(definition.fixture.id.as_str(), analysis.legacy_css.as_str())?;
    let linked_winner =
        linked_emission_winner_v0(definition.fixture.id.as_str(), analysis.linked_css.as_str())?;

    Ok(LinkedEmissionPlacementWitnessV0 {
        witness_id: definition.fixture.id.clone(),
        selectorless_module_paths,
        emission_plan_entry_count,
        output_bytes_differ: !analysis.case.byte_equal,
        marker_orders_agree: analysis.case.legacy_marker_order == analysis.case.linked_marker_order,
        linked_marker_order_matches_authority: analysis.case.linked_marker_order
            == analysis.case.authoritative_marker_order,
        semantic_difference_observed: analysis.case.semantic_mismatch_count > 0,
        difference_reason_observed: !analysis.case.reasons.is_empty(),
        import_graph_winner: definition.import_graph_winner.to_string(),
        legacy_winner,
        linked_winner,
    })
}

fn linked_emission_winner_v0(witness_id: &str, source: &str) -> Result<String, String> {
    let compact = remove_ascii_whitespace_v0(source);
    if witness_id == "cascade-layer-declaration-order" {
        let statement = compact
            .find("@layerbase,theme;")
            .ok_or_else(|| format!("{witness_id} output has no layer-order statement"))?;
        let theme = compact
            .find("@layertheme{")
            .ok_or_else(|| format!("{witness_id} output has no theme layer block"))?;
        let base = compact
            .find("@layerbase{")
            .ok_or_else(|| format!("{witness_id} output has no base layer block"))?;
        return Ok(if statement < theme.min(base) || base < theme {
            "blue"
        } else {
            "orange"
        }
        .to_string());
    }

    let red = compact
        .rfind("color:red")
        .ok_or_else(|| format!("{witness_id} output has no red declaration"))?;
    let green = compact
        .rfind("color:green")
        .ok_or_else(|| format!("{witness_id} output has no green declaration"))?;
    Ok(if red > green { "red" } else { "green" }.to_string())
}

fn derive_difference_reasons_v0(
    fixture: &LinkedEmissionFixtureV0,
    linked_order: &omena_bundler::LinkedStylesheetV0,
    legacy_css: &str,
    linked_css: &str,
    authoritative_marker_order: &[String],
    legacy_marker_order: &[String],
    linked_marker_order: &[String],
) -> Vec<LinkedEmissionByteDifferenceReasonV0> {
    let mut reasons = BTreeSet::new();
    if legacy_marker_order != linked_marker_order
        && linked_marker_order == authoritative_marker_order
    {
        reasons.insert(LinkedEmissionByteDifferenceReasonV0::GlobalModuleOrder);
    }

    let marker_sets_by_module = fixture
        .modules
        .iter()
        .map(|module| {
            (
                module.path.as_str(),
                module.marker_names.iter().cloned().collect::<BTreeSet<_>>(),
            )
        })
        .collect::<BTreeMap<_, _>>();
    if let Some(entry_markers) = marker_sets_by_module.get(fixture.entry_path.as_str())
        && sequence_splits_marker_group_v0(legacy_marker_order, entry_markers)
        && !sequence_splits_marker_group_v0(linked_marker_order, entry_markers)
    {
        reasons.insert(LinkedEmissionByteDifferenceReasonV0::EntryInterleaveCollapse);
    }
    if marker_sets_by_module.values().any(|markers| {
        sequence_splits_marker_group_v0(legacy_marker_order, markers)
            && !sequence_splits_marker_group_v0(linked_marker_order, markers)
    }) {
        reasons.insert(LinkedEmissionByteDifferenceReasonV0::PerModuleGrouping);
    }

    let mut inbound_counts = BTreeMap::new();
    for fact in &linked_order.emission_plan.dependency_facts {
        *inbound_counts
            .entry(fact.to_module.module().as_str())
            .or_insert(0usize) += 1;
    }
    if inbound_counts.iter().any(|(path, count)| {
        *count > 1
            && marker_sets_by_module.get(path).is_some_and(|markers| {
                markers.iter().any(|marker| {
                    legacy_marker_order
                        .iter()
                        .filter(|candidate| *candidate == marker)
                        .count()
                        > linked_marker_order
                            .iter()
                            .filter(|candidate| *candidate == marker)
                            .count()
                })
            })
    }) {
        reasons.insert(LinkedEmissionByteDifferenceReasonV0::SharedImportSingleEmission);
    }
    if remove_ascii_whitespace_v0(legacy_css) == remove_ascii_whitespace_v0(linked_css) {
        reasons.insert(LinkedEmissionByteDifferenceReasonV0::FormattingNormalization);
    }
    reasons.into_iter().collect()
}

fn sequence_splits_marker_group_v0(sequence: &[String], group: &BTreeSet<String>) -> bool {
    let positions = sequence
        .iter()
        .enumerate()
        .filter_map(|(index, marker)| group.contains(marker).then_some(index))
        .collect::<Vec<_>>();
    let (Some(first), Some(last)) = (positions.first().copied(), positions.last().copied()) else {
        return false;
    };
    sequence[first..=last]
        .iter()
        .any(|marker| !group.contains(marker))
}

fn output_marker_order_v0(source: &str, marker_names: &BTreeSet<String>) -> Vec<String> {
    summarize_omena_parser_style_facts(source, StyleDialect::Css)
        .class_selector_names
        .into_iter()
        .filter(|name| marker_names.contains(name))
        .collect()
}

fn remove_ascii_whitespace_v0(source: &str) -> String {
    source
        .chars()
        .filter(|character| !character.is_ascii_whitespace())
        .collect()
}

fn sha256_hex_v0(source: &str) -> String {
    let digest = Sha256::digest(source.as_bytes());
    digest.iter().map(|byte| format!("{byte:02x}")).collect()
}

fn linked_emission_placement_witness_definitions_v0()
-> Vec<LinkedEmissionPlacementWitnessDefinitionV0> {
    vec![
        LinkedEmissionPlacementWitnessDefinitionV0 {
            fixture: LinkedEmissionFixtureV0 {
                id: "element-selector-winner".to_string(),
                entry_path: "linked-order-witness/element/app.css".to_string(),
                shape_classes: Vec::new(),
                modules: vec![
                    LinkedEmissionFixtureModuleV0 {
                        path: "linked-order-witness/element/app.css".to_string(),
                        source: "@import \"./reset.css\"; div { color: red; }".to_string(),
                        dialect: StyleDialect::Css,
                        marker_names: Vec::new(),
                    },
                    LinkedEmissionFixtureModuleV0 {
                        path: "linked-order-witness/element/reset.css".to_string(),
                        source: "div { color: green; }".to_string(),
                        dialect: StyleDialect::Css,
                        marker_names: Vec::new(),
                    },
                ],
            },
            selectorless_module_paths: &[
                "linked-order-witness/element/app.css",
                "linked-order-witness/element/reset.css",
            ],
            import_graph_winner: "red",
        },
        LinkedEmissionPlacementWitnessDefinitionV0 {
            fixture: LinkedEmissionFixtureV0 {
                id: "element-selector-after-rule-bearing-module".to_string(),
                entry_path: "linked-order-witness/mixed/app.css".to_string(),
                shape_classes: Vec::new(),
                modules: vec![
                    LinkedEmissionFixtureModuleV0 {
                        path: "linked-order-witness/mixed/app.css".to_string(),
                        source:
                            "@import \"./reset.css\"; .card { padding: 1px; } div { color: red; }"
                                .to_string(),
                        dialect: StyleDialect::Css,
                        marker_names: vec!["card".to_string()],
                    },
                    LinkedEmissionFixtureModuleV0 {
                        path: "linked-order-witness/mixed/reset.css".to_string(),
                        source: "div { color: green; }".to_string(),
                        dialect: StyleDialect::Css,
                        marker_names: Vec::new(),
                    },
                ],
            },
            selectorless_module_paths: &["linked-order-witness/mixed/reset.css"],
            import_graph_winner: "red",
        },
        LinkedEmissionPlacementWitnessDefinitionV0 {
            fixture: LinkedEmissionFixtureV0 {
                id: "path-name-independence".to_string(),
                entry_path: "linked-order-witness/names/zzz-app.css".to_string(),
                shape_classes: Vec::new(),
                modules: vec![
                    LinkedEmissionFixtureModuleV0 {
                        path: "linked-order-witness/names/zzz-app.css".to_string(),
                        source: "@import \"./aaa-reset.css\"; div { color: red; }".to_string(),
                        dialect: StyleDialect::Css,
                        marker_names: Vec::new(),
                    },
                    LinkedEmissionFixtureModuleV0 {
                        path: "linked-order-witness/names/aaa-reset.css".to_string(),
                        source: "div { color: green; }".to_string(),
                        dialect: StyleDialect::Css,
                        marker_names: Vec::new(),
                    },
                ],
            },
            selectorless_module_paths: &[
                "linked-order-witness/names/zzz-app.css",
                "linked-order-witness/names/aaa-reset.css",
            ],
            import_graph_winner: "red",
        },
        LinkedEmissionPlacementWitnessDefinitionV0 {
            fixture: LinkedEmissionFixtureV0 {
                id: "cascade-layer-declaration-order".to_string(),
                entry_path: "linked-order-witness/layers/app.css".to_string(),
                shape_classes: Vec::new(),
                modules: vec![
                    LinkedEmissionFixtureModuleV0 {
                        path: "linked-order-witness/layers/app.css".to_string(),
                        source: "@import \"./layers.css\"; @layer theme { .card { color: blue; } } @layer base { .card { color: orange; } }".to_string(),
                        dialect: StyleDialect::Css,
                        marker_names: Vec::new(),
                    },
                    LinkedEmissionFixtureModuleV0 {
                        path: "linked-order-witness/layers/layers.css".to_string(),
                        source: "@layer base, theme;".to_string(),
                        dialect: StyleDialect::Css,
                        marker_names: Vec::new(),
                    },
                ],
            },
            selectorless_module_paths: &["linked-order-witness/layers/layers.css"],
            import_graph_winner: "blue",
        },
    ]
}

fn linked_emission_fixtures_v0() -> Vec<LinkedEmissionFixtureV0> {
    let mut fixtures = vec![
        dialect_fixture_v0("css", StyleDialect::Css, "@import"),
        dialect_fixture_v0("scss", StyleDialect::Scss, "@import"),
        dialect_fixture_v0("less", StyleDialect::Less, "@import"),
        shared_import_fixture_v0(),
        selectorless_module_fixture_v0(
            "element-only-reset-module",
            "element-only-reset",
            "reset.css",
            "div { margin: 0; color: green; }",
        ),
        selectorless_module_fixture_v0(
            "bare-layer-statement-module",
            "bare-layer-statement",
            "layers.css",
            "@layer reset;",
        ),
        selectorless_module_fixture_v0(
            "font-face-only-module",
            "font-face-only",
            "fonts.css",
            "@font-face { font-family: \"OmenaFixture\"; src: url(\"./font.woff2\"); }",
        ),
    ];
    if let Some(corpus_fixture) = product_corpus_fixture_v0() {
        fixtures.push(corpus_fixture);
    }
    fixtures
}

fn selectorless_module_fixture_v0(
    fixture_id: &str,
    shape_class: &'static str,
    imported_file_name: &str,
    imported_source: &str,
) -> LinkedEmissionFixtureV0 {
    let root = format!("linked-byte/{fixture_id}");
    let entry_path = format!("{root}/app.css");
    let entry_marker = format!("linked-{fixture_id}-entry");
    LinkedEmissionFixtureV0 {
        id: fixture_id.to_string(),
        entry_path: entry_path.clone(),
        shape_classes: vec![shape_class],
        modules: vec![
            LinkedEmissionFixtureModuleV0 {
                path: entry_path,
                source: format!(
                    "@import \"./{imported_file_name}\"; .{entry_marker} {{ color: red; }}"
                ),
                dialect: StyleDialect::Css,
                marker_names: vec![entry_marker],
            },
            LinkedEmissionFixtureModuleV0 {
                path: format!("{root}/{imported_file_name}"),
                source: imported_source.to_string(),
                dialect: StyleDialect::Css,
                marker_names: Vec::new(),
            },
        ],
    }
}

fn dialect_fixture_v0(
    extension: &str,
    dialect: StyleDialect,
    import_keyword: &str,
) -> LinkedEmissionFixtureV0 {
    let root = format!("linked-byte/{extension}");
    let entry_path = format!("{root}/app.{extension}");
    let before = format!("linked-{extension}-entry-before");
    let after = format!("linked-{extension}-entry-after");
    let a_marker = format!("linked-{extension}-a");
    let z_marker = format!("linked-{extension}-z");
    LinkedEmissionFixtureV0 {
        id: format!("dialect-{extension}-import-order"),
        entry_path: entry_path.clone(),
        shape_classes: vec![match dialect {
            StyleDialect::Css => "css-import-order",
            StyleDialect::Scss | StyleDialect::Sass => "scss-import-order",
            StyleDialect::Less => "less-import-order",
        }],
        modules: vec![
            LinkedEmissionFixtureModuleV0 {
                path: entry_path,
                source: format!(
                    ".{before} {{ color: red; }} {import_keyword} \"./z.{extension}\"; {import_keyword} \"./a.{extension}\"; .{after} {{ color: orange; }}"
                ),
                dialect,
                marker_names: vec![before, after],
            },
            LinkedEmissionFixtureModuleV0 {
                path: format!("{root}/a.{extension}"),
                source: format!(".{a_marker} {{ color: blue; }}"),
                dialect,
                marker_names: vec![a_marker],
            },
            LinkedEmissionFixtureModuleV0 {
                path: format!("{root}/z.{extension}"),
                source: format!(".{z_marker} {{ color: green; }}"),
                dialect,
                marker_names: vec![z_marker],
            },
        ],
    }
}

fn shared_import_fixture_v0() -> LinkedEmissionFixtureV0 {
    let root = "linked-byte/shared";
    LinkedEmissionFixtureV0 {
        id: "shared-import-diamond".to_string(),
        entry_path: format!("{root}/app.css"),
        shape_classes: vec!["shared-import-diamond"],
        modules: vec![
            LinkedEmissionFixtureModuleV0 {
                path: format!("{root}/app.css"),
                source: ".linked-shared-entry-before { color: red; } @import \"./left.css\"; @import \"./right.css\"; .linked-shared-entry-after { color: orange; }".to_string(),
                dialect: StyleDialect::Css,
                marker_names: vec![
                    "linked-shared-entry-before".to_string(),
                    "linked-shared-entry-after".to_string(),
                ],
            },
            LinkedEmissionFixtureModuleV0 {
                path: format!("{root}/left.css"),
                source: ".linked-shared-left-before { color: blue; } @import \"./tokens.css\"; .linked-shared-left-after { color: navy; }".to_string(),
                dialect: StyleDialect::Css,
                marker_names: vec![
                    "linked-shared-left-before".to_string(),
                    "linked-shared-left-after".to_string(),
                ],
            },
            LinkedEmissionFixtureModuleV0 {
                path: format!("{root}/right.css"),
                source: ".linked-shared-right { color: teal; } @import \"./tokens.css\";".to_string(),
                dialect: StyleDialect::Css,
                marker_names: vec!["linked-shared-right".to_string()],
            },
            LinkedEmissionFixtureModuleV0 {
                path: format!("{root}/tokens.css"),
                source: ".linked-shared-token { color: purple; }".to_string(),
                dialect: StyleDialect::Css,
                marker_names: vec!["linked-shared-token".to_string()],
            },
        ],
    }
}

fn product_corpus_fixture_v0() -> Option<LinkedEmissionFixtureV0> {
    let samples = bundler_productization_corpus()
        .into_iter()
        .filter(|sample| sample.dialect == StyleDialect::Css)
        .take(2)
        .collect::<Vec<_>>();
    if samples.len() < 2 {
        return None;
    }
    let root = "linked-byte/product-corpus";
    let entry_path = format!("{root}/app.css");
    let mut modules = vec![LinkedEmissionFixtureModuleV0 {
        path: entry_path.clone(),
        source: format!(
            ".linked-corpus-entry-before {{ color: red; }} @import \"./{}\"; @import \"./{}\"; .linked-corpus-entry-after {{ color: orange; }}",
            samples[0].path, samples[1].path
        ),
        dialect: StyleDialect::Css,
        marker_names: vec![
            "linked-corpus-entry-before".to_string(),
            "linked-corpus-entry-after".to_string(),
        ],
    }];
    for (index, sample) in samples.into_iter().enumerate() {
        let marker = format!("linked-corpus-module-{index}");
        modules.push(LinkedEmissionFixtureModuleV0 {
            path: format!("{root}/{}", sample.path),
            source: format!(
                ".{marker} {{ --omena-corpus-marker: {index}; }}\n{}",
                sample.source
            ),
            dialect: sample.dialect,
            marker_names: vec![marker],
        });
    }
    Some(LinkedEmissionFixtureV0 {
        id: "bundler-productization-corpus".to_string(),
        entry_path,
        shape_classes: vec!["large-product-corpus"],
        modules,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn linked_emission_differential_is_non_vacuous_and_authority_bound() -> Result<(), String> {
        let report = summarize_linked_emission_byte_differential_v0(
            LinkedEmissionByteDifferentialPerturbationV0::None,
        )?;

        assert!(report.fixture_count >= 3);
        assert!(report.total_divergence_count > 0);
        assert!(report.cases.iter().all(|case| {
            case.legacy_emission_path == "importInlineLegacy"
                && case.linked_emission_path == "linkedOrder"
                && case.linked_modules_emitted_once
                && case.linked_marker_order == case.authoritative_marker_order
        }));
        Ok(())
    }

    #[test]
    fn coverage_census_is_derived_from_an_independent_shape_population() -> Result<(), String> {
        let envelope = summarize_linked_emission_byte_differential_envelope_v0(
            LinkedEmissionByteDifferentialPerturbationV0::None,
        )?;
        let census = envelope.census;

        assert_eq!(census.fixture_count, envelope.report.fixture_count);
        assert_eq!(
            census.covered_shape_count + census.not_covered_shape_count,
            census.population_count
        );
        assert!(census.population_count > census.fixture_count);
        assert!(!census.not_covered.is_empty());
        assert!(!census.full_corpus_coverage);
        assert_eq!(census.coverage_scope, "boundedMultiModuleFixtures");
        assert_eq!(
            census.marker_observable_module_count + census.blind_spot_module_count,
            census.module_count
        );
        assert_eq!(
            census.fixture_observability.len(),
            envelope.report.fixture_count
        );
        Ok(())
    }

    #[test]
    fn selectorless_module_shapes_are_marker_blind_unexpected_divergences() -> Result<(), String> {
        let envelope = summarize_linked_emission_byte_differential_envelope_v0(
            LinkedEmissionByteDifferentialPerturbationV0::None,
        )?;
        let expected_fixture_ids = BTreeSet::from([
            "bare-layer-statement-module",
            "element-only-reset-module",
            "font-face-only-module",
        ]);
        let unexpected_fixture_ids = envelope
            .report
            .cases
            .iter()
            .filter(|case| case.difference_class == LinkedEmissionByteDifferenceClassV0::Unexpected)
            .map(|case| case.fixture_id.as_str())
            .collect::<BTreeSet<_>>();
        let blind_spot_fixture_ids = envelope
            .census
            .blind_spots
            .iter()
            .map(|blind_spot| blind_spot.fixture_id.as_str())
            .collect::<BTreeSet<_>>();

        assert_eq!(unexpected_fixture_ids, expected_fixture_ids);
        assert_eq!(blind_spot_fixture_ids, expected_fixture_ids);
        assert!(envelope.census.blind_spots.iter().all(|blind_spot| {
            blind_spot.emission_plan_entry_count == 0
                && blind_spot.output_bytes_differ
                && blind_spot.marker_orders_agree
                && blind_spot.linked_marker_order_matches_authority
                && !blind_spot.semantic_difference_observed
                && !blind_spot.difference_reason_observed
        }));
        assert_eq!(
            envelope
                .census
                .blind_spots
                .iter()
                .map(|blind_spot| {
                    (
                        blind_spot.fixture_id.as_str(),
                        blind_spot.fact_categories.as_slice(),
                    )
                })
                .collect::<BTreeMap<_, _>>(),
            BTreeMap::from([
                ("bare-layer-statement-module", ["atRules"].as_slice()),
                ("element-only-reset-module", [].as_slice()),
                ("font-face-only-module", ["atRules"].as_slice()),
            ])
        );
        assert!(envelope.census.not_covered.iter().all(|entry| {
            !matches!(
                entry.shape_class.as_str(),
                "bare-layer-statement" | "element-only-reset" | "font-face-only"
            )
        }));
        Ok(())
    }

    #[test]
    fn placement_witnesses_expose_selector_only_order_blindness() -> Result<(), String> {
        let envelope = summarize_linked_emission_byte_differential_envelope_v0(
            LinkedEmissionByteDifferentialPerturbationV0::None,
        )?;
        let witnesses = envelope
            .census
            .placement_witnesses
            .iter()
            .map(|witness| (witness.witness_id.as_str(), witness))
            .collect::<BTreeMap<_, _>>();

        assert_eq!(witnesses.len(), 4);
        assert!(witnesses.values().all(|witness| {
            !witness.selectorless_module_paths.is_empty()
                && witness.emission_plan_entry_count == 0
                && witness.output_bytes_differ
                && witness.marker_orders_agree
                && witness.linked_marker_order_matches_authority
        }));
        assert_eq!(
            witnesses
                .iter()
                .map(|(id, witness)| {
                    (
                        *id,
                        (
                            witness.import_graph_winner.as_str(),
                            witness.legacy_winner.as_str(),
                            witness.linked_winner.as_str(),
                            witness.semantic_difference_observed,
                            witness.difference_reason_observed,
                        ),
                    )
                })
                .collect::<BTreeMap<_, _>>(),
            BTreeMap::from([
                (
                    "cascade-layer-declaration-order",
                    ("blue", "blue", "orange", false, false)
                ),
                (
                    "element-selector-after-rule-bearing-module",
                    ("red", "red", "green", true, false)
                ),
                (
                    "element-selector-winner",
                    ("red", "red", "green", true, false)
                ),
                ("path-name-independence", ("red", "red", "red", false, true)),
            ])
        );
        Ok(())
    }

    #[test]
    fn unexpected_semantic_change_is_not_force_classified() -> Result<(), String> {
        let report = summarize_linked_emission_byte_differential_v0(
            LinkedEmissionByteDifferentialPerturbationV0::AddUnexpectedRule,
        )?;

        assert!(report.unexpected_divergence_count > 0);
        Ok(())
    }

    #[test]
    fn collapsed_arms_are_detectably_vacuous() -> Result<(), String> {
        let report = summarize_linked_emission_byte_differential_v0(
            LinkedEmissionByteDifferentialPerturbationV0::CollapseToLegacyBytes,
        )?;

        assert_eq!(report.total_divergence_count, 0);
        Ok(())
    }
}

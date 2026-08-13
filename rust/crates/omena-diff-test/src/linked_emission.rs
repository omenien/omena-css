use std::collections::{BTreeMap, BTreeSet};

use omena_benchmarks::bundler_productization_corpus;
use omena_bundler::{
    BundleResolutionAuthorityV0, EmissionItemKindV0, EmissionOrderingPolicyV0,
    LinkedStylesheetWithEmissionItemsV0, LinkerInputV0, TransformBundleLinkOptionsV0,
    TransformBundleModuleInputV0,
    link_omena_transform_bundle_projection_with_emission_items_and_resolved_dependencies_and_options,
    project_omena_transform_bundle_linker_and_emission_items,
};
use omena_parser::{
    ParsedEmissionSelectorFactsV0, ParsedSelectorFactKind, ParsedStyleFacts, StyleDialect,
    TypedCstNode, collect_style_fact_collection, facts_from_cst, parse, summarize_omena_parser_lex,
    summarize_omena_parser_style_facts,
};
use omena_query::{
    ClassExpressionInputV2, EngineInputV2, OmenaQueryBundleEmissionPathV0,
    OmenaQueryBundlePlanInputV0, OmenaQueryConsumerBuildOptionsV0,
    OmenaQueryEngineInputModuleReachabilityV0, OmenaQueryModuleReachabilityAttributionReportV0,
    OmenaQueryStyleResolutionInputsV0, OmenaQueryStyleSourceInputV0,
    OmenaQueryTransformExecutionContextV0, PositionV2, RangeV2, SourceAnalysisInputV2,
    SourceDocumentV2, StringTypeFactsV2, StyleAnalysisInputV2, StyleDocumentV2, StyleSelectorV2,
    TypeFactEntryV2, compare_omena_query_transform_css_semantics_v0,
    derive_omena_query_module_reachability_from_engine_input,
    run_omena_query_bundle_with_execution_scope_evidence_and_options,
    run_omena_query_bundle_with_module_reachability_and_execution_scope_evidence_and_options,
    summarize_omena_query_transform_context_from_sources_with_resolution_inputs,
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
    DropReachableCrossModuleDeclaration,
    DropComposedDeclaration,
    DropLiveDeclaration,
    AddUnclaimedLinkedToken,
    DropComposesReachability,
    BreakEnginePathEquivalence,
    AddUnattributedReachabilityReference,
    FlipAuthoredLivenessExpectation,
    DropFixture,
    MisattributeLinkedRule,
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
    ImportGraphModulePlacement,
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
    pub authoritative_module_order: Vec<String>,
    pub linked_output_module_order: Vec<String>,
    pub linked_output_module_order_matches_authority: bool,
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
    pub resolution_authority_population_scope: &'static str,
    pub resolved_resolution_count: usize,
    pub legacy_path_inferred_resolution_count: usize,
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
/// One emitted CSS Modules token produced by declarations from multiple modules.
pub struct LinkedEmissionModuleTokenCollisionV0 {
    pub fixture_id: String,
    pub emitted_token: String,
    pub module_paths: Vec<String>,
    pub original_names: Vec<String>,
    pub observed_emission_paths: Vec<&'static str>,
    pub path_scope: LinkedEmissionModuleTokenCollisionPathScopeV0,
    pub reason: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize)]
#[serde(rename_all = "camelCase")]
/// Emission paths on which one authored collision is expected.
pub enum LinkedEmissionModuleTokenCollisionPathScopeV0 {
    BothPaths,
    ImportInlineLegacyOnly,
    LinkedOrderOnly,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
/// A live class declaration that the path-specific token model could not find in output.
pub struct LinkedEmissionUnmodeledDeclarationV0 {
    pub fixture_id: String,
    pub emission_path: &'static str,
    pub module_path: String,
    pub original_name: String,
    pub modeled_token: String,
}

#[derive(Debug, Default)]
struct LinkedEmissionModuleTokenCollisionSummaryV0 {
    collisions: Vec<LinkedEmissionModuleTokenCollisionV0>,
    unmodeled_declarations: Vec<LinkedEmissionUnmodeledDeclarationV0>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
/// One build module observed by the linked-emission liveness instrument.
pub struct LinkedEmissionLiveDeclarationModuleV0 {
    pub module_path: String,
    pub declared_class_names: Vec<String>,
    pub live_declared_class_names: Vec<String>,
    pub authored_liveness_expectation_count: usize,
    pub authored_live_class_names: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
/// Per-fixture domain and source preconditions for linked declaration retention.
pub struct LinkedEmissionLiveDeclarationFixtureV0 {
    pub fixture_id: String,
    pub reachability_reference_count: usize,
    pub engine_input_style_source_count: usize,
    pub engine_input_path_form: &'static str,
    pub unmatched_target_style_path_count: usize,
    pub composes_resolution_count: usize,
    pub declaration_preserving_pass_ids: Vec<String>,
    pub modules: Vec<LinkedEmissionLiveDeclarationModuleV0>,
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
    pub unknown_structural_selector_count: usize,
    pub unknown_at_rule_count: usize,
    pub module_token_collision_scope: &'static str,
    pub module_token_collision_count: usize,
    pub module_token_collisions: Vec<LinkedEmissionModuleTokenCollisionV0>,
    pub ordinal_skew_shared_model_collision_count: usize,
    pub ordinal_skew_path_split_collision_count: usize,
    pub token_model_by_emission_path: BTreeMap<&'static str, &'static str>,
    pub unmodeled_declarations: Vec<LinkedEmissionUnmodeledDeclarationV0>,
    pub reachability_input_fixture_ids: Vec<String>,
    pub in_domain_fixture_ids: Vec<String>,
    pub live_declaration_fixtures: Vec<LinkedEmissionLiveDeclarationFixtureV0>,
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
    order_probe: String,
}

#[derive(Debug, Clone)]
struct LinkedEmissionReachabilityReferenceV0 {
    id: &'static str,
    module_index: usize,
    class_name: &'static str,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum LinkedEmissionLivenessVerdictV0 {
    Live,
    Dead,
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum LinkedEmissionLivenessReasonV0 {
    DirectReference,
    ComposesFrom(String),
    GlobalEscape,
    Unreferenced,
    ReferrerShakenAway,
    CombinatorCompanion,
    AtRuleNested,
    WorkspaceOnlyReferrer,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct LinkedEmissionLivenessExpectationV0 {
    module_path: String,
    class_name: &'static str,
    verdict: LinkedEmissionLivenessVerdictV0,
    reason: LinkedEmissionLivenessReasonV0,
}

struct LinkedEmissionDifferenceObservationV0<'a> {
    legacy_css: &'a str,
    linked_css: &'a str,
    authoritative_marker_order: &'a [String],
    legacy_marker_order: &'a [String],
    linked_marker_order: &'a [String],
    linked_output_module_order_matches_authority: bool,
}

#[derive(Debug, Clone)]
struct LinkedEmissionFixtureV0 {
    id: String,
    entry_path: String,
    shape_classes: Vec<&'static str>,
    modules: Vec<LinkedEmissionFixtureModuleV0>,
    workspace_only_modules: Vec<LinkedEmissionFixtureModuleV0>,
    reachability_references: Vec<LinkedEmissionReachabilityReferenceV0>,
    liveness_expectations: Vec<LinkedEmissionLivenessExpectationV0>,
}

#[derive(Debug)]
struct LinkedEmissionFixtureAnalysisV0 {
    case: LinkedEmissionByteDifferentialCaseV0,
    linked_order: LinkedStylesheetWithEmissionItemsV0,
    legacy_css: String,
    linked_css: String,
    class_name_rewrites_by_module: BTreeMap<String, BTreeMap<String, String>>,
    live_declared_names_by_module: BTreeMap<String, BTreeSet<String>>,
    engine_input_style_source_count: usize,
    engine_input_path_form: &'static str,
    unmatched_target_style_path_count: usize,
    composes_resolution_count: usize,
    declaration_preserving_pass_ids: Vec<String>,
    collision_plan_owner_override: bool,
}

#[derive(Debug)]
struct LinkedEmissionFixtureReachabilityV0 {
    report: OmenaQueryEngineInputModuleReachabilityV0,
    engine_input_style_source_count: usize,
    engine_input_path_form: &'static str,
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
        shape_class: "module-qualified-reachability",
        reentry: "retain a linked fixture with independently attributed module reachability",
    },
    LinkedEmissionCoverageShapeDefinitionV0 {
        shape_class: "entry-ordinal-skew",
        reentry: "retain a fixture whose entry and dependency assign one shared class different rewrite ordinals",
    },
    LinkedEmissionCoverageShapeDefinitionV0 {
        shape_class: "at-rule-nested-liveness",
        reentry: "retain a shaken module with one live and one dead class nested in an at-rule",
    },
    LinkedEmissionCoverageShapeDefinitionV0 {
        shape_class: "two-hop-composes-liveness",
        reentry: "retain a three-module composes chain whose terminal class is live",
    },
    LinkedEmissionCoverageShapeDefinitionV0 {
        shape_class: "package-path-composes-liveness",
        reentry: "retain a composes edge resolved through a package-shaped module path",
    },
    LinkedEmissionCoverageShapeDefinitionV0 {
        shape_class: "global-at-rule-liveness",
        reentry: "retain global escapes and live classes nested under media, supports, and layer rules",
    },
    LinkedEmissionCoverageShapeDefinitionV0 {
        shape_class: "combinator-companion-liveness",
        reentry: "retain compound, child, and descendant selector companions beside a live class",
    },
    LinkedEmissionCoverageShapeDefinitionV0 {
        shape_class: "workspace-only-referrer-liveness",
        reentry: "retain a build-module class reached only through a workspace-only composes referrer",
    },
    LinkedEmissionCoverageShapeDefinitionV0 {
        shape_class: "shaken-referrer-liveness",
        reentry: "retain a target reached through a referrer module outside the emitted entry closure",
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
        shape_class: "empty-module",
        reentry: "retain an imported empty module with a module-boundary emission item",
    },
    LinkedEmissionCoverageShapeDefinitionV0 {
        shape_class: "comment-only-module",
        reentry: "retain an imported comment-only module with a module-boundary emission item",
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
    let mut fixtures = linked_emission_fixtures_v0();
    if perturbation == LinkedEmissionByteDifferentialPerturbationV0::DropFixture {
        fixtures.pop();
    }
    let analyses = fixtures
        .iter()
        .enumerate()
        .map(|(index, fixture)| {
            let case_perturbation = match perturbation {
                LinkedEmissionByteDifferentialPerturbationV0::AddUnexpectedRule if index == 0 => {
                    perturbation
                }
                LinkedEmissionByteDifferentialPerturbationV0::CollapseToLegacyBytes => perturbation,
                LinkedEmissionByteDifferentialPerturbationV0::DropReachableCrossModuleDeclaration
                    if fixture.id == "module-qualified-reachability" =>
                {
                    perturbation
                }
                LinkedEmissionByteDifferentialPerturbationV0::DropComposedDeclaration
                    if fixture.id == "module-qualified-composes-reachability" =>
                {
                    perturbation
                }
                LinkedEmissionByteDifferentialPerturbationV0::DropLiveDeclaration
                    if !fixture.reachability_references.is_empty() =>
                {
                    perturbation
                }
                LinkedEmissionByteDifferentialPerturbationV0::AddUnclaimedLinkedToken
                    if !fixture.reachability_references.is_empty() =>
                {
                    perturbation
                }
                LinkedEmissionByteDifferentialPerturbationV0::DropComposesReachability
                    if fixture.id == "module-qualified-composes-reachability" =>
                {
                    perturbation
                }
                LinkedEmissionByteDifferentialPerturbationV0::BreakEnginePathEquivalence
                    if fixture.id == "module-qualified-composes-reachability" =>
                {
                    perturbation
                }
                LinkedEmissionByteDifferentialPerturbationV0::AddUnattributedReachabilityReference
                    if fixture.id == "module-qualified-reachability" =>
                {
                    perturbation
                }
                LinkedEmissionByteDifferentialPerturbationV0::FlipAuthoredLivenessExpectation
                    if fixture.id == "two-hop-composes-liveness" =>
                {
                    perturbation
                }
                LinkedEmissionByteDifferentialPerturbationV0::MisattributeLinkedRule
                    if fixture.id == "entry-ordinal-skew" =>
                {
                    perturbation
                }
                _ => LinkedEmissionByteDifferentialPerturbationV0::None,
            };
            analyze_linked_emission_fixture_v0(fixture, case_perturbation)
        })
        .collect::<Result<Vec<_>, _>>()?;
    let census = summarize_linked_emission_coverage_census_v0(&fixtures, &analyses)?;
    let resolved_resolution_count = analyses
        .iter()
        .flat_map(|analysis| {
            analysis
                .linked_order
                .dependency_resolution_disclosures
                .iter()
        })
        .filter(|disclosure| disclosure.authority == BundleResolutionAuthorityV0::Resolved)
        .count();
    let legacy_path_inferred_resolution_count = analyses
        .iter()
        .flat_map(|analysis| {
            analysis
                .linked_order
                .dependency_resolution_disclosures
                .iter()
        })
        .filter(|disclosure| {
            disclosure.authority == BundleResolutionAuthorityV0::LegacyPathInferred
        })
        .count();
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
        resolution_authority_population_scope: "repository-owned linked-emission fixtures; excludes crates.io consumers",
        resolved_resolution_count,
        legacy_path_inferred_resolution_count,
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
        // FALSIFIER: id=linked-emission-rust-001 class=accounting via=DropReachableCrossModuleDeclaration producer=can-fail owner=linked-emission-instrument entry=committed-corpus-green
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
            // FALSIFIER: id=linked-emission-rust-002 class=accounting via=DropReachableCrossModuleDeclaration producer=can-fail owner=linked-emission-instrument entry=committed-corpus-green
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
    let mut unknown_structural_selector_count = 0usize;
    let mut unknown_at_rule_count = 0usize;
    let mut module_token_collisions = Vec::new();
    let mut unmodeled_declarations = Vec::new();
    let mut live_declaration_fixtures = Vec::new();

    for (fixture, analysis) in fixtures.iter().zip(analyses) {
        if fixture.id != analysis.case.fixture_id {
            // FALSIFIER: id=linked-emission-rust-003 class=accounting via=DropReachableCrossModuleDeclaration producer=can-fail owner=linked-emission-instrument entry=committed-corpus-green
            return Err(format!(
                "linked-emission fixture/analysis id mismatch: {} != {}",
                fixture.id, analysis.case.fixture_id
            ));
        }
        if !fixture_ids.insert(fixture.id.as_str()) {
            // FALSIFIER: id=linked-emission-rust-004 class=accounting via=DropReachableCrossModuleDeclaration producer=can-fail owner=linked-emission-instrument entry=committed-corpus-green
            return Err(format!(
                "duplicate linked-emission fixture id {}",
                fixture.id
            ));
        }
        if fixture.shape_classes.is_empty() {
            // FALSIFIER: id=linked-emission-rust-005 class=accounting via=DropReachableCrossModuleDeclaration producer=can-fail owner=linked-emission-instrument entry=committed-corpus-green
            return Err(format!(
                "linked-emission fixture {} has no enumerated shape class",
                fixture.id
            ));
        }
        for shape_class in &fixture.shape_classes {
            let Some(shape_fixtures) = fixtures_by_shape.get_mut(shape_class) else {
                // FALSIFIER: id=linked-emission-rust-006 class=accounting via=DropReachableCrossModuleDeclaration producer=can-fail owner=linked-emission-instrument entry=committed-corpus-green
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
                .emission_item_order
                .items
                .iter()
                .filter(|item| item.module_instance.module().as_str() == module.path)
                .count();
            let collection = collect_style_fact_collection(module.source.as_str(), module.dialect);
            blind_spots.push(LinkedEmissionMarkerBlindSpotV0 {
                fixture_id: fixture.id.clone(),
                module_path: module.path.clone(),
                shape_classes: fixture
                    .shape_classes
                    .iter()
                    .map(|shape_class| (*shape_class).to_string())
                    .collect(),
                emission_plan_entry_count,
                fact_categories: populated_fact_categories_v0(
                    &collection.facts,
                    &collection.emission_selectors,
                ),
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
        for item in &analysis.linked_order.emission_item_order.items {
            match item.kind {
                EmissionItemKindV0::UnknownStructuralSelector => {
                    unknown_structural_selector_count += 1;
                }
                EmissionItemKindV0::UnknownAtRule => {
                    unknown_at_rule_count += 1;
                }
                _ => {}
            }
        }
        let collision_summary = summarize_module_token_collisions_v0(fixture, analysis);
        module_token_collisions.extend(collision_summary.collisions);
        unmodeled_declarations.extend(collision_summary.unmodeled_declarations);
        if !fixture.reachability_references.is_empty() {
            live_declaration_fixtures.push(LinkedEmissionLiveDeclarationFixtureV0 {
                fixture_id: fixture.id.clone(),
                reachability_reference_count: fixture.reachability_references.len(),
                engine_input_style_source_count: analysis.engine_input_style_source_count,
                engine_input_path_form: analysis.engine_input_path_form,
                unmatched_target_style_path_count: analysis.unmatched_target_style_path_count,
                composes_resolution_count: analysis.composes_resolution_count,
                declaration_preserving_pass_ids: analysis.declaration_preserving_pass_ids.clone(),
                modules: fixture
                    .modules
                    .iter()
                    .map(|module| {
                        let declared_class_names =
                            declared_class_names_v0(module.source.as_str(), module.dialect)
                                .into_iter()
                                .collect();
                        let live_declared_class_names = analysis
                            .live_declared_names_by_module
                            .get(module.path.as_str())
                            .cloned()
                            .unwrap_or_default()
                            .into_iter()
                            .collect();
                        let module_expectations = fixture
                            .liveness_expectations
                            .iter()
                            .filter(|expectation| expectation.module_path == module.path)
                            .collect::<Vec<_>>();
                        let authored_live_class_names = module_expectations
                            .iter()
                            .filter(|expectation| {
                                expectation.verdict == LinkedEmissionLivenessVerdictV0::Live
                            })
                            .map(|expectation| expectation.class_name.to_string())
                            .collect();
                        LinkedEmissionLiveDeclarationModuleV0 {
                            module_path: module.path.clone(),
                            declared_class_names,
                            live_declared_class_names,
                            authored_liveness_expectation_count: module_expectations.len(),
                            authored_live_class_names,
                        }
                    })
                    .collect(),
            });
        }
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
    module_token_collisions.sort_by(|left, right| {
        left.fixture_id
            .cmp(&right.fixture_id)
            .then_with(|| left.emitted_token.cmp(&right.emitted_token))
    });
    unmodeled_declarations.sort_by(|left, right| {
        left.fixture_id
            .cmp(&right.fixture_id)
            .then_with(|| left.emission_path.cmp(right.emission_path))
            .then_with(|| left.module_path.cmp(&right.module_path))
            .then_with(|| left.original_name.cmp(&right.original_name))
    });
    validate_module_token_collision_paths_v0(module_token_collisions.as_slice())?;
    if !unmodeled_declarations.is_empty() {
        // FALSIFIER: id=linked-emission-unmodeled-declarations class=accounting via=MisattributeLinkedRule producer=can-fail owner=linked-emission-instrument entry=committed-corpus-green
        return Err(format!(
            "linked-emission collision accounting left live declarations unmodeled: {unmodeled_declarations:?}"
        ));
    }
    live_declaration_fixtures.sort_by(|left, right| left.fixture_id.cmp(&right.fixture_id));
    let in_domain_fixture_ids = live_declaration_fixtures
        .iter()
        .map(|fixture| fixture.fixture_id.clone())
        .collect::<Vec<_>>();
    let mut reachability_input_fixture_ids = fixtures
        .iter()
        .filter(|fixture| !fixture.reachability_references.is_empty())
        .map(|fixture| fixture.id.clone())
        .collect::<Vec<_>>();
    reachability_input_fixture_ids.sort();
    let ordinal_skew_path_split_collision_count = module_token_collisions
        .iter()
        .filter(|collision| collision.fixture_id == "entry-ordinal-skew")
        .count();
    let ordinal_skew_shared_model_collision_count = module_token_collisions
        .iter()
        .filter(|collision| {
            collision.fixture_id == "entry-ordinal-skew"
                && collision.observed_emission_paths.contains(&"linkedOrder")
        })
        .count();

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
        unknown_structural_selector_count,
        unknown_at_rule_count,
        module_token_collision_scope: "boundedFixtureRegressionTripwire",
        module_token_collision_count: module_token_collisions.len(),
        module_token_collisions,
        ordinal_skew_shared_model_collision_count,
        ordinal_skew_path_split_collision_count,
        token_model_by_emission_path: BTreeMap::from([
            ("importInlineLegacy", "moduleQualifiedRewriteTable"),
            ("linkedOrder", "moduleQualifiedRewriteTable"),
        ]),
        unmodeled_declarations,
        reachability_input_fixture_ids,
        in_domain_fixture_ids,
        live_declaration_fixtures,
        shapes,
        not_covered,
        fixture_observability,
        blind_spots,
        placement_witnesses,
    })
}

fn summarize_module_token_collisions_v0(
    fixture: &LinkedEmissionFixtureV0,
    analysis: &LinkedEmissionFixtureAnalysisV0,
) -> LinkedEmissionModuleTokenCollisionSummaryV0 {
    let mut declarations_by_module = BTreeMap::<String, BTreeSet<String>>::new();
    for module in &fixture.modules {
        let declarations = collect_style_fact_collection(module.source.as_str(), module.dialect)
            .facts
            .selectors
            .into_iter()
            .filter(|selector| selector.kind == ParsedSelectorFactKind::Class)
            .map(|selector| selector.name)
            .collect::<BTreeSet<_>>();
        declarations_by_module.insert(module.path.clone(), declarations);
    }
    let cross_module_names = declarations_by_module
        .values()
        .flat_map(|names| names.iter())
        .fold(BTreeMap::<&str, usize>::new(), |mut counts, name| {
            *counts.entry(name.as_str()).or_default() += 1;
            counts
        })
        .into_iter()
        .filter_map(|(name, count)| (count > 1).then_some(name))
        .collect::<BTreeSet<_>>();

    let mut linked_plan_owners = analysis
        .linked_order
        .emission_item_order
        .items
        .iter()
        .filter(|item| item.kind == EmissionItemKindV0::SelectorClass)
        .map(|item| {
            (
                item.module_instance.module().as_str().to_string(),
                item.name.trim_start_matches('.').to_string(),
            )
        })
        .collect::<BTreeSet<_>>();
    if analysis.collision_plan_owner_override
        && let Some(module) = fixture
            .modules
            .iter()
            .find(|module| module.path != fixture.entry_path)
    {
        linked_plan_owners.remove(&(module.path.clone(), "shared".to_string()));
        linked_plan_owners.insert((fixture.entry_path.clone(), "shared".to_string()));
    }
    let entry_rewrites = analysis
        .class_name_rewrites_by_module
        .get(fixture.entry_path.as_str())
        .cloned()
        .unwrap_or_default();
    let mut collisions =
        BTreeMap::<String, (BTreeMap<String, BTreeSet<String>>, BTreeSet<&'static str>)>::new();
    let mut unmodeled_declarations = Vec::new();
    for (emission_path, css) in [
        (
            analysis.case.legacy_emission_path,
            analysis.legacy_css.as_str(),
        ),
        (
            analysis.case.linked_emission_path,
            analysis.linked_css.as_str(),
        ),
    ] {
        let selector_counts = output_class_selector_counts_v0(css);
        let mut declarations_by_token =
            BTreeMap::<String, BTreeMap<String, BTreeSet<String>>>::new();
        for module in &fixture.modules {
            let Some(declarations) = declarations_by_module.get(module.path.as_str()) else {
                continue;
            };
            let rewrites = if emission_path == analysis.case.legacy_emission_path {
                &entry_rewrites
            } else {
                analysis
                    .class_name_rewrites_by_module
                    .get(module.path.as_str())
                    .unwrap_or(&entry_rewrites)
            };
            for original_name in declarations {
                if !cross_module_names.contains(original_name.as_str()) {
                    continue;
                }
                let linked_plan_claims_declaration = emission_path
                    != analysis.case.linked_emission_path
                    || linked_plan_owners.contains(&(module.path.clone(), original_name.clone()));
                let rewritten_name = rewrites
                    .get(original_name.as_str())
                    .map(String::as_str)
                    .unwrap_or(original_name.as_str());
                let emitted_token = if selector_counts.contains_key(rewritten_name) {
                    rewritten_name
                } else if selector_counts.contains_key(original_name.as_str()) {
                    original_name.as_str()
                } else {
                    let is_live = fixture.reachability_references.is_empty()
                        || analysis
                            .live_declared_names_by_module
                            .get(module.path.as_str())
                            .is_some_and(|names| names.contains(original_name));
                    if is_live {
                        unmodeled_declarations.push(LinkedEmissionUnmodeledDeclarationV0 {
                            fixture_id: fixture.id.clone(),
                            emission_path,
                            module_path: module.path.clone(),
                            original_name: original_name.clone(),
                            modeled_token: rewritten_name.to_string(),
                        });
                    }
                    continue;
                };
                if !linked_plan_claims_declaration {
                    unmodeled_declarations.push(LinkedEmissionUnmodeledDeclarationV0 {
                        fixture_id: fixture.id.clone(),
                        emission_path,
                        module_path: module.path.clone(),
                        original_name: original_name.clone(),
                        modeled_token: emitted_token.to_string(),
                    });
                    continue;
                }
                declarations_by_token
                    .entry(emitted_token.to_string())
                    .or_default()
                    .entry(module.path.clone())
                    .or_default()
                    .insert(original_name.clone());
            }
        }
        for (emitted_token, declarations) in declarations_by_token {
            if declarations.len() <= 1
                || selector_counts
                    .get(emitted_token.as_str())
                    .copied()
                    .unwrap_or_default()
                    <= 1
            {
                continue;
            }
            let collision = collisions.entry(emitted_token).or_default();
            for (module_path, names) in declarations {
                collision.0.entry(module_path).or_default().extend(names);
            }
            collision.1.insert(emission_path);
        }
    }

    let collisions = collisions
        .into_iter()
        .map(|(emitted_token, (declarations, observed_emission_paths))| {
            let module_paths = declarations.keys().cloned().collect::<Vec<_>>();
            let original_names = declarations
                .into_values()
                .flatten()
                .collect::<BTreeSet<_>>()
                .into_iter()
                .collect();
            let observed_emission_paths = observed_emission_paths.into_iter().collect::<Vec<_>>();
            let path_scope =
                module_token_collision_path_scope_v0(observed_emission_paths.as_slice());
            LinkedEmissionModuleTokenCollisionV0 {
                fixture_id: fixture.id.clone(),
                emitted_token,
                module_paths,
                original_names,
                observed_emission_paths,
                path_scope,
                reason: module_token_collision_reason_v0(path_scope).to_string(),
            }
        })
        .collect();
    LinkedEmissionModuleTokenCollisionSummaryV0 {
        collisions,
        unmodeled_declarations,
    }
}

fn module_token_collision_path_scope_v0(
    observed_emission_paths: &[&str],
) -> LinkedEmissionModuleTokenCollisionPathScopeV0 {
    match observed_emission_paths {
        ["importInlineLegacy", "linkedOrder"] => {
            LinkedEmissionModuleTokenCollisionPathScopeV0::BothPaths
        }
        ["importInlineLegacy"] => {
            LinkedEmissionModuleTokenCollisionPathScopeV0::ImportInlineLegacyOnly
        }
        ["linkedOrder"] => LinkedEmissionModuleTokenCollisionPathScopeV0::LinkedOrderOnly,
        _ => LinkedEmissionModuleTokenCollisionPathScopeV0::BothPaths,
    }
}

fn module_token_collision_reason_v0(
    path_scope: LinkedEmissionModuleTokenCollisionPathScopeV0,
) -> &'static str {
    match path_scope {
        LinkedEmissionModuleTokenCollisionPathScopeV0::BothPaths => {
            "entry and module-local rewrite tables produce the same cross-module token"
        }
        LinkedEmissionModuleTokenCollisionPathScopeV0::ImportInlineLegacyOnly => {
            "the entry rewrite table aliases declarations whose module-local rewrites remain distinct"
        }
        LinkedEmissionModuleTokenCollisionPathScopeV0::LinkedOrderOnly => {
            "module-local rewrite tables alias declarations that the entry rewrite table keeps distinct"
        }
    }
}

fn validate_module_token_collision_paths_v0(
    collisions: &[LinkedEmissionModuleTokenCollisionV0],
) -> Result<(), String> {
    for collision in collisions {
        let observed_scope =
            module_token_collision_path_scope_v0(collision.observed_emission_paths.as_slice());
        if collision.path_scope != observed_scope {
            // FALSIFIER: id=linked-emission-rust-007 class=structuralEntailment via=STRUCTURAL producer=entailed owner=linked-emission-collision-accounting entry=derived-path-scope reentry=path-scope-stored-independently
            return Err(format!(
                "emitted token collision {} in fixture {} declared scope {:?}, observed {:?}",
                collision.emitted_token,
                collision.fixture_id,
                collision.path_scope,
                collision.observed_emission_paths
            ));
        }
    }
    Ok(())
}

fn class_name_rewrites_by_module_v0(
    fixture: &LinkedEmissionFixtureV0,
) -> BTreeMap<String, BTreeMap<String, String>> {
    let styles = fixture
        .modules
        .iter()
        .map(|module| (module.path.as_str(), module.source.as_str()))
        .collect::<Vec<_>>();
    let resolution_inputs = OmenaQueryStyleResolutionInputsV0::default();
    fixture
        .modules
        .iter()
        .map(|module| {
            let context =
                summarize_omena_query_transform_context_from_sources_with_resolution_inputs(
                    module.path.as_str(),
                    styles.iter().copied(),
                    &resolution_inputs,
                )
                .context;
            (
                module.path.clone(),
                context
                    .class_name_rewrites
                    .into_iter()
                    .map(|rewrite| (rewrite.original_name, rewrite.rewritten_name))
                    .collect(),
            )
        })
        .collect()
}

fn emitted_class_name_v0(
    rewrites_by_module: &BTreeMap<String, BTreeMap<String, String>>,
    module_path: &str,
    original_name: &str,
) -> String {
    rewrites_by_module
        .get(module_path)
        .and_then(|rewrites| rewrites.get(original_name))
        .cloned()
        .unwrap_or_else(|| original_name.to_string())
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
    let mut pass_ids = vec!["import-inline".to_string(), "print-css".to_string()];
    let fixture_reachability = module_reachability_for_fixture_v0(fixture, perturbation);
    let module_reachability = fixture_reachability
        .as_ref()
        .map(|reachability| &reachability.report);
    let exercises_path_specific_class_rewrites =
        fixture.shape_classes.contains(&"entry-ordinal-skew");
    let class_name_rewrites_by_module =
        if module_reachability.is_some() || exercises_path_specific_class_rewrites {
            class_name_rewrites_by_module_v0(fixture)
        } else {
            BTreeMap::new()
        };
    if module_reachability.is_some() {
        pass_ids.push("css-modules-class-hashing".to_string());
        pass_ids.push("tree-shake-class".to_string());
    } else if exercises_path_specific_class_rewrites {
        pass_ids.push("css-modules-class-hashing".to_string());
    }
    let expected_pass_ids = if module_reachability.is_some() {
        [
            "import-inline",
            "print-css",
            "css-modules-class-hashing",
            "tree-shake-class",
        ]
        .as_slice()
    } else if exercises_path_specific_class_rewrites {
        ["import-inline", "print-css", "css-modules-class-hashing"].as_slice()
    } else {
        ["import-inline", "print-css"].as_slice()
    };
    if pass_ids
        .iter()
        .map(String::as_str)
        .ne(expected_pass_ids.iter().copied())
    {
        // FALSIFIER: id=linked-emission-pass-set-precondition class=shaking via=DropLiveDeclaration producer=can-fail owner=linked-emission-instrument entry=declaration-preserving-pass-set
        return Err(format!(
            "linked-emission declaration-retention pass precondition failed for fixture {}: {pass_ids:?}",
            fixture.id
        ));
    }
    let engine_input_style_source_count = fixture_reachability
        .as_ref()
        .map(|reachability| reachability.engine_input_style_source_count)
        .unwrap_or_default();
    if module_reachability.is_some()
        && (style_sources.is_empty() || engine_input_style_source_count == 0)
    {
        // FALSIFIER: id=linked-emission-source-text-precondition class=shaking via=DropComposesReachability producer=can-fail owner=linked-emission-instrument entry=nonempty-engine-source-domain
        return Err(format!(
            "linked-emission source-text precondition failed for fixture {}: build sources {}, engine sources {}",
            fixture.id,
            style_sources.len(),
            engine_input_style_source_count
        ));
    }
    let context = OmenaQueryTransformExecutionContextV0::default();
    let resolution_inputs = OmenaQueryStyleResolutionInputsV0::default();
    let run = |emission_path| {
        let input = OmenaQueryBundlePlanInputV0 {
            target_style_path: &fixture.entry_path,
            style_sources: &style_sources,
            source_map_sources: &style_sources,
            requested_pass_ids: &pass_ids,
            context: &context,
            resolution_inputs: &resolution_inputs,
            asset_rewrites: Vec::new(),
            bundle_entry_style_paths: &[],
        };
        let options = OmenaQueryConsumerBuildOptionsV0 {
            bundle_emission_path: emission_path,
            ..OmenaQueryConsumerBuildOptionsV0::default()
        };
        if let Some(module_reachability) = module_reachability.as_ref() {
            run_omena_query_bundle_with_module_reachability_and_execution_scope_evidence_and_options(
                input,
                &[],
                &options,
                module_reachability,
            )
        } else {
            run_omena_query_bundle_with_execution_scope_evidence_and_options(input, &[], &options)
        }
    };
    let legacy = run(OmenaQueryBundleEmissionPathV0::ImportInlineLegacy)?;
    let linked = run(OmenaQueryBundleEmissionPathV0::LinkedOrder)?;
    let legacy_emission_path = legacy.bundle_result.artifact.emission_path.as_wire_label();
    let linked_emission_path = linked.bundle_result.artifact.emission_path.as_wire_label();
    let linked_attribution = linked.reachability_attribution.clone();
    if let Some(attribution) = linked_attribution.as_ref()
        && !attribution.lost_class_names().is_empty()
    {
        // FALSIFIER: id=linked-emission-attribution-domain-consistency class=structuralEntailment via=STRUCTURAL producer=entailed owner=linked-emission-instrument entry=shared-admission-placement-domain reentry=split-module-attribution-domain
        return Err(format!(
            "fixture {} exposed names outside the shared attribution domain: {}",
            fixture.id,
            attribution.lost_class_names().join(", ")
        ));
    }
    let live_declared_names_by_module = linked_attribution
        .as_ref()
        .map(|attribution| live_declared_names_by_module_v0(fixture, attribution))
        .transpose()?
        .unwrap_or_default();
    let composes_resolution_count = module_reachability
        .as_ref()
        .map(|reachability| reachability.summary().css_module_composes_resolution_count)
        .unwrap_or_default();
    let legacy_css = legacy.bundle_result.artifact.output_css;
    let mut linked_css = linked.bundle_result.artifact.output_css;
    let unmatched_target_style_path_count = linked_attribution
        .as_ref()
        .map(|attribution| attribution.unmatched_target_style_paths().len())
        .unwrap_or_default();
    if fixture.id == "module-qualified-composes-reachability"
        && unmatched_target_style_path_count > 0
    {
        // FALSIFIER: id=linked-emission-attribution-path-equivalence class=liveness via=BreakEnginePathEquivalence producer=can-fail owner=linked-emission-instrument entry=equivalent-engine-and-build-paths
        return Err(format!(
            "linked-emission attribution could not reconcile {} target style paths in fixture {}",
            unmatched_target_style_path_count, fixture.id
        ));
    }
    match perturbation {
        LinkedEmissionByteDifferentialPerturbationV0::None => {}
        LinkedEmissionByteDifferentialPerturbationV0::AddUnexpectedRule => {
            linked_css.push_str("\n.injected-unexpected-rule { color: magenta; }");
        }
        LinkedEmissionByteDifferentialPerturbationV0::CollapseToLegacyBytes => {
            linked_css.clone_from(&legacy_css);
        }
        LinkedEmissionByteDifferentialPerturbationV0::DropReachableCrossModuleDeclaration => {
            remove_linked_declaration_for_fault_v0(&mut linked_css, "color: blue;")?;
        }
        LinkedEmissionByteDifferentialPerturbationV0::DropComposedDeclaration => {
            remove_linked_declaration_for_fault_v0(&mut linked_css, "padding: 2px;")?;
        }
        LinkedEmissionByteDifferentialPerturbationV0::DropLiveDeclaration => {
            remove_first_live_linked_declaration_for_fault_v0(
                fixture,
                &mut linked_css,
                linked_attribution.as_ref(),
                &class_name_rewrites_by_module,
            )?;
        }
        LinkedEmissionByteDifferentialPerturbationV0::AddUnclaimedLinkedToken => {
            linked_css.push_str("\n._unclaimed_linked_token { color: inherit; }");
        }
        LinkedEmissionByteDifferentialPerturbationV0::DropComposesReachability
        | LinkedEmissionByteDifferentialPerturbationV0::BreakEnginePathEquivalence
        | LinkedEmissionByteDifferentialPerturbationV0::AddUnattributedReachabilityReference
        | LinkedEmissionByteDifferentialPerturbationV0::FlipAuthoredLivenessExpectation
        | LinkedEmissionByteDifferentialPerturbationV0::DropFixture
        | LinkedEmissionByteDifferentialPerturbationV0::MisattributeLinkedRule => {}
    }
    validate_live_declared_names_survive_linked_emission_v0(
        fixture,
        &linked_css,
        linked_attribution.as_ref(),
        &class_name_rewrites_by_module,
    )?;
    validate_authored_liveness_expectations_v0(
        fixture,
        &live_declared_names_by_module,
        perturbation,
    )?;

    let marker_names = fixture
        .modules
        .iter()
        .flat_map(|module| {
            module.marker_names.iter().map(|marker| {
                emitted_class_name_v0(&class_name_rewrites_by_module, module.path.as_str(), marker)
            })
        })
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
    let projections =
        project_omena_transform_bundle_linker_and_emission_items(&linker_modules, &[]);
    let import_graph_module_order = independent_import_graph_module_order_v0(
        fixture,
        projections.linker_projection().inputs(),
    )?;
    let linked_order =
        link_omena_transform_bundle_projection_with_emission_items_and_resolved_dependencies_and_options(
            std::slice::from_ref(&fixture.entry_path),
            projections.linker_projection(),
            projections.emission_item_projection(),
            &[],
            &[],
            TransformBundleLinkOptionsV0::default()
                .with_emission_ordering_policy(EmissionOrderingPolicyV0::ImportOrderPreserving),
        )
        .map_err(|error| format!("fixture {} could not be linked: {error:?}", fixture.id))?;
    let emission_plan_module_order = emission_item_module_order_v0(&linked_order);
    if emission_plan_module_order != import_graph_module_order {
        // FALSIFIER: id=linked-emission-rust-008 class=accounting via=DropReachableCrossModuleDeclaration producer=can-fail owner=linked-emission-instrument entry=committed-corpus-green
        return Err(format!(
            "fixture {} emission plan diverged from its independently traversed import graph: {:?} != {:?}",
            fixture.id, emission_plan_module_order, import_graph_module_order
        ));
    }
    let modules_by_path = fixture
        .modules
        .iter()
        .map(|module| (module.path.as_str(), module))
        .collect::<BTreeMap<_, _>>();
    let authoritative_marker_order = import_graph_module_order
        .iter()
        .flat_map(|path| {
            modules_by_path
                .get(path.as_str())
                .into_iter()
                .flat_map(|module| {
                    module.marker_names.iter().map(|marker| {
                        emitted_class_name_v0(
                            &class_name_rewrites_by_module,
                            module.path.as_str(),
                            marker,
                        )
                    })
                })
        })
        .collect::<Vec<_>>();
    let authoritative_module_order =
        observable_module_order_v0(fixture, &import_graph_module_order);
    let linked_output_module_order =
        if perturbation == LinkedEmissionByteDifferentialPerturbationV0::CollapseToLegacyBytes {
            authoritative_module_order.clone()
        } else {
            output_module_order_v0(fixture, &linked_css)?
        };
    let linked_output_module_order_matches_authority =
        linked_output_module_order == authoritative_module_order;
    let legacy_marker_order = output_marker_order_v0(&legacy_css, &marker_names);
    let linked_marker_order = output_marker_order_v0(&linked_css, &marker_names);
    if perturbation == LinkedEmissionByteDifferentialPerturbationV0::CollapseToLegacyBytes
        && linked_marker_order != authoritative_marker_order
    {
        // FALSIFIER: id=linked-emission-collapse-marker-order class=accounting via=CollapseToLegacyBytes producer=can-fail owner=linked-emission-instrument entry=linked-marker-order-matches-authority
        return Err(format!(
            "linked emission collapsed to non-authoritative marker order in fixture {}: {:?} != {:?}",
            fixture.id, linked_marker_order, authoritative_marker_order
        ));
    }
    let linked_modules_emitted_once = marker_names.iter().all(|marker| {
        linked_marker_order
            .iter()
            .filter(|candidate| *candidate == marker)
            .count()
            == 1
    });
    let semantic =
        compare_omena_query_transform_css_semantics_v0(&legacy_css, &linked_css, StyleDialect::Css);
    let asset_urls_preserved =
        css_url_arguments_v0(&legacy_css) == css_url_arguments_v0(&linked_css);
    let expected_module_reachability_delta = !fixture.liveness_expectations.is_empty();
    let expected_semantics = if exercises_path_specific_class_rewrites {
        true
    } else if fixture.reachability_references.is_empty() {
        semantic.preserved
    } else {
        expected_module_reachability_delta
    };
    let byte_equal = legacy_css == linked_css;
    let difference_observation = LinkedEmissionDifferenceObservationV0 {
        legacy_css: &legacy_css,
        linked_css: &linked_css,
        authoritative_marker_order: &authoritative_marker_order,
        legacy_marker_order: &legacy_marker_order,
        linked_marker_order: &linked_marker_order,
        linked_output_module_order_matches_authority,
    };
    let reasons = derive_difference_reasons_v0(fixture, &linked_order, &difference_observation);
    let difference_class = if byte_equal {
        LinkedEmissionByteDifferenceClassV0::Equivalent
    } else if expected_semantics
        && asset_urls_preserved
        && linked_modules_emitted_once
        && linked_marker_order == authoritative_marker_order
        && linked_output_module_order_matches_authority
        && !reasons.is_empty()
    {
        LinkedEmissionByteDifferenceClassV0::Expected
    } else {
        LinkedEmissionByteDifferenceClassV0::Unexpected
    };

    let case = LinkedEmissionByteDifferentialCaseV0 {
        fixture_id: fixture.id.clone(),
        module_count: fixture.modules.len(),
        legacy_emission_path,
        linked_emission_path,
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
        authoritative_module_order,
        linked_output_module_order,
        linked_output_module_order_matches_authority,
        linked_modules_emitted_once,
        difference_class,
        reasons,
    };
    Ok(LinkedEmissionFixtureAnalysisV0 {
        case,
        linked_order,
        legacy_css,
        linked_css,
        class_name_rewrites_by_module,
        live_declared_names_by_module,
        engine_input_style_source_count,
        engine_input_path_form: fixture_reachability
            .as_ref()
            .map_or("notApplicable", |reachability| {
                reachability.engine_input_path_form
            }),
        unmatched_target_style_path_count,
        composes_resolution_count,
        declaration_preserving_pass_ids: pass_ids,
        collision_plan_owner_override: perturbation
            == LinkedEmissionByteDifferentialPerturbationV0::MisattributeLinkedRule,
    })
}

fn module_reachability_for_fixture_v0(
    fixture: &LinkedEmissionFixtureV0,
    perturbation: LinkedEmissionByteDifferentialPerturbationV0,
) -> Option<LinkedEmissionFixtureReachabilityV0> {
    if fixture.reachability_references.is_empty() {
        return None;
    }
    let workspace_modules = fixture
        .modules
        .iter()
        .chain(&fixture.workspace_only_modules)
        .collect::<Vec<_>>();
    let uses_equivalent_cross_form_paths = fixture.id == "module-qualified-composes-reachability";
    let engine_input_path_form = if uses_equivalent_cross_form_paths {
        "absoluteEngineRelativeBuild"
    } else {
        "sameAsBuild"
    };
    let engine_style_path = |module_index: usize, module_path: &str| {
        if perturbation == LinkedEmissionByteDifferentialPerturbationV0::BreakEnginePathEquivalence
            && uses_equivalent_cross_form_paths
        {
            format!("/engine-only/module-{module_index}.module.css")
        } else if uses_equivalent_cross_form_paths {
            format!("/workspace/{}", module_path.trim_start_matches('/'))
        } else {
            module_path.to_string()
        }
    };
    let range = fixture_range_v0();
    let mut input = EngineInputV2 {
        version: "2".to_string(),
        sources: fixture
            .reachability_references
            .iter()
            .map(|reference| {
                let module = workspace_modules[reference.module_index];
                SourceAnalysisInputV2 {
                    document: SourceDocumentV2 {
                        class_expressions: vec![ClassExpressionInputV2 {
                            id: reference.id.to_string(),
                            kind: "styleAccess".to_string(),
                            scss_module_path: engine_style_path(
                                reference.module_index,
                                module.path.as_str(),
                            ),
                            range: range.clone(),
                            class_name: Some(reference.class_name.to_string()),
                            root_binding_decl_id: None,
                            access_path: Some(vec![reference.class_name.to_string()]),
                        }],
                    },
                }
            })
            .collect(),
        styles: workspace_modules
            .iter()
            .enumerate()
            .map(|(module_index, module)| {
                let collection =
                    collect_style_fact_collection(module.source.as_str(), module.dialect);
                StyleAnalysisInputV2 {
                    file_path: engine_style_path(module_index, module.path.as_str()),
                    source: Some(module.source.clone()),
                    document: StyleDocumentV2 {
                        selectors: collection
                            .facts
                            .selectors
                            .iter()
                            .filter(|selector| selector.kind == ParsedSelectorFactKind::Class)
                            .map(|selector| StyleSelectorV2 {
                                name: selector.name.clone(),
                                view_kind: "canonical".to_string(),
                                canonical_name: Some(selector.name.clone()),
                                range: range.clone(),
                                nested_safety: Some("safe".to_string()),
                                composes: None,
                                bem_suffix: None,
                            })
                            .collect(),
                    },
                }
            })
            .collect(),
        type_facts: fixture
            .reachability_references
            .iter()
            .map(|reference| TypeFactEntryV2 {
                file_path: "linked-byte/references.tsx".to_string(),
                expression_id: reference.id.to_string(),
                facts: StringTypeFactsV2 {
                    kind: "exact".to_string(),
                    constraint_kind: None,
                    values: Some(vec![reference.class_name.to_string()]),
                    prefix: None,
                    suffix: None,
                    min_len: None,
                    max_len: None,
                    char_must: None,
                    char_may: None,
                    may_include_other_chars: None,
                    provenance: None,
                },
                control_flow_graph: None,
            })
            .collect(),
    };
    if perturbation == LinkedEmissionByteDifferentialPerturbationV0::DropComposesReachability
        && fixture.id == "module-qualified-composes-reachability"
    {
        input.styles[0].source = None;
    }
    if perturbation
        == LinkedEmissionByteDifferentialPerturbationV0::AddUnattributedReachabilityReference
    {
        input.sources.push(SourceAnalysisInputV2 {
            document: SourceDocumentV2 {
                class_expressions: vec![ClassExpressionInputV2 {
                    id: "unattributed-dependency-dead-reference".to_string(),
                    kind: "styleAccess".to_string(),
                    scss_module_path: "linked-byte/no-target.module.css".to_string(),
                    range: range.clone(),
                    class_name: Some("dependency-dead".to_string()),
                    root_binding_decl_id: None,
                    access_path: Some(vec!["dependency-dead".to_string()]),
                }],
            },
        });
        input.type_facts.push(TypeFactEntryV2 {
            file_path: "linked-byte/references.tsx".to_string(),
            expression_id: "unattributed-dependency-dead-reference".to_string(),
            facts: StringTypeFactsV2 {
                kind: "exact".to_string(),
                constraint_kind: None,
                values: Some(vec!["dependency-dead".to_string()]),
                prefix: None,
                suffix: None,
                min_len: None,
                max_len: None,
                char_must: None,
                char_may: None,
                may_include_other_chars: None,
                provenance: None,
            },
            control_flow_graph: None,
        });
    }
    let engine_input_style_source_count = input
        .styles
        .iter()
        .filter(|style| style.source.is_some())
        .count();
    let target_style_path = fixture
        .reachability_references
        .first()
        .and_then(|reference| workspace_modules.get(reference.module_index))
        .map_or(fixture.entry_path.as_str(), |module| module.path.as_str());
    Some(LinkedEmissionFixtureReachabilityV0 {
        report: derive_omena_query_module_reachability_from_engine_input(
            &input,
            target_style_path,
            true,
        ),
        engine_input_style_source_count,
        engine_input_path_form,
    })
}

#[cfg(test)]
fn module_qualified_reachability_converges_v0(
    fixture: &LinkedEmissionFixtureV0,
    legacy_css: &str,
    linked_css: &str,
) -> bool {
    if !fixture.shape_classes.iter().any(|shape| {
        matches!(
            *shape,
            "module-qualified-reachability" | "at-rule-nested-liveness"
        )
    }) {
        return false;
    }
    let legacy_css = remove_ascii_whitespace_v0(legacy_css);
    let linked_css = remove_ascii_whitespace_v0(linked_css);
    if fixture.id == "module-qualified-at-rule-reachability" {
        return legacy_css.contains("border:0")
            && legacy_css.contains("padding:4px")
            && linked_css.contains("border:0")
            && linked_css.contains("padding:4px")
            && !linked_css.contains("color:gray")
            && !linked_css.contains("padding:8px");
    }
    if fixture.id == "module-qualified-composes-reachability" {
        return legacy_css.contains("padding:2px")
            && legacy_css.contains("color:red")
            && !legacy_css.contains("color:gray")
            && linked_css.contains("padding:2px")
            && linked_css.contains("color:blue")
            && linked_css.contains("color:red")
            && !linked_css.contains("color:gray");
    }
    legacy_css.contains("color:blue")
        && legacy_css.contains("color:red")
        && !legacy_css.contains("color:tan")
        && !legacy_css.contains("color:gray")
        && linked_css.contains("color:blue")
        && linked_css.contains("color:red")
        && !linked_css.contains("color:tan")
        && !linked_css.contains("color:gray")
}

fn fixture_range_v0() -> RangeV2 {
    RangeV2 {
        start: PositionV2 {
            line: 0,
            character: 0,
        },
        end: PositionV2 {
            line: 0,
            character: 1,
        },
    }
}

fn populated_fact_categories_v0(
    facts: &ParsedStyleFacts,
    emission_selectors: &ParsedEmissionSelectorFactsV0,
) -> Vec<&'static str> {
    let mut categories = Vec::new();
    if !facts.selectors.is_empty() {
        categories.push("selectors");
    }
    if !emission_selectors.selectors.is_empty() {
        categories.push("emissionSelectors");
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
        // FALSIFIER: id=linked-emission-rust-009 class=accounting via=DropReachableCrossModuleDeclaration producer=can-fail owner=linked-emission-instrument entry=committed-corpus-green
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
            // FALSIFIER: id=linked-emission-rust-010 class=accounting via=DropReachableCrossModuleDeclaration producer=can-fail owner=linked-emission-instrument entry=committed-corpus-green
            return Err(format!(
                "{} designates an absent selector-less module: {path}",
                definition.fixture.id
            ));
        }
    }
    let emission_plan_entry_count = analysis
        .linked_order
        .emission_item_order
        .items
        .iter()
        .filter(|item| {
            selectorless_module_paths
                .iter()
                .any(|path| path == item.module_instance.module().as_str())
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
    linked_order: &LinkedStylesheetWithEmissionItemsV0,
    observation: &LinkedEmissionDifferenceObservationV0<'_>,
) -> Vec<LinkedEmissionByteDifferenceReasonV0> {
    let mut reasons = BTreeSet::new();
    if observation.legacy_marker_order != observation.linked_marker_order
        && observation.linked_marker_order == observation.authoritative_marker_order
    {
        reasons.insert(LinkedEmissionByteDifferenceReasonV0::GlobalModuleOrder);
    }
    let marker_blind_modules = fixture
        .modules
        .iter()
        .filter(|module| module.marker_names.is_empty())
        .map(|module| module.path.as_str())
        .collect::<BTreeSet<_>>();
    if observation.legacy_css != observation.linked_css
        && observation.linked_output_module_order_matches_authority
        && !marker_blind_modules.is_empty()
        && marker_blind_modules.iter().all(|path| {
            linked_order
                .emission_item_order
                .items
                .iter()
                .any(|item| item.module_instance.module().as_str() == *path)
        })
    {
        reasons.insert(LinkedEmissionByteDifferenceReasonV0::ImportGraphModulePlacement);
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
        && sequence_splits_marker_group_v0(observation.legacy_marker_order, entry_markers)
        && !sequence_splits_marker_group_v0(observation.linked_marker_order, entry_markers)
    {
        reasons.insert(LinkedEmissionByteDifferenceReasonV0::EntryInterleaveCollapse);
    }
    if marker_sets_by_module.values().any(|markers| {
        sequence_splits_marker_group_v0(observation.legacy_marker_order, markers)
            && !sequence_splits_marker_group_v0(observation.linked_marker_order, markers)
    }) {
        reasons.insert(LinkedEmissionByteDifferenceReasonV0::PerModuleGrouping);
    }

    let mut inbound_counts = BTreeMap::new();
    for fact in &linked_order
        .linked_stylesheet
        .emission_plan
        .dependency_facts
    {
        *inbound_counts
            .entry(fact.to_module.module().as_str())
            .or_insert(0usize) += 1;
    }
    if inbound_counts.iter().any(|(path, count)| {
        *count > 1
            && marker_sets_by_module.get(path).is_some_and(|markers| {
                markers.iter().any(|marker| {
                    observation
                        .legacy_marker_order
                        .iter()
                        .filter(|candidate| *candidate == marker)
                        .count()
                        > observation
                            .linked_marker_order
                            .iter()
                            .filter(|candidate| *candidate == marker)
                            .count()
                })
            })
    }) {
        reasons.insert(LinkedEmissionByteDifferenceReasonV0::SharedImportSingleEmission);
    }
    if remove_ascii_whitespace_v0(observation.legacy_css)
        == remove_ascii_whitespace_v0(observation.linked_css)
    {
        reasons.insert(LinkedEmissionByteDifferenceReasonV0::FormattingNormalization);
    }
    reasons.into_iter().collect()
}

fn emission_item_module_order_v0(linked: &LinkedStylesheetWithEmissionItemsV0) -> Vec<String> {
    let mut seen = BTreeSet::new();
    linked
        .emission_item_order
        .items
        .iter()
        .filter_map(|item| {
            let module_path = item.module_instance.module().as_str();
            seen.insert(module_path).then(|| module_path.to_string())
        })
        .collect()
}

fn independent_import_graph_module_order_v0(
    fixture: &LinkedEmissionFixtureV0,
    inputs: &[LinkerInputV0],
) -> Result<Vec<String>, String> {
    let module_paths = fixture
        .modules
        .iter()
        .map(|module| module.path.clone())
        .collect::<BTreeSet<_>>();
    let mut dependencies = BTreeMap::<String, Vec<(u32, String)>>::new();
    for input in inputs {
        if !module_paths.contains(input.source_path.as_str()) {
            continue;
        }
        let mut input_dependencies = input
            .dependency_edges
            .iter()
            .map(|edge| {
                resolve_fixture_import_path_v0(
                    input.source_path.as_str(),
                    edge.import_source.as_str(),
                    &module_paths,
                )
                .map(|target| (edge.import_ordinal.unwrap_or(u32::MAX), target))
            })
            .collect::<Result<Vec<_>, _>>()?;
        input_dependencies.sort();
        input_dependencies.dedup();
        dependencies.insert(input.source_path.clone(), input_dependencies);
    }

    let mut visiting = BTreeSet::new();
    let mut visited = BTreeSet::new();
    let mut order = Vec::new();
    visit_import_graph_module_v0(
        fixture.entry_path.as_str(),
        &dependencies,
        &mut visiting,
        &mut visited,
        &mut order,
    )?;
    if visited != module_paths {
        let missing = module_paths
            .difference(&visited)
            .cloned()
            .collect::<Vec<_>>();
        // FALSIFIER: id=linked-emission-rust-011 class=accounting via=DropReachableCrossModuleDeclaration producer=can-fail owner=linked-emission-instrument entry=committed-corpus-green
        return Err(format!(
            "fixture {} contains modules outside the entry import graph: {missing:?}",
            fixture.id
        ));
    }
    Ok(order)
}

fn visit_import_graph_module_v0(
    source_path: &str,
    dependencies: &BTreeMap<String, Vec<(u32, String)>>,
    visiting: &mut BTreeSet<String>,
    visited: &mut BTreeSet<String>,
    order: &mut Vec<String>,
) -> Result<(), String> {
    if visited.contains(source_path) {
        return Ok(());
    }
    if !visiting.insert(source_path.to_string()) {
        // FALSIFIER: id=linked-emission-rust-012 class=accounting via=DropReachableCrossModuleDeclaration producer=can-fail owner=linked-emission-instrument entry=committed-corpus-green
        return Err(format!(
            "linked-emission fixture import graph contains a cycle at {source_path}"
        ));
    }
    if let Some(source_dependencies) = dependencies.get(source_path) {
        for (_, target) in source_dependencies {
            visit_import_graph_module_v0(target.as_str(), dependencies, visiting, visited, order)?;
        }
    }
    visiting.remove(source_path);
    visited.insert(source_path.to_string());
    order.push(source_path.to_string());
    Ok(())
}

fn resolve_fixture_import_path_v0(
    source_path: &str,
    import_source: &str,
    module_paths: &BTreeSet<String>,
) -> Result<String, String> {
    let base = source_path.rsplit_once('/').map_or("", |(base, _)| base);
    let joined = if import_source.starts_with('/') || base.is_empty() {
        import_source.to_string()
    } else {
        format!("{base}/{import_source}")
    };
    let normalized = normalize_fixture_path_v0(joined.as_str())?;
    if module_paths.contains(normalized.as_str()) {
        Ok(normalized)
    } else {
        Err(format!(
            "linked-emission fixture import {import_source:?} from {source_path} does not resolve inside the fixture"
        ))
    }
}

fn normalize_fixture_path_v0(path: &str) -> Result<String, String> {
    let mut components = Vec::new();
    let portable_path = path.replace('\\', "/");
    for component in portable_path.split('/') {
        match component {
            "" | "." => {}
            ".." => {
                if components.pop().is_none() {
                    // FALSIFIER: id=linked-emission-rust-013 class=accounting via=DropReachableCrossModuleDeclaration producer=can-fail owner=linked-emission-instrument entry=committed-corpus-green
                    return Err(format!("fixture path escapes its root: {path}"));
                }
            }
            component => components.push(component),
        }
    }
    Ok(components.join("/"))
}

fn observable_module_order_v0(
    fixture: &LinkedEmissionFixtureV0,
    import_graph_module_order: &[String],
) -> Vec<String> {
    let observable_paths = fixture
        .modules
        .iter()
        .filter(|module| !module.order_probe.trim().is_empty())
        .map(|module| module.path.as_str())
        .collect::<BTreeSet<_>>();
    import_graph_module_order
        .iter()
        .filter(|path| observable_paths.contains(path.as_str()))
        .cloned()
        .collect()
}

fn output_module_order_v0(
    fixture: &LinkedEmissionFixtureV0,
    output_css: &str,
) -> Result<Vec<String>, String> {
    let compact_output = remove_ascii_whitespace_v0(output_css);
    let mut positioned_modules = fixture
        .modules
        .iter()
        .filter(|module| !module.order_probe.trim().is_empty())
        .map(|module| {
            let compact_probe = remove_ascii_whitespace_v0(module.order_probe.as_str());
            let positions = compact_output
                .match_indices(compact_probe.as_str())
                .map(|(index, _)| index)
                .collect::<Vec<_>>();
            if positions.len() != 1 {
                // FALSIFIER: id=linked-emission-rust-014 class=accounting via=DropReachableCrossModuleDeclaration producer=can-fail owner=linked-emission-instrument entry=committed-corpus-green
                return Err(format!(
                    "linked-emission fixture {} expected one order probe {:?} for {}, observed {}",
                    fixture.id,
                    module.order_probe,
                    module.path,
                    positions.len()
                ));
            }
            Ok((positions[0], module.path.clone()))
        })
        .collect::<Result<Vec<_>, String>>()?;
    positioned_modules.sort();
    Ok(positioned_modules
        .into_iter()
        .map(|(_, module_path)| module_path)
        .collect())
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

fn output_class_selector_counts_v0(source: &str) -> BTreeMap<String, usize> {
    let mut counts = BTreeMap::new();
    collect_style_fact_collection(source, StyleDialect::Css)
        .facts
        .selectors
        .into_iter()
        .filter(|selector| selector.kind == ParsedSelectorFactKind::Class)
        .for_each(|selector| {
            *counts.entry(selector.name).or_default() += 1;
        });
    counts
}

fn declared_class_names_v0(source: &str, dialect: StyleDialect) -> BTreeSet<String> {
    let mut names = collect_style_fact_collection(source, dialect)
        .facts
        .selectors
        .into_iter()
        .filter(|selector| selector.kind == ParsedSelectorFactKind::Class)
        .map(|selector| selector.name)
        .collect::<BTreeSet<_>>();
    let tokens = summarize_omena_parser_lex(source, dialect).tokens;
    let mut index = 0usize;
    while index < tokens.len() {
        if tokens[index].kind != "Colon" {
            index += 1;
            continue;
        }
        let Some(global_index) = next_non_trivia_lex_token_v0(&tokens, index + 1) else {
            break;
        };
        if tokens[global_index].kind != "Ident" || tokens[global_index].text != "global" {
            index = global_index + 1;
            continue;
        }
        let Some(open_index) = next_non_trivia_lex_token_v0(&tokens, global_index + 1) else {
            break;
        };
        let (open_kind, close_kind) = match tokens[open_index].kind.as_str() {
            "LeftParen" => ("LeftParen", "RightParen"),
            "LeftBrace" => ("LeftBrace", "RightBrace"),
            _ => {
                index = open_index + 1;
                continue;
            }
        };
        let mut depth = 1usize;
        let mut cursor = open_index + 1;
        while cursor < tokens.len() && depth > 0 {
            if tokens[cursor].kind == open_kind {
                depth += 1;
            } else if tokens[cursor].kind == close_kind {
                depth -= 1;
                if depth == 0 {
                    break;
                }
            } else if tokens[cursor].kind == "Dot"
                && let Some(name_index) = next_non_trivia_lex_token_v0(&tokens, cursor + 1)
                && tokens[name_index].kind == "Ident"
            {
                names.insert(tokens[name_index].text.clone());
            }
            cursor += 1;
        }
        index = cursor.saturating_add(1);
    }
    names
}

fn next_non_trivia_lex_token_v0(
    tokens: &[omena_parser::OmenaParserLexTokenV0],
    mut index: usize,
) -> Option<usize> {
    while index < tokens.len() {
        if !matches!(
            tokens[index].kind.as_str(),
            "Whitespace" | "Comment" | "LineComment"
        ) {
            return Some(index);
        }
        index += 1;
    }
    None
}

fn validate_live_declared_names_survive_linked_emission_v0(
    fixture: &LinkedEmissionFixtureV0,
    linked_css: &str,
    attribution: Option<&OmenaQueryModuleReachabilityAttributionReportV0>,
    rewrites_by_module: &BTreeMap<String, BTreeMap<String, String>>,
) -> Result<(), String> {
    if fixture.reachability_references.is_empty() {
        return Ok(());
    }
    let Some(attribution) = attribution else {
        // FALSIFIER: id=linked-emission-attribution-required class=shaking via=DropLiveDeclaration producer=can-fail owner=linked-emission-instrument entry=attribution-present-for-in-domain-fixture
        return Err(format!(
            "linked emission omitted reachability attribution for in-domain fixture {}",
            fixture.id
        ));
    };
    let live_declared = live_declared_names_by_module_v0(fixture, attribution)?;
    let mut owners_by_token = BTreeMap::<String, BTreeSet<(String, String)>>::new();
    let mut live_owners_by_token = BTreeMap::<String, BTreeSet<(String, String)>>::new();
    for (module_path, names) in &live_declared {
        for name in names {
            live_owners_by_token
                .entry(emitted_class_name_v0(rewrites_by_module, module_path, name))
                .or_default()
                .insert((module_path.clone(), name.clone()));
        }
    }
    for module in &fixture.modules {
        for name in declared_class_names_v0(module.source.as_str(), module.dialect) {
            owners_by_token
                .entry(emitted_class_name_v0(
                    rewrites_by_module,
                    module.path.as_str(),
                    name.as_str(),
                ))
                .or_default()
                .insert((module.path.clone(), name));
        }
    }
    let ambiguous_tokens = owners_by_token
        .iter()
        .filter(|(token, owners)| {
            if owners.len() == 1 {
                return false;
            }
            live_owners_by_token
                .get(token.as_str())
                .is_none_or(|live_owners| live_owners.len() != 1)
        })
        .map(|(token, owners)| format!("{token}={owners:?}"))
        .collect::<Vec<_>>();
    let output_tokens = collect_style_fact_collection(linked_css, StyleDialect::Css)
        .facts
        .selectors
        .into_iter()
        .filter(|selector| selector.kind == ParsedSelectorFactKind::Class)
        .map(|selector| selector.name)
        .collect::<BTreeSet<_>>();
    let unclaimed_output_tokens = output_tokens
        .iter()
        .filter(|token| !owners_by_token.contains_key(token.as_str()))
        .cloned()
        .collect::<Vec<_>>();
    let missing_live_tokens = live_declared
        .iter()
        .flat_map(|(module_path, names)| {
            names.iter().map(|name| {
                emitted_class_name_v0(rewrites_by_module, module_path.as_str(), name.as_str())
            })
        })
        .filter(|token| !output_tokens.contains(token.as_str()))
        .collect::<Vec<_>>();
    if !ambiguous_tokens.is_empty()
        || !unclaimed_output_tokens.is_empty()
        || !missing_live_tokens.is_empty()
    {
        // FALSIFIER: id=linked-emission-token-universe-closure class=shaking via=AddUnclaimedLinkedToken producer=can-fail owner=linked-emission-instrument entry=one-owner-per-output-token
        return Err(format!(
            "linked-emission live token universe is not closed in fixture {}: ambiguous [{}], unclaimed [{}], missing [{}]",
            fixture.id,
            ambiguous_tokens.join(", "),
            unclaimed_output_tokens.join(", "),
            missing_live_tokens.join(", ")
        ));
    }

    for module in &fixture.modules {
        let Some(live_names) = live_declared.get(module.path.as_str()) else {
            continue;
        };
        for name in live_names {
            let token = emitted_class_name_v0(rewrites_by_module, module.path.as_str(), name);
            let source_declarations =
                rule_declaration_counts_for_class_v0(module.source.as_str(), name.as_str());
            let emitted_declarations =
                rule_declaration_counts_for_class_v0(linked_css, token.as_str());
            for (declaration, expected_count) in source_declarations {
                let actual_count = emitted_declarations
                    .get(declaration.as_str())
                    .copied()
                    .unwrap_or_default();
                if actual_count < expected_count {
                    // FALSIFIER: id=linked-emission-live-declaration-retention class=shaking via=DropLiveDeclaration producer=can-fail owner=linked-emission-instrument entry=all-live-declarations-emitted
                    return Err(format!(
                        "linked emission lost live declaration in fixture {}, module {}, name {}, token {}: {} x{} (observed {})",
                        fixture.id,
                        module.path,
                        name,
                        token,
                        declaration,
                        expected_count,
                        actual_count
                    ));
                }
            }
        }
    }
    Ok(())
}

fn live_declared_names_by_module_v0(
    fixture: &LinkedEmissionFixtureV0,
    attribution: &OmenaQueryModuleReachabilityAttributionReportV0,
) -> Result<BTreeMap<String, BTreeSet<String>>, String> {
    let unattributed = attribution
        .unattributed_class_names()
        .iter()
        .map(String::as_str)
        .collect::<BTreeSet<_>>();
    let mut live_declared = BTreeMap::new();
    for module in &fixture.modules {
        let declared = declared_class_names_v0(module.source.as_str(), module.dialect);
        let Some(entry) = attribution.entry_for_style_path(module.path.as_str()) else {
            // FALSIFIER: id=linked-emission-build-module-attribution class=shaking via=DropLiveDeclaration producer=can-fail owner=linked-emission-instrument entry=all-build-modules-attributed
            return Err(format!(
                "linked-emission attribution omitted build module {} in fixture {}",
                module.path, fixture.id
            ));
        };
        let names = entry
            .class_names()
            .iter()
            .filter(|name| {
                !unattributed.contains(name.as_str()) && declared.contains(name.as_str())
            })
            .cloned()
            .collect::<BTreeSet<_>>();
        live_declared.insert(module.path.clone(), names);
    }
    Ok(live_declared)
}

fn validate_authored_liveness_expectations_v0(
    fixture: &LinkedEmissionFixtureV0,
    live_declared_names_by_module: &BTreeMap<String, BTreeSet<String>>,
    perturbation: LinkedEmissionByteDifferentialPerturbationV0,
) -> Result<(), String> {
    let modules = fixture
        .modules
        .iter()
        .map(|module| (module.path.as_str(), module))
        .collect::<BTreeMap<_, _>>();
    if fixture.reachability_references.is_empty() {
        if fixture.liveness_expectations.is_empty() {
            return Ok(());
        }
        // FALSIFIER: id=linked-emission-authored-domain-reachability class=liveness via=DropComposesReachability producer=can-fail owner=linked-emission-instrument entry=expectations-only-on-in-domain-fixtures
        return Err(format!(
            "fixture {} has authored liveness expectations without reachability input",
            fixture.id
        ));
    }
    let declared_keys = fixture
        .modules
        .iter()
        .flat_map(|module| {
            declared_class_names_v0(module.source.as_str(), module.dialect)
                .into_iter()
                .map(|name| (module.path.clone(), name))
        })
        .collect::<BTreeSet<_>>();
    let mut authored_keys = BTreeSet::new();
    for (expectation_index, expectation) in fixture.liveness_expectations.iter().enumerate() {
        let key = (expectation.module_path.as_str(), expectation.class_name);
        if !authored_keys.insert(key) {
            // FALSIFIER: id=linked-emission-authored-expectation-uniqueness class=liveness via=DropComposesReachability producer=can-fail owner=linked-emission-instrument entry=one-expectation-per-module-name
            return Err(format!(
                "duplicate authored liveness expectation in fixture {}, module {}, name {}",
                fixture.id, expectation.module_path, expectation.class_name
            ));
        }
        let Some(module) = modules.get(expectation.module_path.as_str()) else {
            // FALSIFIER: id=linked-emission-authored-build-domain class=liveness via=DropComposesReachability producer=can-fail owner=linked-emission-instrument entry=expectations-name-build-modules
            return Err(format!(
                "authored liveness expectation names a non-build module in fixture {}: {}",
                fixture.id, expectation.module_path
            ));
        };
        let declared = declared_class_names_v0(module.source.as_str(), module.dialect)
            .contains(expectation.class_name);
        if !declared {
            // FALSIFIER: id=linked-emission-authored-declaration-domain class=liveness via=DropComposesReachability producer=can-fail owner=linked-emission-instrument entry=expectations-name-declared-classes
            return Err(format!(
                "authored liveness expectation names an undeclared class in fixture {}, module {}, name {}",
                fixture.id, expectation.module_path, expectation.class_name
            ));
        }
        let actual_live = live_declared_names_by_module
            .get(expectation.module_path.as_str())
            .is_some_and(|names| names.contains(expectation.class_name));
        let mut expected_live = expectation.verdict == LinkedEmissionLivenessVerdictV0::Live;
        if perturbation
            == LinkedEmissionByteDifferentialPerturbationV0::FlipAuthoredLivenessExpectation
            && fixture.id == "two-hop-composes-liveness"
            && expectation_index == 0
        {
            expected_live = !expected_live;
        }
        if actual_live != expected_live {
            // FALSIFIER: id=linked-emission-authored-liveness-equality class=liveness via=DropComposesReachability producer=can-fail owner=linked-emission-instrument entry=authored-and-derived-liveness-agree
            return Err(format!(
                "authored liveness expectation disagrees in fixture {}, module {}, name {}: expected {:?} because {:?}, actual {}",
                fixture.id,
                expectation.module_path,
                expectation.class_name,
                if expected_live {
                    LinkedEmissionLivenessVerdictV0::Live
                } else {
                    LinkedEmissionLivenessVerdictV0::Dead
                },
                expectation.reason,
                if actual_live { "Live" } else { "Dead" }
            ));
        }
    }
    let authored_keys = authored_keys
        .into_iter()
        .map(|(module_path, class_name)| (module_path.to_string(), class_name.to_string()))
        .collect::<BTreeSet<_>>();
    if authored_keys != declared_keys {
        let missing = declared_keys
            .difference(&authored_keys)
            .map(|(module_path, class_name)| format!("{module_path}::{class_name}"))
            .collect::<Vec<_>>();
        let unexpected = authored_keys
            .difference(&declared_keys)
            .map(|(module_path, class_name)| format!("{module_path}::{class_name}"))
            .collect::<Vec<_>>();
        // FALSIFIER: id=linked-emission-authored-totality class=liveness via=DropComposesReachability producer=can-fail owner=linked-emission-instrument entry=one-expectation-per-declared-class
        return Err(format!(
            "authored liveness expectation table is not total for fixture {}: missing [{}], unexpected [{}]",
            fixture.id,
            missing.join(", "),
            unexpected.join(", ")
        ));
    }
    Ok(())
}

fn rule_declaration_counts_for_class_v0(source: &str, class_name: &str) -> BTreeMap<String, usize> {
    let parsed = parse(source, StyleDialect::Css);
    let facts = facts_from_cst(source, &parsed);
    let cst = parsed.cst();
    let rules = cst.rules();
    let mut declarations = BTreeMap::new();
    for declaration in cst.declarations() {
        let declaration_range = declaration.text_range();
        let Some(rule) = rules
            .iter()
            .filter(|rule| {
                let rule_range = rule.text_range();
                rule_range.start() <= declaration_range.start()
                    && declaration_range.end() <= rule_range.end()
            })
            .min_by_key(|rule| u32::from(rule.text_range().len()))
        else {
            continue;
        };
        let rule_range = rule.text_range();
        let declaration_start = u32::from(declaration_range.start()) as usize;
        let declaration_end = u32::from(declaration_range.end()) as usize;
        let Some(declaration_source) = source.get(declaration_start..declaration_end) else {
            continue;
        };
        let carries_class = facts.selectors.iter().any(|selector| {
            selector.kind == ParsedSelectorFactKind::Class
                && selector.name == class_name
                && rule_range.start() <= selector.range.start()
                && selector.range.end() <= rule_range.end()
        });
        if !carries_class {
            continue;
        }
        let declaration = remove_ascii_whitespace_v0(declaration_source);
        if declaration.is_empty() {
            continue;
        }
        *declarations.entry(declaration).or_default() += 1;
    }
    declarations
}

fn remove_first_live_linked_declaration_for_fault_v0(
    fixture: &LinkedEmissionFixtureV0,
    linked_css: &mut String,
    attribution: Option<&OmenaQueryModuleReachabilityAttributionReportV0>,
    rewrites_by_module: &BTreeMap<String, BTreeMap<String, String>>,
) -> Result<(), String> {
    let Some(attribution) = attribution else {
        // FALSIFIER: id=linked-emission-loss-fault-attribution class=shaking via=DropLiveDeclaration producer=can-fail owner=linked-emission-instrument entry=fault-target-has-attribution
        return Err(format!(
            "live-declaration fault lacks attribution for fixture {}",
            fixture.id
        ));
    };
    let live_declared = live_declared_names_by_module_v0(fixture, attribution)?;
    for module in &fixture.modules {
        let Some(names) = live_declared.get(module.path.as_str()) else {
            continue;
        };
        let Some(name) = names.iter().next() else {
            continue;
        };
        let token = emitted_class_name_v0(rewrites_by_module, module.path.as_str(), name);
        let declarations =
            rule_declaration_counts_for_class_v0(module.source.as_str(), name.as_str());
        for declaration in declarations.keys() {
            let spaced = declaration
                .split_once(':')
                .map(|(property, value)| format!("{property}: {value}"))
                .unwrap_or_else(|| declaration.clone());
            for candidate in [spaced.as_str(), declaration.as_str()] {
                let changed = linked_css.replacen(candidate, "", 1);
                if changed != *linked_css {
                    *linked_css = changed;
                    return Ok(());
                }
            }
        }
        // FALSIFIER: id=linked-emission-loss-fault-output class=shaking via=DropLiveDeclaration producer=can-fail owner=linked-emission-instrument entry=fault-target-declaration-present
        return Err(format!(
            "live-declaration fault could not find output for fixture {}, module {}, name {}, token {}",
            fixture.id, module.path, name, token
        ));
    }
    Err(format!(
        "live-declaration fault found no live declared name in fixture {}",
        fixture.id
    ))
}

fn remove_linked_declaration_for_fault_v0(
    linked_css: &mut String,
    declaration: &str,
) -> Result<(), String> {
    let changed = linked_css.replacen(declaration, "", 1);
    if changed == *linked_css {
        // FALSIFIER: id=linked-emission-rust-015 class=accounting via=DropReachableCrossModuleDeclaration producer=can-fail owner=linked-emission-instrument entry=committed-corpus-green
        return Err(format!(
            "linked-emission fault could not find declaration {declaration:?}"
        ));
    }
    *linked_css = changed;
    Ok(())
}

fn remove_ascii_whitespace_v0(source: &str) -> String {
    source
        .chars()
        .filter(|character| !character.is_ascii_whitespace())
        .collect()
}

fn css_url_arguments_v0(source: &str) -> Vec<String> {
    let bytes = source.as_bytes();
    let mut arguments = Vec::new();
    let mut index = 0usize;
    while index + 4 <= bytes.len() {
        if !bytes[index].eq_ignore_ascii_case(&b'u')
            || !bytes[index + 1].eq_ignore_ascii_case(&b'r')
            || !bytes[index + 2].eq_ignore_ascii_case(&b'l')
            || bytes[index + 3] != b'('
        {
            index += 1;
            continue;
        }

        let argument_start = index + 4;
        let mut cursor = argument_start;
        let mut quote = None;
        let mut escaped = false;
        while cursor < bytes.len() {
            let byte = bytes[cursor];
            if escaped {
                escaped = false;
            } else if byte == b'\\' {
                escaped = true;
            } else if let Some(active_quote) = quote {
                if byte == active_quote {
                    quote = None;
                }
            } else if matches!(byte, b'\'' | b'"') {
                quote = Some(byte);
            } else if byte == b')' {
                arguments.push(source[argument_start..cursor].trim().to_string());
                index = cursor + 1;
                break;
            }
            cursor += 1;
        }
        if cursor >= bytes.len() {
            arguments.push(source[argument_start..].trim().to_string());
            break;
        }
    }
    arguments
}

fn sha256_hex_v0(source: &str) -> String {
    let digest = Sha256::digest(source.as_bytes());
    digest.iter().map(|byte| format!("{byte:02x}")).collect()
}

fn liveness_expectation_v0(
    module_path: impl Into<String>,
    class_name: &'static str,
    verdict: LinkedEmissionLivenessVerdictV0,
    reason: LinkedEmissionLivenessReasonV0,
) -> LinkedEmissionLivenessExpectationV0 {
    LinkedEmissionLivenessExpectationV0 {
        module_path: module_path.into(),
        class_name,
        verdict,
        reason,
    }
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
                        order_probe: "color: red".to_string(),
                    },
                    LinkedEmissionFixtureModuleV0 {
                        path: "linked-order-witness/element/reset.css".to_string(),
                        source: "div { color: green; }".to_string(),
                        dialect: StyleDialect::Css,
                        marker_names: Vec::new(),
                        order_probe: "color: green".to_string(),
                    },
                ],
                workspace_only_modules: Vec::new(),
                reachability_references: Vec::new(),
                liveness_expectations: Vec::new(),
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
                        order_probe: "padding: 1px".to_string(),
                    },
                    LinkedEmissionFixtureModuleV0 {
                        path: "linked-order-witness/mixed/reset.css".to_string(),
                        source: "div { color: green; }".to_string(),
                        dialect: StyleDialect::Css,
                        marker_names: Vec::new(),
                        order_probe: "color: green".to_string(),
                    },
                ],
                workspace_only_modules: Vec::new(),
                reachability_references: Vec::new(),
                liveness_expectations: Vec::new(),
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
                        order_probe: "color: red".to_string(),
                    },
                    LinkedEmissionFixtureModuleV0 {
                        path: "linked-order-witness/names/aaa-reset.css".to_string(),
                        source: "div { color: green; }".to_string(),
                        dialect: StyleDialect::Css,
                        marker_names: Vec::new(),
                        order_probe: "color: green".to_string(),
                    },
                ],
                workspace_only_modules: Vec::new(),
                reachability_references: Vec::new(),
                liveness_expectations: Vec::new(),
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
                        order_probe: "color: orange".to_string(),
                    },
                    LinkedEmissionFixtureModuleV0 {
                        path: "linked-order-witness/layers/layers.css".to_string(),
                        source: "@layer base, theme;".to_string(),
                        dialect: StyleDialect::Css,
                        marker_names: Vec::new(),
                        order_probe: "@layer base, theme;".to_string(),
                    },
                ],
                workspace_only_modules: Vec::new(),
                reachability_references: Vec::new(),
                liveness_expectations: Vec::new(),
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
        module_qualified_reachability_fixture_v0(),
        entry_ordinal_skew_fixture_v0(),
        module_qualified_composes_reachability_fixture_v0(),
        module_qualified_at_rule_reachability_fixture_v0(),
        two_hop_composes_liveness_fixture_v0(),
        package_path_composes_liveness_fixture_v0(),
        global_at_rule_liveness_fixture_v0(),
        combinator_companion_liveness_fixture_v0(),
        workspace_only_referrer_liveness_fixture_v0(),
        shaken_referrer_liveness_fixture_v0(),
        selectorless_module_fixture_v0(
            "element-only-reset-module",
            "element-only-reset",
            "reset.css",
            "div { margin: 0; color: green; }",
            "margin: 0",
        ),
        selectorless_module_fixture_v0(
            "bare-layer-statement-module",
            "bare-layer-statement",
            "layers.css",
            "@layer reset;",
            "@layer reset;",
        ),
        selectorless_module_fixture_v0(
            "font-face-only-module",
            "font-face-only",
            "fonts.css",
            "@font-face { font-family: \"OmenaFixture\"; src: url(\"./font.woff2\"); }",
            "OmenaFixture",
        ),
        selectorless_module_fixture_v0(
            "empty-module-boundary",
            "empty-module",
            "empty.css",
            "",
            "",
        ),
        selectorless_module_fixture_v0(
            "comment-only-module-boundary",
            "comment-only-module",
            "license.css",
            "/* Omena fixture license */",
            "Omena fixture license",
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
    imported_order_probe: &str,
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
                marker_names: vec![entry_marker.clone()],
                order_probe: entry_marker,
            },
            LinkedEmissionFixtureModuleV0 {
                path: format!("{root}/{imported_file_name}"),
                source: imported_source.to_string(),
                dialect: StyleDialect::Css,
                marker_names: Vec::new(),
                order_probe: imported_order_probe.to_string(),
            },
        ],
        workspace_only_modules: Vec::new(),
        reachability_references: Vec::new(),
        liveness_expectations: Vec::new(),
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
                marker_names: vec![before.clone(), after],
                order_probe: before,
            },
            LinkedEmissionFixtureModuleV0 {
                path: format!("{root}/a.{extension}"),
                source: format!(".{a_marker} {{ color: blue; }}"),
                dialect,
                marker_names: vec![a_marker.clone()],
                order_probe: a_marker,
            },
            LinkedEmissionFixtureModuleV0 {
                path: format!("{root}/z.{extension}"),
                source: format!(".{z_marker} {{ color: green; }}"),
                dialect,
                marker_names: vec![z_marker.clone()],
                order_probe: z_marker,
            },
        ],
        workspace_only_modules: Vec::new(),
        reachability_references: Vec::new(),
        liveness_expectations: Vec::new(),
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
                order_probe: "linked-shared-entry-before".to_string(),
            },
            LinkedEmissionFixtureModuleV0 {
                path: format!("{root}/left.css"),
                source: ".linked-shared-left-before { color: blue; } @import \"./tokens.css\"; .linked-shared-left-after { color: navy; }".to_string(),
                dialect: StyleDialect::Css,
                marker_names: vec![
                    "linked-shared-left-before".to_string(),
                    "linked-shared-left-after".to_string(),
                ],
                order_probe: "linked-shared-left-before".to_string(),
            },
            LinkedEmissionFixtureModuleV0 {
                path: format!("{root}/right.css"),
                source: ".linked-shared-right { color: teal; } @import \"./tokens.css\";".to_string(),
                dialect: StyleDialect::Css,
                marker_names: vec!["linked-shared-right".to_string()],
                order_probe: "linked-shared-right".to_string(),
            },
            LinkedEmissionFixtureModuleV0 {
                path: format!("{root}/tokens.css"),
                source: ".linked-shared-token { color: purple; }".to_string(),
                dialect: StyleDialect::Css,
                marker_names: vec!["linked-shared-token".to_string()],
                order_probe: "linked-shared-token".to_string(),
            },
        ],
        workspace_only_modules: Vec::new(),
        reachability_references: Vec::new(),
        liveness_expectations: Vec::new(),
    }
}

fn module_qualified_reachability_fixture_v0() -> LinkedEmissionFixtureV0 {
    let root = "linked-byte/module-qualified-reachability";
    let entry_path = format!("{root}/app.module.css");
    LinkedEmissionFixtureV0 {
        id: "module-qualified-reachability".to_string(),
        entry_path: entry_path.clone(),
        shape_classes: vec!["module-qualified-reachability"],
        modules: vec![
            LinkedEmissionFixtureModuleV0 {
                path: entry_path,
                source: "@import \"./dependency.module.css\"; .shared { color: red; } .entry-marker { border: 0; } .entry-dead { color: tan; }"
                    .to_string(),
                dialect: StyleDialect::Css,
                marker_names: vec!["entry-marker".to_string()],
                order_probe: "color: red".to_string(),
            },
            LinkedEmissionFixtureModuleV0 {
                path: format!("{root}/dependency.module.css"),
                source: ".shared { padding: 8px; } .dependency-own { color: blue; } .dependency-dead { color: gray; }"
                    .to_string(),
                dialect: StyleDialect::Css,
                marker_names: vec!["dependency-own".to_string()],
                order_probe: "color: blue".to_string(),
            },
        ],
        workspace_only_modules: vec![LinkedEmissionFixtureModuleV0 {
            path: format!("{root}/workspace-only.module.css"),
            source: ".workspace-only { color: purple; }".to_string(),
            dialect: StyleDialect::Css,
            marker_names: Vec::new(),
            order_probe: String::new(),
        }],
        reachability_references: vec![
            LinkedEmissionReachabilityReferenceV0 {
                id: "entry-reference",
                module_index: 0,
                class_name: "shared",
            },
            LinkedEmissionReachabilityReferenceV0 {
                id: "entry-marker-reference",
                module_index: 0,
                class_name: "entry-marker",
            },
            LinkedEmissionReachabilityReferenceV0 {
                id: "dependency-reference",
                module_index: 1,
                class_name: "dependency-own",
            },
            LinkedEmissionReachabilityReferenceV0 {
                id: "workspace-only-reference",
                module_index: 2,
                class_name: "workspace-only",
            },
        ],
        liveness_expectations: vec![
            liveness_expectation_v0(
                format!("{root}/app.module.css"),
                "shared",
                LinkedEmissionLivenessVerdictV0::Live,
                LinkedEmissionLivenessReasonV0::DirectReference,
            ),
            liveness_expectation_v0(
                format!("{root}/app.module.css"),
                "entry-marker",
                LinkedEmissionLivenessVerdictV0::Live,
                LinkedEmissionLivenessReasonV0::DirectReference,
            ),
            liveness_expectation_v0(
                format!("{root}/app.module.css"),
                "entry-dead",
                LinkedEmissionLivenessVerdictV0::Dead,
                LinkedEmissionLivenessReasonV0::Unreferenced,
            ),
            liveness_expectation_v0(
                format!("{root}/dependency.module.css"),
                "shared",
                LinkedEmissionLivenessVerdictV0::Dead,
                LinkedEmissionLivenessReasonV0::Unreferenced,
            ),
            liveness_expectation_v0(
                format!("{root}/dependency.module.css"),
                "dependency-own",
                LinkedEmissionLivenessVerdictV0::Live,
                LinkedEmissionLivenessReasonV0::DirectReference,
            ),
            liveness_expectation_v0(
                format!("{root}/dependency.module.css"),
                "dependency-dead",
                LinkedEmissionLivenessVerdictV0::Dead,
                LinkedEmissionLivenessReasonV0::Unreferenced,
            ),
        ],
    }
}

fn entry_ordinal_skew_fixture_v0() -> LinkedEmissionFixtureV0 {
    let root = "linked-byte/entry-ordinal-skew";
    let entry_path = format!("{root}/app.module.css");
    let dependency_path = format!("{root}/dependency.module.css");
    LinkedEmissionFixtureV0 {
        id: "entry-ordinal-skew".to_string(),
        entry_path: entry_path.clone(),
        shape_classes: vec!["entry-ordinal-skew"],
        modules: vec![
            LinkedEmissionFixtureModuleV0 {
                path: entry_path.clone(),
                source: "@import \"./dependency.module.css\"; .entry-first { color: red; } .bridge { display: block; } .shared { border: 0; }"
                    .to_string(),
                dialect: StyleDialect::Css,
                marker_names: vec!["entry-first".to_string()],
                order_probe: "color: red".to_string(),
            },
            LinkedEmissionFixtureModuleV0 {
                path: dependency_path.clone(),
                source: ".shared { padding: 4px; } .dependency-own { color: blue; }".to_string(),
                dialect: StyleDialect::Css,
                marker_names: vec!["dependency-own".to_string()],
                order_probe: "padding: 4px".to_string(),
            },
        ],
        workspace_only_modules: Vec::new(),
        reachability_references: Vec::new(),
        liveness_expectations: Vec::new(),
    }
}

fn module_qualified_composes_reachability_fixture_v0() -> LinkedEmissionFixtureV0 {
    let root = "linked-byte/module-qualified-composes";
    let entry_path = format!("{root}/entry.module.css");
    LinkedEmissionFixtureV0 {
        id: "module-qualified-composes-reachability".to_string(),
        entry_path: entry_path.clone(),
        shape_classes: vec!["module-qualified-reachability"],
        modules: vec![
            LinkedEmissionFixtureModuleV0 {
                path: entry_path.clone(),
                source: "@import \"./base.module.css\"; .card { composes: base from \"./base.module.css\"; color: red; }"
                    .to_string(),
                dialect: StyleDialect::Css,
                marker_names: vec!["card".to_string()],
                order_probe: "color: red".to_string(),
            },
            LinkedEmissionFixtureModuleV0 {
                path: format!("{root}/base.module.css"),
                source: ".base { padding: 2px; } .base-live { color: blue; } .base-dead { color: gray; }"
                    .to_string(),
                dialect: StyleDialect::Css,
                marker_names: vec!["base".to_string(), "base-live".to_string()],
                order_probe: "padding: 2px".to_string(),
            },
        ],
        workspace_only_modules: Vec::new(),
        reachability_references: vec![
            LinkedEmissionReachabilityReferenceV0 {
                id: "entry-card-reference",
                module_index: 0,
                class_name: "card",
            },
            LinkedEmissionReachabilityReferenceV0 {
                id: "base-live-reference",
                module_index: 1,
                class_name: "base-live",
            },
        ],
        liveness_expectations: vec![
            liveness_expectation_v0(
                entry_path,
                "card",
                LinkedEmissionLivenessVerdictV0::Live,
                LinkedEmissionLivenessReasonV0::DirectReference,
            ),
            liveness_expectation_v0(
                format!("{root}/base.module.css"),
                "base",
                LinkedEmissionLivenessVerdictV0::Live,
                LinkedEmissionLivenessReasonV0::ComposesFrom(
                    "linked-byte/module-qualified-composes/entry.module.css".to_string(),
                ),
            ),
            liveness_expectation_v0(
                format!("{root}/base.module.css"),
                "base-live",
                LinkedEmissionLivenessVerdictV0::Live,
                LinkedEmissionLivenessReasonV0::DirectReference,
            ),
            liveness_expectation_v0(
                format!("{root}/base.module.css"),
                "base-dead",
                LinkedEmissionLivenessVerdictV0::Dead,
                LinkedEmissionLivenessReasonV0::Unreferenced,
            ),
        ],
    }
}

fn module_qualified_at_rule_reachability_fixture_v0() -> LinkedEmissionFixtureV0 {
    let root = "linked-byte/module-qualified-at-rule";
    let entry_path = format!("{root}/entry.module.css");
    LinkedEmissionFixtureV0 {
        id: "module-qualified-at-rule-reachability".to_string(),
        entry_path: entry_path.clone(),
        shape_classes: vec!["at-rule-nested-liveness"],
        modules: vec![
            LinkedEmissionFixtureModuleV0 {
                path: entry_path,
                source: "@import \"./media.module.css\"; .card { composes: base from \"./media.module.css\"; color: red; }"
                    .to_string(),
                dialect: StyleDialect::Css,
                marker_names: vec!["card".to_string()],
                order_probe: "color: red".to_string(),
            },
            LinkedEmissionFixtureModuleV0 {
                path: format!("{root}/media.module.css"),
                source: ".base { border: 0; } .base-dead { color: gray; } @media (min-width: 40rem) { .media-live { padding: 4px; } .media-dead { padding: 8px; } }"
                    .to_string(),
                dialect: StyleDialect::Css,
                marker_names: vec!["base".to_string(), "media-live".to_string()],
                order_probe: "padding: 4px".to_string(),
            },
        ],
        workspace_only_modules: Vec::new(),
        reachability_references: vec![
            LinkedEmissionReachabilityReferenceV0 {
                id: "card-reference",
                module_index: 0,
                class_name: "card",
            },
            LinkedEmissionReachabilityReferenceV0 {
                id: "media-live-reference",
                module_index: 1,
                class_name: "media-live",
            },
        ],
        liveness_expectations: vec![
            liveness_expectation_v0(
                format!("{root}/entry.module.css"),
                "card",
                LinkedEmissionLivenessVerdictV0::Live,
                LinkedEmissionLivenessReasonV0::DirectReference,
            ),
            liveness_expectation_v0(
                format!("{root}/media.module.css"),
                "base",
                LinkedEmissionLivenessVerdictV0::Live,
                LinkedEmissionLivenessReasonV0::ComposesFrom(
                    "linked-byte/module-qualified-at-rule/entry.module.css".to_string(),
                ),
            ),
            liveness_expectation_v0(
                format!("{root}/media.module.css"),
                "base-dead",
                LinkedEmissionLivenessVerdictV0::Dead,
                LinkedEmissionLivenessReasonV0::Unreferenced,
            ),
            liveness_expectation_v0(
                format!("{root}/media.module.css"),
                "media-live",
                LinkedEmissionLivenessVerdictV0::Live,
                LinkedEmissionLivenessReasonV0::AtRuleNested,
            ),
            liveness_expectation_v0(
                format!("{root}/media.module.css"),
                "media-dead",
                LinkedEmissionLivenessVerdictV0::Dead,
                LinkedEmissionLivenessReasonV0::AtRuleNested,
            ),
        ],
    }
}

fn two_hop_composes_liveness_fixture_v0() -> LinkedEmissionFixtureV0 {
    let root = "linked-byte/two-hop-composes";
    let entry_path = format!("{root}/entry.module.css");
    let mid_path = format!("{root}/mid.module.css");
    let base_path = format!("{root}/base.module.css");
    LinkedEmissionFixtureV0 {
        id: "two-hop-composes-liveness".to_string(),
        entry_path: entry_path.clone(),
        shape_classes: vec!["two-hop-composes-liveness"],
        modules: vec![
            LinkedEmissionFixtureModuleV0 {
                path: entry_path.clone(),
                source: "@import \"./mid.module.css\"; .card { composes: mid from \"./mid.module.css\"; color: red; } .card-dead { color: tan; }"
                    .to_string(),
                dialect: StyleDialect::Css,
                marker_names: vec!["card".to_string()],
                order_probe: "color: red".to_string(),
            },
            LinkedEmissionFixtureModuleV0 {
                path: mid_path.clone(),
                source: "@import \"./base.module.css\"; .mid { composes: base from \"./base.module.css\"; padding: 2px; } .mid-dead { color: gray; }"
                    .to_string(),
                dialect: StyleDialect::Css,
                marker_names: vec!["mid".to_string()],
                order_probe: "padding: 2px".to_string(),
            },
            LinkedEmissionFixtureModuleV0 {
                path: base_path.clone(),
                source: ".base { border: 0; } .base-dead { color: silver; }".to_string(),
                dialect: StyleDialect::Css,
                marker_names: vec!["base".to_string()],
                order_probe: "border: 0".to_string(),
            },
        ],
        workspace_only_modules: Vec::new(),
        reachability_references: vec![LinkedEmissionReachabilityReferenceV0 {
            id: "two-hop-card-reference",
            module_index: 0,
            class_name: "card",
        }],
        liveness_expectations: vec![
            liveness_expectation_v0(
                entry_path.clone(),
                "card",
                LinkedEmissionLivenessVerdictV0::Live,
                LinkedEmissionLivenessReasonV0::DirectReference,
            ),
            liveness_expectation_v0(
                entry_path,
                "card-dead",
                LinkedEmissionLivenessVerdictV0::Dead,
                LinkedEmissionLivenessReasonV0::Unreferenced,
            ),
            liveness_expectation_v0(
                mid_path.clone(),
                "mid",
                LinkedEmissionLivenessVerdictV0::Live,
                LinkedEmissionLivenessReasonV0::ComposesFrom(
                    "linked-byte/two-hop-composes/entry.module.css".to_string(),
                ),
            ),
            liveness_expectation_v0(
                mid_path,
                "mid-dead",
                LinkedEmissionLivenessVerdictV0::Dead,
                LinkedEmissionLivenessReasonV0::Unreferenced,
            ),
            liveness_expectation_v0(
                base_path.clone(),
                "base",
                LinkedEmissionLivenessVerdictV0::Live,
                LinkedEmissionLivenessReasonV0::ComposesFrom(
                    "linked-byte/two-hop-composes/mid.module.css".to_string(),
                ),
            ),
            liveness_expectation_v0(
                base_path,
                "base-dead",
                LinkedEmissionLivenessVerdictV0::Dead,
                LinkedEmissionLivenessReasonV0::Unreferenced,
            ),
        ],
    }
}

fn package_path_composes_liveness_fixture_v0() -> LinkedEmissionFixtureV0 {
    let root = "linked-byte/package-composes";
    let entry_path = format!("{root}/entry.module.css");
    let package_path = "node_modules/@acme/theme/utility.module.css".to_string();
    LinkedEmissionFixtureV0 {
        id: "package-path-composes-liveness".to_string(),
        entry_path: entry_path.clone(),
        shape_classes: vec!["package-path-composes-liveness"],
        modules: vec![
            LinkedEmissionFixtureModuleV0 {
                path: entry_path.clone(),
                source: "@import \"../../node_modules/@acme/theme/utility.module.css\"; .card { composes: utility from \"../../node_modules/@acme/theme/utility.module.css\"; color: red; }"
                    .to_string(),
                dialect: StyleDialect::Css,
                marker_names: vec!["card".to_string()],
                order_probe: "color: red".to_string(),
            },
            LinkedEmissionFixtureModuleV0 {
                path: package_path.clone(),
                source: ".utility { padding: 4px; } .utility-dead { padding: 8px; }".to_string(),
                dialect: StyleDialect::Css,
                marker_names: vec!["utility".to_string()],
                order_probe: "padding: 4px".to_string(),
            },
        ],
        workspace_only_modules: Vec::new(),
        reachability_references: vec![LinkedEmissionReachabilityReferenceV0 {
            id: "package-card-reference",
            module_index: 0,
            class_name: "card",
        }],
        liveness_expectations: vec![
            liveness_expectation_v0(
                entry_path,
                "card",
                LinkedEmissionLivenessVerdictV0::Live,
                LinkedEmissionLivenessReasonV0::DirectReference,
            ),
            liveness_expectation_v0(
                package_path.clone(),
                "utility",
                LinkedEmissionLivenessVerdictV0::Live,
                LinkedEmissionLivenessReasonV0::ComposesFrom(
                    "linked-byte/package-composes/entry.module.css".to_string(),
                ),
            ),
            liveness_expectation_v0(
                package_path,
                "utility-dead",
                LinkedEmissionLivenessVerdictV0::Dead,
                LinkedEmissionLivenessReasonV0::Unreferenced,
            ),
        ],
    }
}

fn global_at_rule_liveness_fixture_v0() -> LinkedEmissionFixtureV0 {
    let root = "linked-byte/global-at-rule";
    let entry_path = format!("{root}/entry.module.css");
    let feature_path = format!("{root}/features.module.css");
    LinkedEmissionFixtureV0 {
        id: "global-at-rule-liveness".to_string(),
        entry_path: entry_path.clone(),
        shape_classes: vec!["global-at-rule-liveness", "at-rule-nested-liveness"],
        modules: vec![
            LinkedEmissionFixtureModuleV0 {
                path: entry_path.clone(),
                source: "@import \"./features.module.css\"; .entry { color: red; }".to_string(),
                dialect: StyleDialect::Css,
                marker_names: vec!["entry".to_string()],
                order_probe: "color: red".to_string(),
            },
            LinkedEmissionFixtureModuleV0 {
                path: feature_path.clone(),
                source: ":global(.global-root) { color: black; } @media (min-width: 1px) { :global(.nested-global) { color: navy; } .media-live { padding: 1px; } .media-dead { padding: 2px; } } @supports (display: grid) { .supports-live { display: grid; } .supports-dead { display: block; } } @layer components { .layer-live { color: blue; } .layer-dead { color: gray; } }"
                    .to_string(),
                dialect: StyleDialect::Css,
                marker_names: vec![
                    "media-live".to_string(),
                    "supports-live".to_string(),
                    "layer-live".to_string(),
                ],
                order_probe: "padding: 1px".to_string(),
            },
        ],
        workspace_only_modules: Vec::new(),
        reachability_references: vec![
            LinkedEmissionReachabilityReferenceV0 {
                id: "global-at-rule-entry-reference",
                module_index: 0,
                class_name: "entry",
            },
            LinkedEmissionReachabilityReferenceV0 {
                id: "media-live-reference",
                module_index: 1,
                class_name: "media-live",
            },
            LinkedEmissionReachabilityReferenceV0 {
                id: "supports-live-reference",
                module_index: 1,
                class_name: "supports-live",
            },
            LinkedEmissionReachabilityReferenceV0 {
                id: "layer-live-reference",
                module_index: 1,
                class_name: "layer-live",
            },
        ],
        liveness_expectations: vec![
            liveness_expectation_v0(
                entry_path,
                "entry",
                LinkedEmissionLivenessVerdictV0::Live,
                LinkedEmissionLivenessReasonV0::DirectReference,
            ),
            liveness_expectation_v0(
                feature_path.clone(),
                "global-root",
                LinkedEmissionLivenessVerdictV0::Dead,
                LinkedEmissionLivenessReasonV0::GlobalEscape,
            ),
            liveness_expectation_v0(
                feature_path.clone(),
                "nested-global",
                LinkedEmissionLivenessVerdictV0::Dead,
                LinkedEmissionLivenessReasonV0::GlobalEscape,
            ),
            liveness_expectation_v0(
                feature_path.clone(),
                "media-live",
                LinkedEmissionLivenessVerdictV0::Live,
                LinkedEmissionLivenessReasonV0::AtRuleNested,
            ),
            liveness_expectation_v0(
                feature_path.clone(),
                "media-dead",
                LinkedEmissionLivenessVerdictV0::Dead,
                LinkedEmissionLivenessReasonV0::AtRuleNested,
            ),
            liveness_expectation_v0(
                feature_path.clone(),
                "supports-live",
                LinkedEmissionLivenessVerdictV0::Live,
                LinkedEmissionLivenessReasonV0::AtRuleNested,
            ),
            liveness_expectation_v0(
                feature_path.clone(),
                "supports-dead",
                LinkedEmissionLivenessVerdictV0::Dead,
                LinkedEmissionLivenessReasonV0::AtRuleNested,
            ),
            liveness_expectation_v0(
                feature_path.clone(),
                "layer-live",
                LinkedEmissionLivenessVerdictV0::Live,
                LinkedEmissionLivenessReasonV0::AtRuleNested,
            ),
            liveness_expectation_v0(
                feature_path,
                "layer-dead",
                LinkedEmissionLivenessVerdictV0::Dead,
                LinkedEmissionLivenessReasonV0::AtRuleNested,
            ),
        ],
    }
}

fn combinator_companion_liveness_fixture_v0() -> LinkedEmissionFixtureV0 {
    let root = "linked-byte/combinator-companion";
    let entry_path = format!("{root}/entry.module.css");
    let selectors_path = format!("{root}/selectors.module.css");
    LinkedEmissionFixtureV0 {
        id: "combinator-companion-liveness".to_string(),
        entry_path: entry_path.clone(),
        shape_classes: vec!["combinator-companion-liveness"],
        modules: vec![
            LinkedEmissionFixtureModuleV0 {
                path: entry_path.clone(),
                source: "@import \"./selectors.module.css\"; .entry-marker { display: block; }"
                    .to_string(),
                dialect: StyleDialect::Css,
                marker_names: vec!["entry-marker".to_string()],
                order_probe: "display: block".to_string(),
            },
            LinkedEmissionFixtureModuleV0 {
                path: selectors_path.clone(),
                source: ".fixture-marker { --omena-marker: 1; } .live.same { color: red; } .live > .child { padding: 1px; } .live .descendant { margin: 1px; } .orphan { color: gray; }"
                    .to_string(),
                dialect: StyleDialect::Css,
                marker_names: vec!["fixture-marker".to_string()],
                order_probe: "color: red".to_string(),
            },
        ],
        workspace_only_modules: Vec::new(),
        reachability_references: vec![
            LinkedEmissionReachabilityReferenceV0 {
                id: "combinator-entry-reference",
                module_index: 0,
                class_name: "entry-marker",
            },
            LinkedEmissionReachabilityReferenceV0 {
                id: "combinator-marker-reference",
                module_index: 1,
                class_name: "fixture-marker",
            },
            LinkedEmissionReachabilityReferenceV0 {
                id: "combinator-live-reference",
                module_index: 1,
                class_name: "live",
            },
        ],
        liveness_expectations: vec![
            liveness_expectation_v0(
                entry_path,
                "entry-marker",
                LinkedEmissionLivenessVerdictV0::Live,
                LinkedEmissionLivenessReasonV0::DirectReference,
            ),
            liveness_expectation_v0(
                selectors_path.clone(),
                "fixture-marker",
                LinkedEmissionLivenessVerdictV0::Live,
                LinkedEmissionLivenessReasonV0::DirectReference,
            ),
            liveness_expectation_v0(
                selectors_path.clone(),
                "live",
                LinkedEmissionLivenessVerdictV0::Live,
                LinkedEmissionLivenessReasonV0::DirectReference,
            ),
            liveness_expectation_v0(
                selectors_path.clone(),
                "same",
                LinkedEmissionLivenessVerdictV0::Dead,
                LinkedEmissionLivenessReasonV0::CombinatorCompanion,
            ),
            liveness_expectation_v0(
                selectors_path.clone(),
                "child",
                LinkedEmissionLivenessVerdictV0::Dead,
                LinkedEmissionLivenessReasonV0::CombinatorCompanion,
            ),
            liveness_expectation_v0(
                selectors_path.clone(),
                "descendant",
                LinkedEmissionLivenessVerdictV0::Dead,
                LinkedEmissionLivenessReasonV0::CombinatorCompanion,
            ),
            liveness_expectation_v0(
                selectors_path,
                "orphan",
                LinkedEmissionLivenessVerdictV0::Dead,
                LinkedEmissionLivenessReasonV0::Unreferenced,
            ),
        ],
    }
}

fn workspace_only_referrer_liveness_fixture_v0() -> LinkedEmissionFixtureV0 {
    let root = "linked-byte/workspace-only-referrer";
    let entry_path = format!("{root}/entry.module.css");
    let base_path = format!("{root}/base.module.css");
    LinkedEmissionFixtureV0 {
        id: "workspace-only-referrer-liveness".to_string(),
        entry_path: entry_path.clone(),
        shape_classes: vec!["workspace-only-referrer-liveness"],
        modules: vec![
            LinkedEmissionFixtureModuleV0 {
                path: entry_path.clone(),
                source: "@import \"./base.module.css\"; .entry-marker { display: block; }"
                    .to_string(),
                dialect: StyleDialect::Css,
                marker_names: vec!["entry-marker".to_string()],
                order_probe: "display: block".to_string(),
            },
            LinkedEmissionFixtureModuleV0 {
                path: base_path.clone(),
                source: ".base { padding: 3px; } .base-dead { padding: 6px; }".to_string(),
                dialect: StyleDialect::Css,
                marker_names: vec!["base".to_string()],
                order_probe: "padding: 3px".to_string(),
            },
        ],
        workspace_only_modules: vec![LinkedEmissionFixtureModuleV0 {
            path: format!("{root}/workspace/bridge.module.css"),
            source: ".bridge { composes: base from \"../base.module.css\"; color: teal; }"
                .to_string(),
            dialect: StyleDialect::Css,
            marker_names: Vec::new(),
            order_probe: String::new(),
        }],
        reachability_references: vec![
            LinkedEmissionReachabilityReferenceV0 {
                id: "workspace-bridge-reference",
                module_index: 2,
                class_name: "bridge",
            },
            LinkedEmissionReachabilityReferenceV0 {
                id: "workspace-entry-reference",
                module_index: 0,
                class_name: "entry-marker",
            },
        ],
        liveness_expectations: vec![
            liveness_expectation_v0(
                entry_path,
                "entry-marker",
                LinkedEmissionLivenessVerdictV0::Live,
                LinkedEmissionLivenessReasonV0::DirectReference,
            ),
            liveness_expectation_v0(
                base_path.clone(),
                "base",
                LinkedEmissionLivenessVerdictV0::Live,
                LinkedEmissionLivenessReasonV0::WorkspaceOnlyReferrer,
            ),
            liveness_expectation_v0(
                base_path,
                "base-dead",
                LinkedEmissionLivenessVerdictV0::Dead,
                LinkedEmissionLivenessReasonV0::Unreferenced,
            ),
        ],
    }
}

fn shaken_referrer_liveness_fixture_v0() -> LinkedEmissionFixtureV0 {
    let root = "linked-byte/shaken-referrer";
    let entry_path = format!("{root}/entry.module.css");
    let bridge_path = format!("{root}/bridge.module.css");
    let base_path = format!("{root}/base.module.css");
    LinkedEmissionFixtureV0 {
        id: "shaken-referrer-liveness".to_string(),
        entry_path: entry_path.clone(),
        shape_classes: vec!["shaken-referrer-liveness"],
        modules: vec![
            LinkedEmissionFixtureModuleV0 {
                path: entry_path.clone(),
                source: "@import \"./bridge.module.css\"; .entry-live { color: red; }".to_string(),
                dialect: StyleDialect::Css,
                marker_names: vec!["entry-live".to_string()],
                order_probe: "color: red".to_string(),
            },
            LinkedEmissionFixtureModuleV0 {
                path: bridge_path.clone(),
                source: "@import \"./base.module.css\"; .bridge { composes: base from \"./base.module.css\"; color: blue; }"
                    .to_string(),
                dialect: StyleDialect::Css,
                marker_names: vec!["bridge".to_string()],
                order_probe: "color: blue".to_string(),
            },
            LinkedEmissionFixtureModuleV0 {
                path: base_path.clone(),
                source: ".base { padding: 5px; }".to_string(),
                dialect: StyleDialect::Css,
                marker_names: vec!["base".to_string()],
                order_probe: "padding: 5px".to_string(),
            },
        ],
        workspace_only_modules: Vec::new(),
        reachability_references: vec![LinkedEmissionReachabilityReferenceV0 {
            id: "entry-live-reference",
            module_index: 0,
            class_name: "entry-live",
        }],
        liveness_expectations: vec![
            liveness_expectation_v0(
                entry_path,
                "entry-live",
                LinkedEmissionLivenessVerdictV0::Live,
                LinkedEmissionLivenessReasonV0::DirectReference,
            ),
            liveness_expectation_v0(
                bridge_path,
                "bridge",
                LinkedEmissionLivenessVerdictV0::Dead,
                LinkedEmissionLivenessReasonV0::Unreferenced,
            ),
            liveness_expectation_v0(
                base_path,
                "base",
                LinkedEmissionLivenessVerdictV0::Dead,
                LinkedEmissionLivenessReasonV0::ReferrerShakenAway,
            ),
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
        order_probe: "linked-corpus-entry-before".to_string(),
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
            marker_names: vec![marker.clone()],
            order_probe: marker,
        });
    }
    Some(LinkedEmissionFixtureV0 {
        id: "bundler-productization-corpus".to_string(),
        entry_path,
        shape_classes: vec!["large-product-corpus"],
        modules,
        workspace_only_modules: Vec::new(),
        reachability_references: Vec::new(),
        liveness_expectations: Vec::new(),
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

        // FALSIFIER: id=linked-emission-rust-016 class=accounting via=DropReachableCrossModuleDeclaration producer=can-fail owner=linked-emission-instrument entry=committed-corpus-green
        assert!(report.fixture_count >= 3);
        // FALSIFIER: id=linked-emission-rust-017 class=accounting via=DropReachableCrossModuleDeclaration producer=can-fail owner=linked-emission-instrument entry=committed-corpus-green
        assert!(report.total_divergence_count > 0);
        // FALSIFIER: id=linked-emission-rust-018 class=accounting via=DropReachableCrossModuleDeclaration producer=can-fail owner=linked-emission-instrument entry=committed-corpus-green
        assert!(report.cases.iter().all(|case| {
            case.legacy_emission_path == "importInlineLegacy"
                && case.linked_emission_path == "linkedOrder"
                && case.linked_modules_emitted_once
                && case.linked_marker_order == case.authoritative_marker_order
        }));
        Ok(())
    }

    #[test]
    fn legacy_emission_order_matches_the_independent_import_graph() -> Result<(), String> {
        let fixture = module_qualified_reachability_fixture_v0();
        let analysis = analyze_linked_emission_fixture_v0(
            &fixture,
            LinkedEmissionByteDifferentialPerturbationV0::None,
        )?;
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
        let projections =
            project_omena_transform_bundle_linker_and_emission_items(&linker_modules, &[]);
        let import_graph_order = independent_import_graph_module_order_v0(
            &fixture,
            projections.linker_projection().inputs(),
        )?;

        // This oracle derives order from authored imports, not either product
        // path's emission plan. Path sorting therefore cannot satisfy it.
        // FALSIFIER: id=linked-emission-legacy-source-order class=placement via=legacy_emission_order_matches_the_independent_import_graph producer=can-fail owner=linked-emission-instrument entry=module-qualified-import-order
        assert_eq!(
            output_module_order_v0(&fixture, &analysis.legacy_css)?,
            observable_module_order_v0(&fixture, &import_graph_order)
        );
        Ok(())
    }

    #[test]
    fn module_qualified_reachability_converges_across_both_paths() -> Result<(), String> {
        let fixture = module_qualified_reachability_fixture_v0();
        let analysis = analyze_linked_emission_fixture_v0(
            &fixture,
            LinkedEmissionByteDifferentialPerturbationV0::None,
        )?;

        // FALSIFIER: id=linked-emission-rust-019 class=accounting via=DropReachableCrossModuleDeclaration producer=can-fail owner=linked-emission-instrument entry=committed-corpus-green
        assert_eq!(
            analysis.case.difference_class,
            LinkedEmissionByteDifferenceClassV0::Equivalent
        );
        // FALSIFIER: id=linked-emission-rust-020 class=accounting via=DropReachableCrossModuleDeclaration producer=can-fail owner=linked-emission-instrument entry=committed-corpus-green
        assert_eq!(analysis.case.semantic_mismatch_count, 0);
        // FALSIFIER: id=linked-emission-rust-021 class=accounting via=DropReachableCrossModuleDeclaration producer=can-fail owner=linked-emission-instrument entry=committed-corpus-green
        assert!(module_qualified_reachability_converges_v0(
            &fixture,
            &analysis.legacy_css,
            &analysis.linked_css
        ));
        // FALSIFIER: id=linked-emission-rust-022 class=accounting via=DropReachableCrossModuleDeclaration producer=can-fail owner=linked-emission-instrument entry=committed-corpus-green
        assert!(!module_qualified_reachability_converges_v0(
            &fixture,
            &analysis.legacy_css,
            &analysis.linked_css.replace("blue", "black")
        ));
        // FALSIFIER: id=linked-emission-rust-023 class=accounting via=DropReachableCrossModuleDeclaration producer=can-fail owner=linked-emission-instrument entry=committed-corpus-green
        assert!(remove_ascii_whitespace_v0(&analysis.legacy_css).contains("color:blue"));
        // FALSIFIER: id=linked-emission-rust-023b class=accounting via=DropReachableCrossModuleDeclaration producer=can-fail owner=linked-emission-instrument entry=committed-corpus-green
        assert!(remove_ascii_whitespace_v0(&analysis.linked_css).contains("color:blue"));
        // FALSIFIER: id=linked-emission-rust-024 class=accounting via=DropReachableCrossModuleDeclaration producer=can-fail owner=linked-emission-instrument entry=committed-corpus-green
        assert!(!analysis.legacy_css.contains("workspace-only"));
        // FALSIFIER: id=linked-emission-rust-025 class=accounting via=DropReachableCrossModuleDeclaration producer=can-fail owner=linked-emission-instrument entry=committed-corpus-green
        assert!(!analysis.linked_css.contains("workspace-only"));
        Ok(())
    }

    #[test]
    fn live_cross_module_declaration_loss_is_rejected() {
        let fixture = module_qualified_reachability_fixture_v0();
        let analysis = analyze_linked_emission_fixture_v0(
            &fixture,
            LinkedEmissionByteDifferentialPerturbationV0::DropReachableCrossModuleDeclaration,
        );

        // FALSIFIER: id=linked-emission-rust-026 class=liveness via=DropReachableCrossModuleDeclaration producer=can-fail owner=linked-emission-instrument entry=live-cross-module-declaration-preserved
        assert!(matches!(
            analysis,
            Err(error)
                if error.contains("linked emission lost live declaration")
                    && error.contains("dependency-own")
                    && error.contains("color:blue")
        ));
    }

    #[test]
    fn live_declaration_arm_rejects_composed_declaration_removal() {
        let result = summarize_linked_emission_byte_differential_envelope_v0(
            LinkedEmissionByteDifferentialPerturbationV0::DropComposedDeclaration,
        );

        // FALSIFIER: id=linked-emission-rust-027 class=shaking via=DropComposedDeclaration producer=can-fail owner=linked-emission-instrument entry=live-composed-declaration-preserved
        assert!(matches!(
            result,
            Err(error)
                if error.contains("linked emission lost live declaration")
                    && error.contains("padding:2px")
        ));
    }

    #[test]
    fn live_declaration_loss_is_rejected_for_every_in_domain_fixture() {
        let fixtures = linked_emission_fixtures_v0()
            .into_iter()
            .filter(|fixture| !fixture.reachability_references.is_empty())
            .collect::<Vec<_>>();
        // FALSIFIER: id=linked-emission-in-domain-nonempty class=shaking via=DropLiveDeclaration producer=can-fail owner=linked-emission-instrument entry=three-in-domain-fixtures
        assert!(!fixtures.is_empty());
        for fixture in fixtures {
            let result = analyze_linked_emission_fixture_v0(
                &fixture,
                LinkedEmissionByteDifferentialPerturbationV0::DropLiveDeclaration,
            );
            // FALSIFIER: id=linked-emission-live-loss-per-fixture class=shaking via=DropLiveDeclaration producer=can-fail owner=linked-emission-instrument entry=every-in-domain-fixture-rejects-loss
            assert!(matches!(
                result,
                Err(error)
                    if error.contains("linked emission lost live declaration")
                        && error.contains(fixture.id.as_str())
                        && error.contains("module ")
                        && error.contains("name ")
            ));
        }
    }

    #[test]
    fn unclaimed_linked_token_is_rejected_by_token_universe() {
        let result = summarize_linked_emission_byte_differential_envelope_v0(
            LinkedEmissionByteDifferentialPerturbationV0::AddUnclaimedLinkedToken,
        );
        // FALSIFIER: id=linked-emission-unclaimed-token-closure class=shaking via=AddUnclaimedLinkedToken producer=can-fail owner=linked-emission-instrument entry=closed-live-token-universe
        assert!(matches!(
            result,
            Err(error)
                if error.contains("linked-emission live token universe is not closed")
                    && error.contains("unclaimed [_unclaimed_linked_token]")
        ));
    }

    #[test]
    fn authored_composes_expectation_detects_upstream_liveness_loss() {
        let result = summarize_linked_emission_byte_differential_envelope_v0(
            LinkedEmissionByteDifferentialPerturbationV0::DropComposesReachability,
        );

        // FALSIFIER: id=linked-emission-composes-authored-expectation class=liveness via=DropComposesReachability producer=can-fail owner=linked-emission-instrument entry=base-live-through-composes
        assert!(matches!(
            result,
            Err(error)
                if error.contains("authored liveness expectation disagrees")
                    && error.contains("base.module.css")
                    && error.contains("name base")
                    && error.contains("expected Live")
                    && error.contains("actual Dead")
        ));
    }

    #[test]
    fn unattributed_reference_is_excluded_from_module_liveness() -> Result<(), String> {
        let envelope = summarize_linked_emission_byte_differential_envelope_v0(
            LinkedEmissionByteDifferentialPerturbationV0::AddUnattributedReachabilityReference,
        )?;
        let fixture = envelope
            .census
            .live_declaration_fixtures
            .iter()
            .find(|fixture| fixture.fixture_id == "module-qualified-reachability")
            .ok_or_else(|| "module-qualified reachability fixture is absent".to_string())?;
        let dependency = fixture
            .modules
            .iter()
            .find(|module| module.module_path.ends_with("/dependency.module.css"))
            .ok_or_else(|| "dependency module is absent".to_string())?;

        // FALSIFIER: id=linked-emission-unattributed-fan-in class=shaking via=AddUnattributedReachabilityReference producer=can-fail owner=linked-emission-instrument entry=unattributed-dependency-dead-subtracted
        assert!(
            !dependency
                .live_declared_class_names
                .iter()
                .any(|name| name == "dependency-dead")
        );
        Ok(())
    }

    #[test]
    fn at_rule_nested_liveness_uses_linked_tokens_without_legacy_oracle() -> Result<(), String> {
        let fixture = module_qualified_at_rule_reachability_fixture_v0();
        let analysis = analyze_linked_emission_fixture_v0(
            &fixture,
            LinkedEmissionByteDifferentialPerturbationV0::None,
        )?;
        let linked = remove_ascii_whitespace_v0(&analysis.linked_css);

        let media_live_token = emitted_class_name_v0(
            &analysis.class_name_rewrites_by_module,
            "linked-byte/module-qualified-at-rule/media.module.css",
            "media-live",
        );
        // FALSIFIER: id=linked-emission-at-rule-live-token class=shaking via=DropLiveDeclaration producer=can-fail owner=linked-emission-instrument entry=media-live-linked-token-preserved
        assert!(linked.contains(&format!(".{media_live_token}{{padding:4px;}}")));
        // FALSIFIER: id=linked-emission-at-rule-dead-token class=liveness via=DropComposesReachability producer=can-fail owner=linked-emission-instrument entry=media-dead-removed
        assert!(!linked.contains("media-dead"));
        // FALSIFIER: id=linked-emission-at-rule-legacy-oracle class=liveness via=DropComposesReachability producer=can-fail owner=linked-emission-instrument entry=both-paths-remove-dead-declarations
        assert!(!analysis.legacy_css.contains("base-dead"));
        // FALSIFIER: id=linked-emission-at-rule-legacy-media-dead class=liveness via=DropComposesReachability producer=can-fail owner=linked-emission-instrument entry=both-paths-remove-dead-declarations
        assert!(!analysis.legacy_css.contains("media-dead"));
        let live_names = analysis
            .live_declared_names_by_module
            .get("linked-byte/module-qualified-at-rule/media.module.css")
            .cloned()
            .unwrap_or_default();
        // FALSIFIER: id=linked-emission-at-rule-base-live class=liveness via=DropComposesReachability producer=can-fail owner=linked-emission-instrument entry=base-live-through-composes
        assert!(live_names.contains("base"));
        // FALSIFIER: id=linked-emission-at-rule-media-live class=liveness via=DropComposesReachability producer=can-fail owner=linked-emission-instrument entry=media-live-direct-reference
        assert!(live_names.contains("media-live"));
        // FALSIFIER: id=linked-emission-at-rule-base-dead class=liveness via=DropComposesReachability producer=can-fail owner=linked-emission-instrument entry=base-dead-not-live
        assert!(!live_names.contains("base-dead"));
        // FALSIFIER: id=linked-emission-at-rule-media-dead class=liveness via=DropComposesReachability producer=can-fail owner=linked-emission-instrument entry=media-dead-not-live
        assert!(!live_names.contains("media-dead"));
        Ok(())
    }

    #[test]
    fn coverage_census_is_derived_from_an_independent_shape_population() -> Result<(), String> {
        let envelope = summarize_linked_emission_byte_differential_envelope_v0(
            LinkedEmissionByteDifferentialPerturbationV0::None,
        )?;
        let census = envelope.census;

        // FALSIFIER: id=linked-emission-rust-028 class=accounting via=DropReachableCrossModuleDeclaration producer=can-fail owner=linked-emission-instrument entry=committed-corpus-green
        assert_eq!(census.fixture_count, envelope.report.fixture_count);
        // FALSIFIER: id=linked-emission-rust-029 class=accounting via=DropReachableCrossModuleDeclaration producer=can-fail owner=linked-emission-instrument entry=committed-corpus-green
        assert_eq!(
            census.covered_shape_count + census.not_covered_shape_count,
            census.population_count
        );
        // FALSIFIER: id=linked-emission-rust-030 class=accounting via=DropReachableCrossModuleDeclaration producer=can-fail owner=linked-emission-instrument entry=committed-corpus-green
        assert!(census.population_count > census.fixture_count);
        // FALSIFIER: id=linked-emission-rust-031 class=accounting via=DropReachableCrossModuleDeclaration producer=can-fail owner=linked-emission-instrument entry=committed-corpus-green
        assert!(!census.not_covered.is_empty());
        // FALSIFIER: id=linked-emission-rust-032 class=accounting via=DropReachableCrossModuleDeclaration producer=can-fail owner=linked-emission-instrument entry=committed-corpus-green
        assert!(!census.full_corpus_coverage);
        // FALSIFIER: id=linked-emission-rust-033 class=accounting via=DropReachableCrossModuleDeclaration producer=can-fail owner=linked-emission-instrument entry=committed-corpus-green
        assert_eq!(census.coverage_scope, "boundedMultiModuleFixtures");
        // FALSIFIER: id=linked-emission-rust-034 class=accounting via=DropReachableCrossModuleDeclaration producer=can-fail owner=linked-emission-instrument entry=committed-corpus-green
        assert_eq!(
            census.marker_observable_module_count + census.blind_spot_module_count,
            census.module_count
        );
        // FALSIFIER: id=linked-emission-rust-035 class=accounting via=DropReachableCrossModuleDeclaration producer=can-fail owner=linked-emission-instrument entry=committed-corpus-green
        assert_eq!(
            census.fixture_observability.len(),
            envelope.report.fixture_count
        );
        // FALSIFIER: id=linked-emission-rust-036 class=accounting via=DropReachableCrossModuleDeclaration producer=can-fail owner=linked-emission-instrument entry=committed-corpus-green
        assert_eq!(
            census.module_token_collision_count,
            census.module_token_collisions.len()
        );
        // FALSIFIER: id=linked-emission-rust-037 class=accounting via=DropReachableCrossModuleDeclaration producer=can-fail owner=linked-emission-instrument entry=committed-corpus-green
        assert_eq!(
            census.module_token_collision_scope,
            "boundedFixtureRegressionTripwire"
        );
        // FALSIFIER: id=linked-emission-rust-038 class=accounting via=DropReachableCrossModuleDeclaration producer=can-fail owner=linked-emission-instrument entry=committed-corpus-green
        assert_eq!(census.module_token_collision_count, 0);
        // FALSIFIER: id=linked-emission-rust-039 class=accounting via=DropReachableCrossModuleDeclaration producer=can-fail owner=linked-emission-instrument entry=committed-corpus-green
        assert!(census.unmodeled_declarations.is_empty());
        Ok(())
    }

    #[test]
    fn module_qualified_token_census_consumes_emitted_class_rewrites() -> Result<(), String> {
        let fixture = module_qualified_reachability_fixture_v0();
        let analysis = analyze_linked_emission_fixture_v0(
            &fixture,
            LinkedEmissionByteDifferentialPerturbationV0::None,
        )?;
        let summary = summarize_module_token_collisions_v0(&fixture, &analysis);

        // FALSIFIER: id=linked-emission-rust-040 class=accounting via=DropReachableCrossModuleDeclaration producer=can-fail owner=linked-emission-instrument entry=committed-corpus-green
        assert!(summary.collisions.is_empty());
        let emitted_tokens = fixture
            .modules
            .iter()
            .map(|module| {
                analysis
                    .class_name_rewrites_by_module
                    .get(module.path.as_str())
                    .and_then(|rewrites| rewrites.get("shared"))
                    .cloned()
                    .ok_or_else(|| format!("shared rewrite is absent for {}", module.path))
            })
            .collect::<Result<BTreeSet<_>, String>>()?;
        // FALSIFIER: id=linked-emission-rust-041 class=accounting via=DropReachableCrossModuleDeclaration producer=can-fail owner=linked-emission-instrument entry=committed-corpus-green
        assert_eq!(emitted_tokens.len(), fixture.modules.len());
        let entry_token =
            analysis.class_name_rewrites_by_module[fixture.entry_path.as_str()]["shared"].as_str();
        let dependency_path = fixture
            .modules
            .iter()
            .find(|module| module.path != fixture.entry_path)
            .map(|module| module.path.as_str())
            .ok_or_else(|| "dependency module is absent".to_string())?;
        let dependency_token =
            analysis.class_name_rewrites_by_module[dependency_path]["shared"].as_str();
        assert_ne!(entry_token, dependency_token);
        for output in [&analysis.legacy_css, &analysis.linked_css] {
            let selector_counts = output_class_selector_counts_v0(output);
            // FALSIFIER: id=linked-emission-rust-043 class=accounting via=DropReachableCrossModuleDeclaration producer=can-fail owner=linked-emission-instrument entry=committed-corpus-green
            assert_eq!(selector_counts.get(entry_token), Some(&1));
            // FALSIFIER: id=linked-emission-module-token-dead-owner class=liveness via=DropReachableCrossModuleDeclaration producer=can-fail owner=linked-emission-instrument entry=dead-owner-token-absent
            assert_eq!(selector_counts.get(dependency_token), None);
        }
        Ok(())
    }

    #[test]
    fn token_collision_path_gate_rejects_path_specific_selector_loss() -> Result<(), String> {
        let collision = LinkedEmissionModuleTokenCollisionV0 {
            fixture_id: "path-scope-control".to_string(),
            emitted_token: "_forced_shared".to_string(),
            module_paths: vec![
                "src/a.module.css".to_string(),
                "src/b.module.css".to_string(),
            ],
            original_names: vec!["shared".to_string()],
            observed_emission_paths: vec!["importInlineLegacy"],
            path_scope: LinkedEmissionModuleTokenCollisionPathScopeV0::BothPaths,
            reason: "falsifier-only path-scope mismatch".to_string(),
        };

        // FALSIFIER: id=linked-emission-rust-044 class=accounting via=DropReachableCrossModuleDeclaration producer=can-fail owner=linked-emission-instrument entry=committed-corpus-green
        assert!(validate_module_token_collision_paths_v0(&[collision]).is_err());
        Ok(())
    }

    #[test]
    fn marker_blind_modules_are_represented_and_classified_explicitly() -> Result<(), String> {
        let envelope = summarize_linked_emission_byte_differential_envelope_v0(
            LinkedEmissionByteDifferentialPerturbationV0::None,
        )?;
        let expected_fixture_ids = BTreeSet::from([
            "bare-layer-statement-module",
            "comment-only-module-boundary",
            "element-only-reset-module",
            "empty-module-boundary",
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

        // FALSIFIER: id=linked-emission-rust-047 class=accounting via=DropReachableCrossModuleDeclaration producer=can-fail owner=linked-emission-instrument entry=committed-corpus-green
        assert_eq!(
            unexpected_fixture_ids,
            BTreeSet::from(["font-face-only-module"])
        );
        // FALSIFIER: id=linked-emission-rust-048 class=accounting via=DropReachableCrossModuleDeclaration producer=can-fail owner=linked-emission-instrument entry=committed-corpus-green
        assert_eq!(blind_spot_fixture_ids, expected_fixture_ids);
        // FALSIFIER: id=linked-emission-rust-049 class=accounting via=DropReachableCrossModuleDeclaration producer=can-fail owner=linked-emission-instrument entry=committed-corpus-green
        assert!(envelope.census.blind_spots.iter().all(|blind_spot| {
            blind_spot.emission_plan_entry_count > 0
                && blind_spot.marker_orders_agree
                && blind_spot.linked_marker_order_matches_authority
                && !blind_spot.semantic_difference_observed
                && (!blind_spot.output_bytes_differ || blind_spot.difference_reason_observed)
        }));
        // FALSIFIER: id=linked-emission-rust-050 class=accounting via=DropReachableCrossModuleDeclaration producer=can-fail owner=linked-emission-instrument entry=committed-corpus-green
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
                ("comment-only-module-boundary", [].as_slice()),
                (
                    "element-only-reset-module",
                    ["emissionSelectors"].as_slice()
                ),
                ("empty-module-boundary", [].as_slice()),
                ("font-face-only-module", ["atRules"].as_slice()),
            ])
        );
        // FALSIFIER: id=linked-emission-rust-051 class=accounting via=DropReachableCrossModuleDeclaration producer=can-fail owner=linked-emission-instrument entry=committed-corpus-green
        assert!(envelope.census.not_covered.iter().all(|entry| {
            !matches!(
                entry.shape_class.as_str(),
                "bare-layer-statement"
                    | "comment-only-module"
                    | "element-only-reset"
                    | "empty-module"
                    | "font-face-only"
            )
        }));
        Ok(())
    }

    #[test]
    fn asset_url_drift_is_not_laundered_as_module_placement() -> Result<(), String> {
        let fixture = linked_emission_fixtures_v0()
            .into_iter()
            .find(|fixture| fixture.id == "font-face-only-module")
            .ok_or_else(|| "font-face fixture is missing".to_string())?;
        let analysis = analyze_linked_emission_fixture_v0(
            &fixture,
            LinkedEmissionByteDifferentialPerturbationV0::None,
        )?;

        assert_ne!(
            css_url_arguments_v0(&analysis.legacy_css),
            css_url_arguments_v0(&analysis.linked_css)
        );
        // FALSIFIER: id=linked-emission-rust-052 class=accounting via=DropReachableCrossModuleDeclaration producer=can-fail owner=linked-emission-instrument entry=committed-corpus-green
        assert!(analysis.case.semantic_preserved);
        // FALSIFIER: id=linked-emission-rust-053 class=accounting via=DropReachableCrossModuleDeclaration producer=can-fail owner=linked-emission-instrument entry=committed-corpus-green
        assert!(
            analysis
                .case
                .reasons
                .contains(&LinkedEmissionByteDifferenceReasonV0::ImportGraphModulePlacement)
        );
        // FALSIFIER: id=linked-emission-rust-054 class=accounting via=DropReachableCrossModuleDeclaration producer=can-fail owner=linked-emission-instrument entry=committed-corpus-green
        assert_eq!(
            analysis.case.difference_class,
            LinkedEmissionByteDifferenceClassV0::Unexpected
        );
        Ok(())
    }

    #[test]
    fn module_placement_reason_requires_import_graph_order() -> Result<(), String> {
        let fixture = linked_emission_fixtures_v0()
            .into_iter()
            .find(|fixture| fixture.id == "element-only-reset-module")
            .ok_or_else(|| "element-only reset fixture is missing".to_string())?;
        let analysis = analyze_linked_emission_fixture_v0(
            &fixture,
            LinkedEmissionByteDifferentialPerturbationV0::None,
        )?;
        let lexicographic_output = format!(
            ".linked-element-only-reset-module-entry {{ color: red; }}\n{}",
            fixture.modules[1].source
        );
        let lexicographic_module_order = output_module_order_v0(&fixture, &lexicographic_output)?;

        assert_ne!(
            lexicographic_module_order,
            analysis.case.authoritative_module_order
        );
        let linked_output_module_order_matches_authority =
            lexicographic_module_order == analysis.case.authoritative_module_order;
        let difference_observation = LinkedEmissionDifferenceObservationV0 {
            legacy_css: &analysis.legacy_css,
            linked_css: &lexicographic_output,
            authoritative_marker_order: &analysis.case.authoritative_marker_order,
            legacy_marker_order: &analysis.case.legacy_marker_order,
            linked_marker_order: &analysis.case.linked_marker_order,
            linked_output_module_order_matches_authority,
        };
        let reasons =
            derive_difference_reasons_v0(&fixture, &analysis.linked_order, &difference_observation);
        // FALSIFIER: id=linked-emission-rust-055 class=accounting via=DropReachableCrossModuleDeclaration producer=can-fail owner=linked-emission-instrument entry=committed-corpus-green
        assert!(
            !reasons.contains(&LinkedEmissionByteDifferenceReasonV0::ImportGraphModulePlacement)
        );
        Ok(())
    }

    #[test]
    fn placement_witnesses_follow_import_graph_order() -> Result<(), String> {
        let envelope = summarize_linked_emission_byte_differential_envelope_v0(
            LinkedEmissionByteDifferentialPerturbationV0::None,
        )?;
        let witnesses = envelope
            .census
            .placement_witnesses
            .iter()
            .map(|witness| (witness.witness_id.as_str(), witness))
            .collect::<BTreeMap<_, _>>();

        // FALSIFIER: id=linked-emission-rust-056 class=accounting via=DropReachableCrossModuleDeclaration producer=can-fail owner=linked-emission-instrument entry=committed-corpus-green
        assert_eq!(witnesses.len(), 4);
        // FALSIFIER: id=linked-emission-rust-057 class=accounting via=DropReachableCrossModuleDeclaration producer=can-fail owner=linked-emission-instrument entry=committed-corpus-green
        assert!(witnesses.values().all(|witness| {
            !witness.selectorless_module_paths.is_empty()
                && witness.emission_plan_entry_count > 0
                && witness.output_bytes_differ
                && witness.marker_orders_agree
                && witness.linked_marker_order_matches_authority
        }));
        // FALSIFIER: id=linked-emission-rust-058 class=accounting via=DropReachableCrossModuleDeclaration producer=can-fail owner=linked-emission-instrument entry=committed-corpus-green
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
                    ("blue", "blue", "blue", false, true)
                ),
                (
                    "element-selector-after-rule-bearing-module",
                    ("red", "red", "red", false, true)
                ),
                (
                    "element-selector-winner",
                    ("red", "red", "red", false, true)
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

        // FALSIFIER: id=linked-emission-rust-059 class=accounting via=DropReachableCrossModuleDeclaration producer=can-fail owner=linked-emission-instrument entry=committed-corpus-green
        assert!(report.unexpected_divergence_count > 0);
        Ok(())
    }

    #[test]
    fn collapsed_linked_output_is_rejected_by_marker_order_authority() {
        let result = summarize_linked_emission_byte_differential_v0(
            LinkedEmissionByteDifferentialPerturbationV0::CollapseToLegacyBytes,
        );

        // FALSIFIER: id=linked-emission-rust-060 class=accounting via=DropReachableCrossModuleDeclaration producer=can-fail owner=linked-emission-instrument entry=committed-corpus-green
        assert!(matches!(
            result,
            Err(error)
                if error.contains("linked emission collapsed to non-authoritative marker order")
        ));
    }
}

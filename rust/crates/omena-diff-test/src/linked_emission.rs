use std::collections::{BTreeMap, BTreeSet};

use omena_benchmarks::bundler_productization_corpus;
use omena_bundler::{TransformBundleModuleInputV0, link_omena_transform_bundle_modules};
use omena_parser::{StyleDialect, summarize_omena_parser_style_facts};
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
    modules: Vec<LinkedEmissionFixtureModuleV0>,
}

/// Compares legacy and linked emission for the shared corpus.
///
/// The optional perturbation lets callers confirm that unexpected byte changes
/// remain distinguishable from the documented linked-order differences.
pub fn summarize_linked_emission_byte_differential_v0(
    perturbation: LinkedEmissionByteDifferentialPerturbationV0,
) -> Result<LinkedEmissionByteDifferentialReportV0, String> {
    let cases = linked_emission_fixtures_v0()
        .into_iter()
        .enumerate()
        .map(|(index, fixture)| {
            let case_perturbation = match perturbation {
                LinkedEmissionByteDifferentialPerturbationV0::AddUnexpectedRule if index == 0 => {
                    perturbation
                }
                LinkedEmissionByteDifferentialPerturbationV0::CollapseToLegacyBytes => perturbation,
                _ => LinkedEmissionByteDifferentialPerturbationV0::None,
            };
            summarize_linked_emission_fixture_v0(&fixture, case_perturbation)
        })
        .collect::<Result<Vec<_>, _>>()?;
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

    Ok(LinkedEmissionByteDifferentialReportV0 {
        schema_version: "0",
        product: "omena-diff-test.linked-emission-byte-differential",
        fixture_count: cases.len(),
        equivalent_count,
        expected_divergence_count,
        unexpected_divergence_count,
        total_divergence_count: expected_divergence_count + unexpected_divergence_count,
        cases,
    })
}

fn summarize_linked_emission_fixture_v0(
    fixture: &LinkedEmissionFixtureV0,
    perturbation: LinkedEmissionByteDifferentialPerturbationV0,
) -> Result<LinkedEmissionByteDifferentialCaseV0, String> {
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

    Ok(LinkedEmissionByteDifferentialCaseV0 {
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
    })
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

fn linked_emission_fixtures_v0() -> Vec<LinkedEmissionFixtureV0> {
    let mut fixtures = vec![
        dialect_fixture_v0("css", StyleDialect::Css, "@import"),
        dialect_fixture_v0("scss", StyleDialect::Scss, "@import"),
        dialect_fixture_v0("less", StyleDialect::Less, "@import"),
        shared_import_fixture_v0(),
    ];
    if let Some(corpus_fixture) = product_corpus_fixture_v0() {
        fixtures.push(corpus_fixture);
    }
    fixtures
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

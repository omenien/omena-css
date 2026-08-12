use std::collections::{BTreeMap, BTreeSet};

use serde::Deserialize;

use super::super::{
    OmenaQueryClassSitePlaneV0, OmenaQueryClassSiteTypeFactInputV0, OmenaQueryClassSiteValueV0,
    StringTypeFactsV2, build_omena_query_guarded_token_map_for_site,
    resolve_omena_query_class_site_values_for_source,
    resolve_omena_query_class_site_values_for_source_with_type_facts,
};

const FIXTURE_SOURCE: &str =
    include_str!("../../../../../test/_fixtures/guarded-token-map/ClassSitePlane.tsx");
const EXPECTATION_SOURCE: &str = include_str!("../../data/class-site-expectations-v0.json");

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct ExpectationTableV0 {
    rows: Vec<ExpectationRowV0>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct ExpectationRowV0 {
    site_text: String,
    must: Vec<String>,
    may: Vec<String>,
    unknown_cause: Option<String>,
    #[serde(default)]
    type_fact_values: Vec<String>,
    type_fact_refusal_cause: Option<String>,
    #[serde(default)]
    must_with_type_fact: Vec<String>,
    #[serde(default)]
    may_with_type_fact: Vec<String>,
}

#[test]
fn class_site_plane_matches_authored_sites_with_and_without_type_facts()
-> Result<(), Box<dyn std::error::Error>> {
    let expectations = serde_json::from_str::<ExpectationTableV0>(EXPECTATION_SOURCE)?;
    let cfg_only = resolve_omena_query_class_site_values_for_source(
        "test/_fixtures/guarded-token-map/ClassSitePlane.tsx",
        FIXTURE_SOURCE,
        Some("typescriptreact"),
    );
    let expected_sites = expectations
        .rows
        .iter()
        .map(|row| row.site_text.as_str())
        .collect::<BTreeSet<_>>();
    let actual_by_site = sites_by_source_text(FIXTURE_SOURCE, &cfg_only)?;
    let actual_sites = actual_by_site.keys().copied().collect::<BTreeSet<_>>();
    assert_eq!(
        expected_sites
            .difference(&actual_sites)
            .copied()
            .collect::<Vec<_>>(),
        Vec::<&str>::new(),
        "authored class sites missing from product enumerator"
    );
    assert_eq!(
        actual_sites
            .difference(&expected_sites)
            .copied()
            .collect::<Vec<_>>(),
        Vec::<&str>::new(),
        "product enumerator returned unauthored class sites"
    );

    for row in &expectations.rows {
        let site = actual_by_site[&row.site_text.as_str()];
        assert_support(site, &row.must, &row.may, row.site_text.as_str());
        assert_eq!(
            site.unknown_cause
                .as_ref()
                .map(serde_json::to_value)
                .transpose()?
                .and_then(|value| value.as_str().map(str::to_string)),
            row.unknown_cause,
            "unknown cause for {}",
            row.site_text
        );
        assert_eq!(site.type_fact_cause.as_deref(), Some("typeFactNotProvided"));
        if !row.may.is_empty() {
            assert!(
                site.token_provenance
                    .iter()
                    .all(|token| token.planes.contains(&OmenaQueryClassSitePlaneV0::Cfg)),
                "CFG provenance for {}",
                row.site_text
            );
        }
    }

    let guarded = actual_by_site[&"className={clsx({ active: flag })}"];
    let guarded_map = build_omena_query_guarded_token_map_for_site(guarded)?;
    let active = super::super::GuardedTokenLanguageV0::concrete("active");
    assert!(guarded_map.is_may(&active));
    assert!(!guarded_map.is_must(&active));

    let static_site = actual_by_site[&"className=\"root root\""];
    let static_map = build_omena_query_guarded_token_map_for_site(static_site)?;
    let root = super::super::GuardedTokenLanguageV0::concrete("root");
    assert!(static_map.is_must(&root));

    let symbolic_site = actual_by_site[&"className={`tone-${computeClassName()}`}"];
    let symbolic_map = build_omena_query_guarded_token_map_for_site(symbolic_site)?;
    let symbolic = super::super::GuardedTokenLanguageV0::symbolic("`tone-${computeClassName()}`");
    assert!(symbolic_map.is_must(&symbolic));

    let type_facts = expectations
        .rows
        .iter()
        .map(|row| {
            let site = actual_by_site[&row.site_text.as_str()];
            OmenaQueryClassSiteTypeFactInputV0 {
                site_byte_span: site.site_byte_span,
                facts: (!row.type_fact_values.is_empty()).then(|| StringTypeFactsV2 {
                    kind: if row.type_fact_values.len() == 1 {
                        "exact".to_string()
                    } else {
                        "finiteSet".to_string()
                    },
                    values: Some(row.type_fact_values.clone()),
                    constraint_kind: None,
                    prefix: None,
                    suffix: None,
                    min_len: None,
                    max_len: None,
                    char_must: None,
                    char_may: None,
                    may_include_other_chars: None,
                    provenance: Some("authoredTypeFactFixture".to_string()),
                }),
                refusal_cause: row.type_fact_refusal_cause.clone(),
            }
        })
        .collect::<Vec<_>>();
    let joined = resolve_omena_query_class_site_values_for_source_with_type_facts(
        "test/_fixtures/guarded-token-map/ClassSitePlane.tsx",
        FIXTURE_SOURCE,
        Some("typescriptreact"),
        &type_facts,
    );
    let joined_by_site = sites_by_source_text(FIXTURE_SOURCE, &joined)?;
    for row in &expectations.rows {
        let site = joined_by_site[&row.site_text.as_str()];
        assert_support(
            site,
            &row.must_with_type_fact,
            &row.may_with_type_fact,
            row.site_text.as_str(),
        );
        assert_eq!(
            site.type_fact_cause, row.type_fact_refusal_cause,
            "type-fact cause for {}",
            row.site_text
        );
        if !row.type_fact_values.is_empty() && !row.may.is_empty() {
            assert!(
                site.token_provenance.iter().all(|token| {
                    token.planes.contains(&OmenaQueryClassSitePlaneV0::TypeFact)
                        && token.planes.contains(&OmenaQueryClassSitePlaneV0::Joined)
                }),
                "joined provenance for {}",
                row.site_text
            );
        }
    }
    Ok(())
}

fn sites_by_source_text<'a>(
    source: &'a str,
    sites: &'a [OmenaQueryClassSiteValueV0],
) -> Result<BTreeMap<&'a str, &'a OmenaQueryClassSiteValueV0>, String> {
    sites
        .iter()
        .map(|site| {
            source
                .get(site.site_byte_span.start..site.site_byte_span.end)
                .map(|text| (text, site))
                .ok_or_else(|| format!("site span is outside fixture: {:?}", site.site_byte_span))
        })
        .collect()
}

fn assert_support(
    site: &OmenaQueryClassSiteValueV0,
    expected_must: &[String],
    expected_may: &[String],
    label: &str,
) {
    let (actual_must, actual_may) = site
        .support
        .as_ref()
        .map(|support| {
            (
                support
                    .must()
                    .iter()
                    .map(|token| token.as_str())
                    .collect::<Vec<_>>(),
                support
                    .may()
                    .iter()
                    .map(|token| token.as_str())
                    .collect::<Vec<_>>(),
            )
        })
        .unwrap_or_default();
    assert_eq!(actual_must, expected_must, "must support for {label}");
    assert_eq!(actual_may, expected_may, "may support for {label}");
}

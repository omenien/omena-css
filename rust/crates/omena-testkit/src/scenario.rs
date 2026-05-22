use serde::Serialize;

use crate::fixture::{CmeFixtureFileV0, CmeFixtureV0, parse_cme_fixture_v0};

/// v0.1 scenario archetypes supported by the Rust testkit substrate.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum CmeScenarioArchetypeV0 {
    /// Boundary checks that validate a product surface without executing a
    /// transform pipeline.
    Boundary,
    /// Transform-execution checks that require a style input and a transform
    /// product expectation.
    TransformExecute,
    /// LSP request/response scenarios with source, style, and position markers.
    LspScenario,
    /// Shadow runner scenarios exposing `shadow.omena(<verb>)` intent.
    ShadowOmenaVerb,
}

impl CmeScenarioArchetypeV0 {
    /// Stable archetype identifier used by reports and downstream adapters.
    pub const fn id(self) -> &'static str {
        match self {
            Self::Boundary => "boundary",
            Self::TransformExecute => "transform-execute",
            Self::LspScenario => "lsp-scenario",
            Self::ShadowOmenaVerb => "shadow-omena-verb",
        }
    }
}

/// Readiness evidence for one `cme-fixture-v0` scenario.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CmeScenarioReadinessV0 {
    /// Whether the fixture parsed successfully before scenario classification.
    pub fixture_parses: bool,
    /// Whether the fixture declares at least one product expectation.
    pub has_expected_products: bool,
    /// Whether the fixture has the files required by the archetype.
    pub has_required_files: bool,
    /// Whether the fixture has the markers required by the archetype.
    pub has_required_markers: bool,
    /// Whether the fixture exposes the product or introspection surface expected
    /// by the archetype.
    pub has_supported_introspection: bool,
    /// Whether this scenario is ready for downstream runner adoption.
    pub ready: bool,
    /// Reasons preventing runner adoption.
    pub unsupported_reasons: Vec<&'static str>,
}

/// Rust call-site evidence captured when a scenario is built through the macro.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CmeScenarioCallSiteV0 {
    /// Source file where the scenario macro was invoked.
    pub file: &'static str,
    /// 1-based source line where the scenario macro was invoked.
    pub line: u32,
    /// 1-based source column where the scenario macro was invoked.
    pub column: u32,
}

/// Scenario summary produced by the v0.1 testkit scenario macro substrate.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CmeScenarioV0 {
    /// Schema version.
    pub schema_version: &'static str,
    /// Product surface.
    pub product: &'static str,
    /// Fixture grammar consumed by this scenario.
    pub fixture_grammar: &'static str,
    /// Scenario archetype.
    pub archetype: CmeScenarioArchetypeV0,
    /// Stable archetype identifier.
    pub archetype_id: &'static str,
    /// Parsed file count.
    pub file_count: usize,
    /// Parsed source file count.
    pub source_file_count: usize,
    /// Parsed style file count.
    pub style_file_count: usize,
    /// Parsed expectation count.
    pub expectation_count: usize,
    /// Parsed marker count.
    pub marker_count: usize,
    /// Parsed metadata count.
    pub metadata_count: usize,
    /// Product expectations declared by the fixture.
    pub expected_products: Vec<String>,
    /// Extracted `shadow.omena(<verb>)` introspection verbs.
    pub shadow_omena_verbs: Vec<String>,
    /// Readiness evidence.
    pub readiness: CmeScenarioReadinessV0,
    /// Macro call-site evidence, present only for macro-built scenarios.
    pub call_site: Option<CmeScenarioCallSiteV0>,
}

/// One built-in scenario macro seed.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OmenaTestkitScenarioMacroSeedReportV0 {
    /// Stable seed label.
    pub label: &'static str,
    /// Scenario summary.
    pub scenario: CmeScenarioV0,
}

/// Boundary evidence that the scenario macro preserves call-site location data.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OmenaTestkitScenarioMacroCallSiteProbeV0 {
    /// Whether the probe scenario carried call-site evidence.
    pub present: bool,
    /// Whether the captured file location points at the testkit source file.
    pub file_suffix_matches: bool,
    /// Whether the captured line is non-zero.
    pub line_non_zero: bool,
    /// Whether the captured column is non-zero.
    pub column_non_zero: bool,
}

/// Built-in scenario macro substrate report.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OmenaTestkitScenarioMacroReportV0 {
    /// Schema version.
    pub schema_version: &'static str,
    /// Product surface.
    pub product: &'static str,
    /// Fixture grammar.
    pub fixture_grammar: &'static str,
    /// Supported archetype identifiers.
    pub supported_archetypes: Vec<&'static str>,
    /// Scenario seed count.
    pub scenario_count: usize,
    /// Whether every built-in scenario seed is ready.
    pub all_scenario_macros_ready: bool,
    /// Whether macro expansion records Rust call-site evidence.
    pub call_site_evidence_supported: bool,
    /// Macro call-site evidence probe.
    pub call_site_probe: OmenaTestkitScenarioMacroCallSiteProbeV0,
    /// Built-in scenario seed reports.
    pub reports: Vec<OmenaTestkitScenarioMacroSeedReportV0>,
}

struct OmenaTestkitScenarioMacroSeedV0 {
    label: &'static str,
    archetype: CmeScenarioArchetypeV0,
    raw: &'static str,
}

const SCENARIO_MACRO_SEEDS: &[OmenaTestkitScenarioMacroSeedV0] = &[
    OmenaTestkitScenarioMacroSeedV0 {
        label: "boundary-product-scenario",
        archetype: CmeScenarioArchetypeV0::Boundary,
        raw: r#"--- file: src/Button.module.scss
.button { color: red; }
--- expect: product
omena-parser.style-facts
--- expect: assertion
boundary scenario keeps the product expectation and workspace file together
"#,
    },
    OmenaTestkitScenarioMacroSeedV0 {
        label: "transform-execute-scenario",
        archetype: CmeScenarioArchetypeV0::TransformExecute,
        raw: r#"//- src/Button.module.scss dialect:scss layer:style
.button { color: light-dark(red, blue); }
--- expect: product
omena-query.transform-execute
--- expect: assertion
transform-execute scenario carries a style fixture and transform product
"#,
    },
    OmenaTestkitScenarioMacroSeedV0 {
        label: "lsp-hover-scenario",
        archetype: CmeScenarioArchetypeV0::LspScenario,
        raw: r#"--- file: src/App.tsx
import styles from "./Button.module.scss";
export const App = () => <div className={styles./*|*/button} />;
--- file: src/Button.module.scss
.button { color: red; }
--- expect: product
omena-lsp-server.hover
--- expect: assertion
lsp scenario carries source, style, and request-position marker evidence
"#,
    },
    OmenaTestkitScenarioMacroSeedV0 {
        label: "shadow-omena-verb-scenario",
        archetype: CmeScenarioArchetypeV0::ShadowOmenaVerb,
        raw: r#"--- file: src/App.tsx
import styles from "./Button.module.scss";
const cls = styles.button;
--- file: src/Button.module.scss
.button { color: red; }
--- expect: product
shadow.omena.hover
--- expect: product
shadow.omena.definition
--- expect: assertion
shadow scenario exposes product-facing omena verbs without binding to one runner
"#,
    },
];

/// Build a v0.1 scenario summary from a `cme-fixture-v0` fixture.
pub fn summarize_cme_scenario_v0(
    raw: &str,
    archetype: CmeScenarioArchetypeV0,
) -> Result<CmeScenarioV0, String> {
    let fixture = parse_cme_fixture_v0(raw)?;
    Ok(build_cme_scenario_v0(fixture, archetype, None))
}

/// Build a v0.1 scenario summary and record the macro invocation call-site.
#[track_caller]
pub fn summarize_cme_scenario_at_call_site_v0(
    raw: &str,
    archetype: CmeScenarioArchetypeV0,
) -> Result<CmeScenarioV0, String> {
    let caller = std::panic::Location::caller();
    let call_site = CmeScenarioCallSiteV0 {
        file: caller.file(),
        line: caller.line(),
        column: caller.column(),
    };
    let fixture = parse_cme_fixture_v0(raw)?;
    Ok(build_cme_scenario_v0(fixture, archetype, Some(call_site)))
}

/// Construct a `cme-fixture-v0` scenario using the v0.1 archetype macro.
#[macro_export]
macro_rules! cme_scenario_v0 {
    (boundary, $raw:expr) => {
        $crate::summarize_cme_scenario_at_call_site_v0(
            $raw,
            $crate::CmeScenarioArchetypeV0::Boundary,
        )
    };
    (transform_execute, $raw:expr) => {
        $crate::summarize_cme_scenario_at_call_site_v0(
            $raw,
            $crate::CmeScenarioArchetypeV0::TransformExecute,
        )
    };
    (lsp_scenario, $raw:expr) => {
        $crate::summarize_cme_scenario_at_call_site_v0(
            $raw,
            $crate::CmeScenarioArchetypeV0::LspScenario,
        )
    };
    (shadow_omena, $raw:expr) => {
        $crate::summarize_cme_scenario_at_call_site_v0(
            $raw,
            $crate::CmeScenarioArchetypeV0::ShadowOmenaVerb,
        )
    };
}

/// Summarize the built-in v0.1 scenario macro substrate.
pub fn summarize_omena_testkit_scenario_macro_report() -> OmenaTestkitScenarioMacroReportV0 {
    let call_site_probe = summarize_scenario_macro_call_site_probe();
    let call_site_evidence_supported = call_site_probe.present
        && call_site_probe.file_suffix_matches
        && call_site_probe.line_non_zero
        && call_site_probe.column_non_zero;
    let reports = SCENARIO_MACRO_SEEDS
        .iter()
        .map(|seed| OmenaTestkitScenarioMacroSeedReportV0 {
            label: seed.label,
            scenario: summarize_cme_scenario_v0(seed.raw, seed.archetype)
                .unwrap_or_else(|error| scenario_parse_error(seed.archetype, error)),
        })
        .collect::<Vec<_>>();
    let all_scenario_macros_ready = reports.iter().all(|report| report.scenario.readiness.ready);

    OmenaTestkitScenarioMacroReportV0 {
        schema_version: "0",
        product: "omena-testkit.scenario-macro-report",
        fixture_grammar: "cme-fixture-v0",
        supported_archetypes: vec![
            CmeScenarioArchetypeV0::Boundary.id(),
            CmeScenarioArchetypeV0::TransformExecute.id(),
            CmeScenarioArchetypeV0::LspScenario.id(),
            CmeScenarioArchetypeV0::ShadowOmenaVerb.id(),
        ],
        scenario_count: reports.len(),
        all_scenario_macros_ready,
        call_site_evidence_supported,
        call_site_probe,
        reports,
    }
}

fn summarize_scenario_macro_call_site_probe() -> OmenaTestkitScenarioMacroCallSiteProbeV0 {
    let scenario = crate::cme_scenario_v0!(boundary, SCENARIO_MACRO_SEEDS[0].raw).ok();
    let call_site = scenario.and_then(|scenario| scenario.call_site);

    OmenaTestkitScenarioMacroCallSiteProbeV0 {
        present: call_site.is_some(),
        file_suffix_matches: call_site
            .as_ref()
            .is_some_and(|site| site.file.ends_with("scenario.rs")),
        line_non_zero: call_site.as_ref().is_some_and(|site| site.line > 0),
        column_non_zero: call_site.as_ref().is_some_and(|site| site.column > 0),
    }
}

fn build_cme_scenario_v0(
    fixture: CmeFixtureV0,
    archetype: CmeScenarioArchetypeV0,
    call_site: Option<CmeScenarioCallSiteV0>,
) -> CmeScenarioV0 {
    let source_file_count = fixture
        .files
        .iter()
        .filter(|file| file_is_source(file))
        .count();
    let style_file_count = fixture
        .files
        .iter()
        .filter(|file| file_is_style(file))
        .count();
    let expectation_count = fixture.expectations.len();
    let marker_count = fixture.files.iter().map(|file| file.markers.len()).sum();
    let metadata_count = fixture.files.iter().map(|file| file.metadata.len()).sum();
    let expected_products = fixture
        .expectations
        .iter()
        .filter(|expectation| expectation.key == "product")
        .map(|expectation| expectation.value.clone())
        .collect::<Vec<_>>();
    let shadow_omena_verbs = shadow_omena_verbs_from_products(expected_products.as_slice());
    let readiness = cme_scenario_readiness(
        archetype,
        style_file_count,
        source_file_count,
        marker_count,
        expected_products.as_slice(),
        shadow_omena_verbs.as_slice(),
    );

    CmeScenarioV0 {
        schema_version: "0",
        product: "omena-testkit.scenario",
        fixture_grammar: "cme-fixture-v0",
        archetype,
        archetype_id: archetype.id(),
        file_count: fixture.files.len(),
        source_file_count,
        style_file_count,
        expectation_count,
        marker_count,
        metadata_count,
        expected_products,
        shadow_omena_verbs,
        readiness,
        call_site,
    }
}

fn cme_scenario_readiness(
    archetype: CmeScenarioArchetypeV0,
    style_file_count: usize,
    source_file_count: usize,
    marker_count: usize,
    expected_products: &[String],
    shadow_omena_verbs: &[String],
) -> CmeScenarioReadinessV0 {
    let has_expected_products = !expected_products.is_empty();
    let has_required_files = match archetype {
        CmeScenarioArchetypeV0::Boundary => style_file_count + source_file_count > 0,
        CmeScenarioArchetypeV0::TransformExecute => style_file_count > 0,
        CmeScenarioArchetypeV0::LspScenario => style_file_count > 0 && source_file_count > 0,
        CmeScenarioArchetypeV0::ShadowOmenaVerb => style_file_count + source_file_count > 0,
    };
    let has_required_markers = match archetype {
        CmeScenarioArchetypeV0::LspScenario => marker_count > 0,
        CmeScenarioArchetypeV0::Boundary
        | CmeScenarioArchetypeV0::TransformExecute
        | CmeScenarioArchetypeV0::ShadowOmenaVerb => true,
    };
    let has_supported_introspection = match archetype {
        CmeScenarioArchetypeV0::Boundary => has_expected_products,
        CmeScenarioArchetypeV0::TransformExecute => expected_products.iter().any(|product| {
            product == "omena-query.transform-execute"
                || product.starts_with("omena-transform-passes.")
        }),
        CmeScenarioArchetypeV0::LspScenario => expected_products
            .iter()
            .any(|product| product.starts_with("omena-lsp-server.")),
        CmeScenarioArchetypeV0::ShadowOmenaVerb => !shadow_omena_verbs.is_empty(),
    };
    let mut unsupported_reasons = Vec::new();
    if !has_expected_products {
        unsupported_reasons.push("missingProductExpectation");
    }
    if !has_required_files {
        unsupported_reasons.push("missingRequiredFiles");
    }
    if !has_required_markers {
        unsupported_reasons.push("missingRequiredMarkers");
    }
    if !has_supported_introspection {
        unsupported_reasons.push("unsupportedIntrospectionSurface");
    }

    CmeScenarioReadinessV0 {
        fixture_parses: true,
        has_expected_products,
        has_required_files,
        has_required_markers,
        has_supported_introspection,
        ready: unsupported_reasons.is_empty(),
        unsupported_reasons,
    }
}

fn shadow_omena_verbs_from_products(expected_products: &[String]) -> Vec<String> {
    expected_products
        .iter()
        .filter_map(|product| product.strip_prefix("shadow.omena."))
        .map(ToString::to_string)
        .collect()
}

fn scenario_parse_error(archetype: CmeScenarioArchetypeV0, _error: String) -> CmeScenarioV0 {
    CmeScenarioV0 {
        schema_version: "0",
        product: "omena-testkit.scenario",
        fixture_grammar: "cme-fixture-v0",
        archetype,
        archetype_id: archetype.id(),
        file_count: 0,
        source_file_count: 0,
        style_file_count: 0,
        expectation_count: 0,
        marker_count: 0,
        metadata_count: 0,
        expected_products: Vec::new(),
        shadow_omena_verbs: Vec::new(),
        readiness: CmeScenarioReadinessV0 {
            fixture_parses: false,
            has_expected_products: false,
            has_required_files: false,
            has_required_markers: false,
            has_supported_introspection: false,
            ready: false,
            unsupported_reasons: vec!["fixtureParseError"],
        },
        call_site: None,
    }
}

fn file_is_source(file: &CmeFixtureFileV0) -> bool {
    metadata_value(file, "dialect")
        .is_some_and(|dialect| matches!(dialect, "ts" | "tsx" | "js" | "jsx"))
        || file.path.ends_with(".ts")
        || file.path.ends_with(".tsx")
        || file.path.ends_with(".js")
        || file.path.ends_with(".jsx")
        || file.path.ends_with(".mts")
        || file.path.ends_with(".cts")
}

fn file_is_style(file: &CmeFixtureFileV0) -> bool {
    metadata_value(file, "dialect")
        .is_some_and(|dialect| matches!(dialect, "css" | "scss" | "less"))
        || file.path.ends_with(".css")
        || file.path.ends_with(".scss")
        || file.path.ends_with(".sass")
        || file.path.ends_with(".less")
}

fn metadata_value<'a>(file: &'a CmeFixtureFileV0, key: &str) -> Option<&'a str> {
    file.metadata
        .iter()
        .find(|metadata| metadata.key == key)
        .map(|metadata| metadata.value.as_str())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn scenario_macro_builds_boundary_archetype_summary() -> Result<(), String> {
        let scenario = crate::cme_scenario_v0!(
            boundary,
            r#"--- file: src/Button.module.css
.button { color: red; }
--- expect: product
omena-parser.style-facts
"#
        )?;

        assert_eq!(scenario.product, "omena-testkit.scenario");
        assert_eq!(scenario.archetype_id, "boundary");
        assert_eq!(scenario.style_file_count, 1);
        assert_eq!(scenario.expected_products, vec!["omena-parser.style-facts"]);
        assert!(scenario.shadow_omena_verbs.is_empty());
        assert!(scenario.readiness.ready);
        let call_site = scenario
            .call_site
            .as_ref()
            .ok_or("macro scenario must carry call-site evidence")?;
        assert!(call_site.file.ends_with("scenario.rs"));
        assert!(call_site.line > 0);
        assert!(call_site.column > 0);

        Ok(())
    }

    #[test]
    fn scenario_macro_builds_transform_execute_archetype_summary() -> Result<(), String> {
        let scenario = crate::cme_scenario_v0!(
            transform_execute,
            r#"//- src/Button.module.scss dialect:scss
.button { color: light-dark(red, blue); }
--- expect: product
omena-query.transform-execute
"#
        )?;

        assert_eq!(scenario.archetype_id, "transform-execute");
        assert_eq!(scenario.style_file_count, 1);
        assert_eq!(scenario.metadata_count, 1);
        assert!(
            scenario
                .expected_products
                .contains(&"omena-query.transform-execute".to_string())
        );
        assert!(scenario.readiness.ready);
        assert!(
            scenario.call_site.is_some(),
            "macro scenario must preserve call-site evidence"
        );

        Ok(())
    }

    #[test]
    fn transform_execute_scenario_requires_transform_product() -> Result<(), String> {
        let scenario = summarize_cme_scenario_v0(
            r#"--- file: src/Button.module.scss
.button { color: red; }
--- expect: product
omena-parser.style-facts
"#,
            CmeScenarioArchetypeV0::TransformExecute,
        )?;

        assert!(!scenario.readiness.ready);
        assert_eq!(scenario.call_site, None);
        assert_eq!(
            scenario.readiness.unsupported_reasons,
            vec!["unsupportedIntrospectionSurface"]
        );

        Ok(())
    }

    #[test]
    fn scenario_macro_builds_lsp_archetype_summary() -> Result<(), String> {
        let scenario = crate::cme_scenario_v0!(
            lsp_scenario,
            r#"--- file: src/App.tsx
import styles from "./Button.module.scss";
styles./*|*/button;
--- file: src/Button.module.scss
.button { color: red; }
--- expect: product
omena-lsp-server.definition
"#
        )?;

        assert_eq!(scenario.archetype_id, "lsp-scenario");
        assert_eq!(scenario.source_file_count, 1);
        assert_eq!(scenario.style_file_count, 1);
        assert_eq!(scenario.marker_count, 1);
        assert!(scenario.readiness.has_required_markers);
        assert!(scenario.readiness.has_supported_introspection);
        assert!(scenario.readiness.ready);

        Ok(())
    }

    #[test]
    fn lsp_scenario_requires_request_marker() -> Result<(), String> {
        let scenario = summarize_cme_scenario_v0(
            r#"--- file: src/App.tsx
import styles from "./Button.module.scss";
styles.button;
--- file: src/Button.module.scss
.button { color: red; }
--- expect: product
omena-lsp-server.definition
"#,
            CmeScenarioArchetypeV0::LspScenario,
        )?;

        assert!(!scenario.readiness.ready);
        assert_eq!(scenario.marker_count, 0);
        assert_eq!(
            scenario.readiness.unsupported_reasons,
            vec!["missingRequiredMarkers"]
        );

        Ok(())
    }

    #[test]
    fn scenario_macro_builds_shadow_omena_verb_summary() -> Result<(), String> {
        let scenario = crate::cme_scenario_v0!(
            shadow_omena,
            r#"--- file: src/App.tsx
import styles from "./Button.module.scss";
styles.button;
--- file: src/Button.module.scss
.button { color: red; }
--- expect: product
shadow.omena.hover
--- expect: product
shadow.omena.references
"#
        )?;

        assert_eq!(scenario.archetype_id, "shadow-omena-verb");
        assert_eq!(scenario.shadow_omena_verbs, vec!["hover", "references"]);
        assert!(scenario.readiness.has_supported_introspection);
        assert!(scenario.readiness.ready);

        Ok(())
    }
}

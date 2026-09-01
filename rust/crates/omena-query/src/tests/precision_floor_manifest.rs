use super::dynamic_classname::dynamic_classname_input;
use super::*;
use crate::{
    AbstractClassValueV0, AnalysisPrecisionV1, FactPrecision, OmenaQueryAnalysisPrecisionV0,
    OmenaQuerySourceDocumentInputV0, OmenaQueryStyleSourceInputV0, ParserPositionV0, ParserRangeV0,
    RevisionIdentityV1, analysis_precision_from_class_value,
    read_omena_query_cascade_at_position_analysis_result,
    resolve_omena_query_source_precision_for_source,
    summarize_omena_query_css_modules_export_usage,
    summarize_omena_query_dynamic_classname_m_tier_diagnostics_with_context_depth,
    summarize_omena_query_evaluation_runtime,
    summarize_omena_query_global_class_fallthrough_diagnostic,
    summarize_omena_query_source_diagnostics_for_workspace_file,
};
use serde::Serialize;
use serde_json::{Value, json};

fn wire<T: Serialize>(value: T) -> Result<String, String> {
    serde_json::to_value(value)
        .map_err(|error| error.to_string())?
        .as_str()
        .map(str::to_string)
        .ok_or_else(|| "precision wire value must serialize as a string".to_string())
}

fn effective(precision: &OmenaQueryAnalysisPrecisionV0) -> FactPrecision {
    omena_query_core::fact_precision_from_analysis_precision(precision)
}

fn observation(output: &str, after: impl Into<String>) -> Value {
    json!({ "output": output, "after": after.into() })
}

fn producer_arm(
    id: &str,
    axis: &str,
    producer_path: &str,
    observed: String,
    precision: FactPrecision,
) -> Value {
    json!({
        "id": id,
        "axis": axis,
        "producerPath": producer_path,
        "observed": observed,
        "effective": precision.describe(),
        "requiredFloor": FactPrecision::Conservative.describe(),
        "gateOpen": precision.satisfies(FactPrecision::Conservative),
    })
}

#[test]
fn precision_floor_executable_manifest_from_real_product_fixtures() -> Result<(), String> {
    let [open_world, non_enumerated, missing_member, empty_input] =
        crate::style::precision_floor_closed_set_observations()?;

    let style_sources = vec![
        OmenaQueryStyleSourceInputV0 {
            style_path: "/workspace/base.module.css".to_string(),
            style_source: ".base {}".to_string(),
        },
        OmenaQueryStyleSourceInputV0 {
            style_path: "/workspace/middle.module.css".to_string(),
            style_source: ".middle { composes: base from \"./base.module.css\"; }".to_string(),
        },
        OmenaQueryStyleSourceInputV0 {
            style_path: "/workspace/app.module.css".to_string(),
            style_source: ".composed { composes: middle from \"./middle.module.css\"; } .ghost {}"
                .to_string(),
        },
    ];
    let source_documents = vec![OmenaQuerySourceDocumentInputV0 {
        source_path: "/workspace/App.tsx".to_string(),
        source_source: r#"import styles from "./app.module.css";
export const App = () => <div className={styles.composed} />;"#
            .to_string(),
        source_syntax_index: None,
        has_unresolved_style_import: false,
    }];
    let usage = summarize_omena_query_css_modules_export_usage(
        &style_sources,
        &source_documents,
        &[],
        None,
    );
    let safe_ghost = usage
        .diagnostics
        .iter()
        .find(|diagnostic| diagnostic.export_name == "ghost")
        .ok_or_else(|| "safe ghost diagnostic must exist".to_string())?;

    let global = summarize_omena_query_global_class_fallthrough_diagnostic(
        "global-only",
        "file:///workspace/global.css",
        "file:///workspace/App.module.css",
        ".local {}",
        ParserRangeV0 {
            start: ParserPositionV0 {
                line: 0,
                character: 0,
            },
            end: ParserPositionV0 {
                line: 0,
                character: 11,
            },
        },
    );
    let global_precision = global
        .precision
        .as_ref()
        .ok_or_else(|| "global fallthrough precision must exist".to_string())?;

    let source = r#"import bind from "classnames/bind";
import styles from "./App.module.scss";
import missing from "./Missing.module.scss";
const cx = bind.bind(styles);
const variant = Math.random() > 0.5 ? "chip" : "ghost";
const dynamicPrefix = "lost-" + suffix;
export function App({ suffix }) {
  return <div className={cx("ghost", variant, dynamicPrefix, `empty-${suffix}`)} data-x={styles.ghost} />;
}"#;
    let source_report = summarize_omena_query_source_diagnostics_for_workspace_file(
        "/workspace/src/App.tsx",
        source,
        &[OmenaQueryStyleSourceInputV0 {
            style_path: "/workspace/src/App.module.scss".to_string(),
            style_source: ".root {}\n.chip {}\n".to_string(),
        }],
        &[],
    );
    let source_precision = |code: &str| -> Result<FactPrecision, String> {
        let precision = source_report
            .diagnostics
            .iter()
            .find(|diagnostic| diagnostic.code == code)
            .and_then(|diagnostic| diagnostic.precision.as_ref())
            .ok_or_else(|| format!("{code} precision must exist"))?;
        Ok(effective(precision))
    };
    let no_capture = resolve_omena_query_source_precision_for_source(
        "/workspace/NoCapture.tsx",
        "export const unrelated = 'value';",
        Some("tsx"),
        "styles",
        0,
    );

    let input = sample_input();
    let mut cascade_runtime = OmenaQueryExpressionDomainFlowRuntimeV0::default();
    let cascade_runtime_summary =
        summarize_omena_query_evaluation_runtime(&input, &mut cascade_runtime);
    let resolved_cascade = read_omena_query_cascade_at_position_analysis_result(
        "Component.module.css",
        ":root { --surface: white; }\n:root { --surface: black; }\n.button { color: var(--surface); }\n",
        &input,
        ParserPositionV0 {
            line: 2,
            character: 24,
        },
        &cascade_runtime_summary,
    )
    .ok_or_else(|| "resolved cascade fixture must produce a result".to_string())?;
    let unresolved_cascade = read_omena_query_cascade_at_position_analysis_result(
        "Component.module.css",
        ".button { color: var(--missing); }",
        &input,
        ParserPositionV0 {
            line: 0,
            character: 23,
        },
        &cascade_runtime_summary,
    )
    .ok_or_else(|| "unresolved cascade fixture must produce a result".to_string())?;

    let two_cfa = summarize_omena_query_dynamic_classname_m_tier_diagnostics_with_context_depth(
        &dynamic_classname_input(2),
    );
    let two_cfa_precision = two_cfa
        .diagnostics
        .first()
        .and_then(|diagnostic| diagnostic.precision.as_ref())
        .ok_or_else(|| "2-CFA precision fixture must produce a diagnostic".to_string())?;
    let zero_cfa = summarize_omena_query_dynamic_classname_m_tier_diagnostics_with_context_depth(
        &dynamic_classname_input(0),
    );
    let zero_cfa_precision = zero_cfa
        .diagnostics
        .first()
        .and_then(|diagnostic| diagnostic.precision.as_ref())
        .ok_or_else(|| "0-CFA precision fixture must produce a diagnostic".to_string())?;

    let value_axes = analysis_precision_from_class_value(&AbstractClassValueV0::FiniteSet {
        values: vec!["card".to_string(), "panel".to_string()],
    });
    let value_effective = omena_query_core::fact_precision_from_precision_axes(&value_axes);

    let mut unresolved_input = sample_input();
    unresolved_input.type_facts.clear();
    let mut unresolved_runtime = OmenaQueryExpressionDomainFlowRuntimeV0::default();
    let unresolved = summarize_omena_query_expression_domain_incremental_flow_analysis_result(
        &unresolved_input,
        &mut unresolved_runtime,
    );
    let unresolved_effective = effective(&unresolved.precision);

    let mut open_input = sample_input();
    open_input.styles.clear();
    let mut open_runtime = OmenaQueryExpressionDomainFlowRuntimeV0::default();
    let open = summarize_omena_query_expression_domain_incremental_flow_analysis_result(
        &open_input,
        &mut open_runtime,
    );
    let open_effective = effective(&open.precision);

    let mut revision_runtime = OmenaQueryExpressionDomainFlowRuntimeV0::default();
    let first_revision = summarize_omena_query_expression_domain_incremental_flow_analysis_result(
        &sample_input(),
        &mut revision_runtime,
    );
    let mut edited_input = sample_input();
    edited_input.type_facts[0].facts.suffix = Some("-edited".to_string());
    let _second_revision = summarize_omena_query_expression_domain_incremental_flow_analysis_result(
        &edited_input,
        &mut revision_runtime,
    );
    let stale_axes = AnalysisPrecisionV1 {
        revision: RevisionIdentityV1::expression_domain_runtime(
            first_revision.revision,
            revision_runtime.revision(),
        ),
        ..first_revision.precision.axes
    };
    let stale_effective = omena_query_core::fact_precision_from_precision_axes(&stale_axes);

    let observations = vec![
        observation(
            "closedSetFiniteReachability.openWorldPrecision",
            open_world.describe(),
        ),
        observation(
            "closedSetFiniteReachability.nonEnumeratedPrecision",
            non_enumerated.describe(),
        ),
        observation(
            "closedSetFiniteReachability.missingMemberPrecision",
            missing_member.describe(),
        ),
        observation(
            "cssModulesUnusedExportFlow.safeGhost.precision",
            safe_ghost.precision.describe(),
        ),
        observation(
            "globalClassFallthrough.precision",
            effective(global_precision).describe(),
        ),
        observation(
            "missing-module.precision",
            source_precision("missing-module")?.describe(),
        ),
        observation(
            "missingResolvedClassDomain.precision",
            crate::style::empty_resolved_class_domain_precision_fixture()?.describe(),
        ),
        observation(
            "sourcePrecisionReference.precision",
            effective(&no_capture.precision).describe(),
        ),
        observation(
            "cascadeAtPosition.precision",
            effective(&resolved_cascade.precision).describe(),
        ),
        observation(
            "closedSetFiniteReachability.emptyInputPrecision",
            empty_input.describe(),
        ),
        observation(
            "cascadeAtPosition.unresolved.precision",
            effective(&unresolved_cascade.precision).describe(),
        ),
        observation(
            "analysisPrecision.contextSensitivity",
            wire(two_cfa_precision.axes.context)?,
        ),
    ];

    let producer_gate_arms = vec![
        producer_arm(
            "valueDomain.closedSet",
            "valueDomain",
            "omena-abstract-value.analysis_precision_from_class_value",
            wire(value_axes.value_domain)?,
            value_effective,
        ),
        producer_arm(
            "flow.dynamicClassname",
            "flow",
            "omena-query.dynamic-classname-product-fixture",
            wire(two_cfa_precision.axes.flow)?,
            effective(two_cfa_precision),
        ),
        producer_arm(
            "context.zeroCfa",
            "context",
            "omena-query.dynamic-classname-product-fixture",
            wire(zero_cfa_precision.axes.context)?,
            effective(zero_cfa_precision),
        ),
        producer_arm(
            "provider.missingTypeFacts",
            "providerCompleteness",
            "omena-query-core.incremental-expression-domain",
            wire(unresolved.precision.axes.provider_completeness)?,
            unresolved_effective,
        ),
        producer_arm(
            "world.missingStyles",
            "worldAssumption",
            "omena-query-core.incremental-expression-domain",
            wire(open.precision.axes.world_assumption)?,
            open_effective,
        ),
        producer_arm(
            "revision.staleRuntime",
            "revision",
            "omena-query-core.expression-domain-runtime",
            wire(stale_axes.revision)?,
            stale_effective,
        ),
    ];

    let manifest = json!({
        "schemaVersion": "1",
        "test": "tests::precision_floor_manifest::precision_floor_executable_manifest_from_real_product_fixtures",
        "observations": observations,
        "producerGateArms": producer_gate_arms,
    });
    println!("OMENA_PRECISION_FLOOR_EXECUTABLE_MANIFEST={manifest}");
    Ok(())
}

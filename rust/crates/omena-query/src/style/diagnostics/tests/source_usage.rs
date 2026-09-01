use super::source_usage::{
    OmenaQueryCssModuleExportUsageStatusV0, OmenaQueryCssModulesUnusedExportSkipReasonV0,
    summarize_omena_query_css_modules_export_usage,
};
use super::{FactPrecision, OmenaQuerySourceDocumentInputV0, OmenaQueryStyleSourceInputV0};

#[test]
fn css_modules_interface_export_usage_reprojects_existing_selector_usage() -> serde_json::Result<()>
{
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

    let report = summarize_omena_query_css_modules_export_usage(
        &style_sources,
        &source_documents,
        &[],
        None,
    );

    assert_eq!(report.used_export_count, 3);
    assert_eq!(report.unused_export_count, 1);
    assert_eq!(report.skipped_export_count, 0);
    assert_eq!(report.diagnostics[0].export_name, "ghost");
    assert_eq!(report.diagnostics[0].precision, FactPrecision::Conservative);
    let calibration_report: serde_json::Value = serde_json::from_str(include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../omena-precision-calibration-report.json"
    )))?;
    assert_eq!(
        calibration_report["cases"][2],
        serde_json::json!({
            "caseId": "cssModulesUnusedExportFlow",
            "inputStyleModuleCount": 3,
            "inputSourceDocumentCount": 1,
            "representation": "styleModuleResolution",
            "authority": "sourceSelectorUsage",
            "currentPrecision": report.diagnostics[0].precision,
            "precisionLabelDrops": [{
                "output": "cssModulesUnusedExportFlow.safeGhost.precision",
                "before": "exact",
                "after": "conservative",
                "loweringAxis": "flow",
            }],
        })
    );
    assert!(report.exports.iter().any(|export| {
        export.style_path.ends_with("base.module.css")
            && export.export_name == "base"
            && export.status == OmenaQueryCssModuleExportUsageStatusV0::Used
    }));
    assert!(report.exports.iter().any(|export| {
        export.style_path.ends_with("middle.module.css")
            && export.export_name == "middle"
            && export.status == OmenaQueryCssModuleExportUsageStatusV0::Used
    }));
    Ok(())
}

#[test]
fn css_modules_interface_unresolved_edges_skip_unused_export_claims() {
    let style_sources = vec![
        OmenaQueryStyleSourceInputV0 {
            style_path: "/workspace/app.module.css".to_string(),
            style_source: ".button { composes: missing from \"./missing.module.css\"; } .ghost {}"
                .to_string(),
        },
        OmenaQueryStyleSourceInputV0 {
            style_path: "/workspace/safe.module.css".to_string(),
            style_source: ".safe {} .safeGhost {}".to_string(),
        },
    ];
    let source_documents = vec![
        OmenaQuerySourceDocumentInputV0 {
            source_path: "/workspace/App.tsx".to_string(),
            source_source: r#"import styles from "./app.module.css";
export const App = () => <div className={styles.button} />;"#
                .to_string(),
            source_syntax_index: None,
            has_unresolved_style_import: false,
        },
        OmenaQuerySourceDocumentInputV0 {
            source_path: "/workspace/Safe.tsx".to_string(),
            source_source: r#"import styles from "./safe.module.css";
export const Safe = () => <div className={styles.safe} />;"#
                .to_string(),
            source_syntax_index: None,
            has_unresolved_style_import: false,
        },
    ];

    let report = summarize_omena_query_css_modules_export_usage(
        &style_sources,
        &source_documents,
        &[],
        None,
    );

    assert_eq!(report.unresolved_import_edge_count, 1);
    assert_eq!(report.used_export_count, 1);
    assert_eq!(report.unused_export_count, 1);
    assert_eq!(report.skipped_export_count, 2);
    assert_eq!(report.diagnostics[0].export_name, "safeGhost");
    assert!(
        report
            .exports
            .iter()
            .filter(|export| { export.style_path.ends_with("app.module.css") })
            .all(|export| {
                export.status == OmenaQueryCssModuleExportUsageStatusV0::Skipped
                    && export.precision == FactPrecision::Unknown
                    && export.skip_reasons.contains(
                        &OmenaQueryCssModulesUnusedExportSkipReasonV0::UnresolvedImportEdge,
                    )
            })
    );
    assert!(report.exports.iter().any(|export| {
        export.style_path.ends_with("safe.module.css")
            && export.export_name == "safe"
            && export.status == OmenaQueryCssModuleExportUsageStatusV0::Used
    }));
    assert_eq!(report.diagnostics[0].precision, FactPrecision::Conservative);
}

use super::{OmenaQueryCssModulesCrossFileResolutionV0, OmenaQueryModuleInterfaceProjectionV0};
use crate::{
    OmenaCrossFileSummaryViewReportV0, OmenaQueryCssModulesExportUsageReportV0,
    OmenaQueryModuleIdV0,
};
use serde::Serialize;
use std::collections::{BTreeMap, BTreeSet};

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OmenaQueryCssModulesInterfaceBundleV0 {
    pub schema_version: &'static str,
    pub product: &'static str,
    pub module_count: usize,
    pub class_export_count: usize,
    pub icss_export_count: usize,
    pub modules: Vec<OmenaQueryCssModuleInterfaceV0>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OmenaQueryCssModuleInterfaceV0 {
    pub module_id: OmenaQueryModuleIdV0,
    pub style_path: String,
    pub class_exports: Vec<OmenaQueryCssModuleClassExportV0>,
    pub icss_exports: Vec<OmenaQueryCssModuleIcssExportV0>,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OmenaQueryCssModuleClassReferenceV0 {
    pub module_id: OmenaQueryModuleIdV0,
    pub name: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OmenaQueryCssModuleClassExportV0 {
    pub name: String,
    pub resolved_classes: Vec<OmenaQueryCssModuleClassReferenceV0>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OmenaQueryCssModuleIcssExportV0 {
    pub name: String,
    pub value: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OmenaQueryCssModulesInterfaceSummaryViewV0 {
    pub schema_version: &'static str,
    pub product: &'static str,
    pub module_count: usize,
    pub export_count: usize,
    pub icss_export_count: usize,
    pub unused_export_count: usize,
    pub skipped_export_count: usize,
    pub cross_file_summary_ready: bool,
    pub interface_summary_ready: bool,
}

pub(super) fn summarize_css_modules_interface_bundle_from_projections(
    projections: &[OmenaQueryModuleInterfaceProjectionV0],
    resolution: &OmenaQueryCssModulesCrossFileResolutionV0,
    icss_export_values_by_path: &BTreeMap<String, BTreeMap<String, String>>,
) -> OmenaQueryCssModulesInterfaceBundleV0 {
    let mut projections = projections.iter().collect::<Vec<_>>();
    projections.sort_by(|left, right| left.style_path.cmp(&right.style_path));

    let modules = projections
        .into_iter()
        .map(|projection| {
            module_interface_from_projection(
                projection,
                resolution,
                icss_export_values_by_path.get(projection.style_path.as_str()),
            )
        })
        .collect::<Vec<_>>();
    let class_export_count = modules
        .iter()
        .map(|module| module.class_exports.len())
        .sum();
    let icss_export_count = modules.iter().map(|module| module.icss_exports.len()).sum();

    OmenaQueryCssModulesInterfaceBundleV0 {
        schema_version: "0",
        product: "omena-query.css-modules-interface-bundle",
        module_count: modules.len(),
        class_export_count,
        icss_export_count,
        modules,
    }
}

fn module_interface_from_projection(
    projection: &OmenaQueryModuleInterfaceProjectionV0,
    resolution: &OmenaQueryCssModulesCrossFileResolutionV0,
    icss_export_values: Option<&BTreeMap<String, String>>,
) -> OmenaQueryCssModuleInterfaceV0 {
    let module_id = OmenaQueryModuleIdV0::new(projection.style_path.clone());
    let class_exports = projection
        .css_modules_style_facts
        .class_selector_names
        .iter()
        .cloned()
        .collect::<BTreeSet<_>>()
        .into_iter()
        .map(|name| {
            let mut resolved_classes = BTreeSet::from([OmenaQueryCssModuleClassReferenceV0 {
                module_id: module_id.clone(),
                name: name.clone(),
            }]);
            resolved_classes.extend(
                resolution
                    .composes_closure_edges
                    .iter()
                    .filter(|edge| {
                        edge.from_style_path == projection.style_path
                            && edge.owner_selector_name == name
                    })
                    .map(|edge| OmenaQueryCssModuleClassReferenceV0 {
                        module_id: OmenaQueryModuleIdV0::new(edge.target_style_path.clone()),
                        name: edge.target_selector_name.clone(),
                    }),
            );
            OmenaQueryCssModuleClassExportV0 {
                name,
                resolved_classes: resolved_classes.into_iter().collect(),
            }
        })
        .collect::<Vec<_>>();

    let icss_exports = projection
        .css_modules_style_facts
        .icss_export_names
        .iter()
        .filter_map(|export_name| {
            icss_export_values
                .and_then(|values| values.get(export_name.as_str()))
                .map(|value| (export_name.clone(), value.clone()))
        })
        .collect::<BTreeMap<_, _>>()
        .into_iter()
        .map(|(name, value)| OmenaQueryCssModuleIcssExportV0 { name, value })
        .collect();

    OmenaQueryCssModuleInterfaceV0 {
        module_id,
        style_path: projection.style_path.clone(),
        class_exports,
        icss_exports,
    }
}

pub fn render_omena_query_css_module_typescript_declaration(
    module: &OmenaQueryCssModuleInterfaceV0,
) -> String {
    let mut output = String::from("declare const styles: {\n");
    let export_names = module
        .class_exports
        .iter()
        .map(|export| export.name.as_str())
        .chain(
            module
                .icss_exports
                .iter()
                .map(|export| export.name.as_str()),
        )
        .collect::<BTreeSet<_>>();
    for export_name in export_names {
        let name = serde_json::to_string(export_name).unwrap_or_else(|_| "\"\"".to_string());
        output.push_str("  readonly ");
        output.push_str(name.as_str());
        output.push_str(": string;\n");
    }
    output.push_str("};\nexport default styles;\n");
    output
}

pub fn render_omena_query_css_modules_interface_json(
    bundle: &OmenaQueryCssModulesInterfaceBundleV0,
) -> Result<String, serde_json::Error> {
    serde_json::to_string_pretty(bundle).map(|mut output| {
        output.push('\n');
        output
    })
}

pub fn summarize_omena_query_css_modules_interface_summary_view(
    cross_file_summary_view: &OmenaCrossFileSummaryViewReportV0,
    bundle: &OmenaQueryCssModulesInterfaceBundleV0,
    usage: &OmenaQueryCssModulesExportUsageReportV0,
) -> OmenaQueryCssModulesInterfaceSummaryViewV0 {
    let export_count_matches = bundle.class_export_count == usage.export_count;
    OmenaQueryCssModulesInterfaceSummaryViewV0 {
        schema_version: "0",
        product: "omena-query.css-modules-interface-summary-view",
        module_count: bundle.module_count,
        export_count: bundle.class_export_count,
        icss_export_count: bundle.icss_export_count,
        unused_export_count: usage.unused_export_count,
        skipped_export_count: usage.skipped_export_count,
        cross_file_summary_ready: cross_file_summary_view.summary_view_ready,
        interface_summary_ready: cross_file_summary_view.summary_view_ready
            && export_count_matches
            && usage.unresolved_import_edge_count == 0
            && usage.skipped_export_count == 0,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{OmenaQueryStylePackageManifestV0, OmenaQueryStyleSourceInputV0};

    #[test]
    fn css_modules_interface_resolves_composes_and_preserves_icss_values() -> Result<(), String> {
        let sources = vec![
            OmenaQueryStyleSourceInputV0 {
                style_path: "/workspace/base.module.css".to_string(),
                style_source: ".base {}".to_string(),
            },
            OmenaQueryStyleSourceInputV0 {
                style_path: "/workspace/button.module.css".to_string(),
                style_source:
                    ":export { tone: #0af; } .button { composes: base from \"./base.module.css\"; }"
                        .to_string(),
            },
        ];

        let bundle = super::super::summarize_omena_query_css_modules_interface_bundle(
            &sources,
            &Vec::<OmenaQueryStylePackageManifestV0>::new(),
        );

        assert_eq!(bundle.module_count, 2);
        let button = bundle
            .modules
            .iter()
            .find(|module| module.style_path.ends_with("button.module.css"))
            .ok_or_else(|| "missing button interface".to_string())?;
        assert_eq!(button.icss_exports[0].name, "tone");
        assert_eq!(button.icss_exports[0].value, "#0af");
        assert_eq!(button.class_exports[0].name, "button");
        assert_eq!(button.class_exports[0].resolved_classes.len(), 2);
        assert!(
            button.class_exports[0]
                .resolved_classes
                .iter()
                .any(|class| {
                    class.module_id.as_str().ends_with("base.module.css") && class.name == "base"
                })
        );
        let declaration = render_omena_query_css_module_typescript_declaration(button);
        assert!(declaration.contains("readonly \"button\": string;"));
        assert!(declaration.contains("readonly \"tone\": string;"));
        Ok(())
    }

    #[test]
    fn css_modules_interface_declaration_and_json_are_byte_deterministic() -> Result<(), String> {
        let sources = vec![
            OmenaQueryStyleSourceInputV0 {
                style_path: "/workspace/zeta.module.css".to_string(),
                style_source: ".zeta {} .alpha {} :export { tone: rgb(1, 2, 3); }".to_string(),
            },
            OmenaQueryStyleSourceInputV0 {
                style_path: "/workspace/alpha.module.css".to_string(),
                style_source: ".root {}".to_string(),
            },
        ];
        let mut reversed_sources = sources.clone();
        reversed_sources.reverse();
        let bundle =
            super::super::summarize_omena_query_css_modules_interface_bundle(&sources, &[]);
        let reversed_bundle = super::super::summarize_omena_query_css_modules_interface_bundle(
            &reversed_sources,
            &[],
        );

        let first_declaration =
            render_omena_query_css_module_typescript_declaration(&bundle.modules[1]);
        let second_declaration =
            render_omena_query_css_module_typescript_declaration(&reversed_bundle.modules[1]);
        assert_eq!(first_declaration, second_declaration);
        let alpha_offset = first_declaration
            .find("alpha")
            .ok_or_else(|| "missing alpha declaration".to_string())?;
        let zeta_offset = first_declaration
            .find("zeta")
            .ok_or_else(|| "missing zeta declaration".to_string())?;
        assert!(alpha_offset < zeta_offset);

        let first_json = render_omena_query_css_modules_interface_json(&bundle)
            .map_err(|error| error.to_string())?;
        let second_json = render_omena_query_css_modules_interface_json(&reversed_bundle)
            .map_err(|error| error.to_string())?;
        assert_eq!(first_json, second_json);
        Ok(())
    }

    #[test]
    fn css_modules_interface_summary_is_a_view_over_existing_outputs() {
        let sources = vec![OmenaQueryStyleSourceInputV0 {
            style_path: "/workspace/card.module.css".to_string(),
            style_source: ".used {} .ghost {}".to_string(),
        }];
        let source_documents = vec![crate::OmenaQuerySourceDocumentInputV0 {
            source_path: "/workspace/card.tsx".to_string(),
            source_source: r#"import styles from "./card.module.css";
export const Card = () => <div className={styles.used} />;"#
                .to_string(),
            source_syntax_index: None,
            has_unresolved_style_import: false,
        }];
        let bundle =
            super::super::summarize_omena_query_css_modules_interface_bundle(&sources, &[]);
        let usage = crate::summarize_omena_query_css_modules_export_usage(
            &sources,
            &source_documents,
            &[],
            None,
        );
        let summary = crate::summarize_omena_query_workspace_cross_file_summary(
            &sources,
            &source_documents,
            &[],
        );
        let cross_file_view = crate::summarize_cross_file_summary_view_v0(&summary);

        let view = summarize_omena_query_css_modules_interface_summary_view(
            &cross_file_view,
            &bundle,
            &usage,
        );

        assert_eq!(view.module_count, bundle.module_count);
        assert_eq!(view.export_count, bundle.class_export_count);
        assert_eq!(view.unused_export_count, usage.unused_export_count);
        assert!(view.interface_summary_ready);
    }
}

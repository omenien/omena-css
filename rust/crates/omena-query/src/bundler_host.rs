use crate::{
    OmenaBundlerHostCapabilitiesV0, OmenaBundlerHostComposesEdgeV0, OmenaBundlerHostDiagnosticV0,
    OmenaBundlerHostResolveModuleRequestV0, OmenaBundlerHostResolveModuleResponseV0,
    render_omena_query_css_module_typescript_declaration,
    summarize_omena_query_css_modules_interface_bundle,
};
use std::collections::BTreeMap;

pub const OMENA_BUNDLER_HOST_PROTOCOL_VERSION_V0: &str = "0";

pub fn current_omena_bundler_host_capabilities_v0() -> OmenaBundlerHostCapabilitiesV0 {
    OmenaBundlerHostCapabilitiesV0 {
        protocol_version: OMENA_BUNDLER_HOST_PROTOCOL_VERSION_V0.to_string(),
        capabilities: vec![
            "semanticClassMap".to_string(),
            "namedExports".to_string(),
            "composesEdges".to_string(),
        ],
    }
}

pub fn resolve_omena_bundler_host_module_v0(
    request: OmenaBundlerHostResolveModuleRequestV0,
) -> OmenaBundlerHostResolveModuleResponseV0 {
    let bundle = summarize_omena_query_css_modules_interface_bundle(
        request.style_sources.as_slice(),
        request.package_manifests.as_slice(),
    );
    let Some(module) = bundle
        .modules
        .into_iter()
        .find(|module| module.style_path == request.style_path)
    else {
        return OmenaBundlerHostResolveModuleResponseV0 {
            snapshot_id: request.snapshot_id,
            protocol_version: OMENA_BUNDLER_HOST_PROTOCOL_VERSION_V0.to_string(),
            module_id: request.style_path.clone(),
            class_map: BTreeMap::new(),
            named_exports: BTreeMap::new(),
            typescript_declaration: String::new(),
            composes_edges: Vec::new(),
            diagnostics: vec![OmenaBundlerHostDiagnosticV0 {
                code: "moduleNotFound".to_string(),
                message: format!(
                    "CSS Module '{}' is not present in the bundler host snapshot.",
                    request.style_path
                ),
            }],
            ready: false,
        };
    };

    let mut class_map = BTreeMap::new();
    let mut named_exports = BTreeMap::new();
    let mut composes_edges = Vec::new();
    let mut diagnostics = Vec::new();
    let typescript_declaration = render_omena_query_css_module_typescript_declaration(&module);

    for export in module.class_exports {
        if export.emitted_classes.len() != export.resolved_classes.len() {
            diagnostics.push(OmenaBundlerHostDiagnosticV0 {
                code: "unresolvedEmittedClass".to_string(),
                message: format!(
                    "CSS Module export '{}' could not resolve every emitted class name.",
                    export.name
                ),
            });
            continue;
        }
        let value = export.emitted_classes.join(" ");
        class_map.insert(export.name.clone(), value.clone());
        if let Some(named_export) = export.named_export {
            named_exports.insert(named_export, value);
        }
        composes_edges.extend(export.resolved_classes.into_iter().skip(1).map(|class| {
            OmenaBundlerHostComposesEdgeV0 {
                exported_name: export.name.clone(),
                module_id: class.module_id.as_str().to_string(),
                class_name: class.name,
            }
        }));
    }

    for export in module.icss_exports {
        class_map.insert(export.name.clone(), export.value.clone());
        if let Some(named_export) = export.named_export {
            named_exports.insert(named_export, export.value);
        }
    }

    OmenaBundlerHostResolveModuleResponseV0 {
        snapshot_id: request.snapshot_id,
        protocol_version: OMENA_BUNDLER_HOST_PROTOCOL_VERSION_V0.to_string(),
        module_id: module.module_id.as_str().to_string(),
        class_map,
        named_exports,
        typescript_declaration,
        composes_edges,
        ready: diagnostics.is_empty(),
        diagnostics,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        IncrementalRevisionV0, OmenaQueryStylePackageManifestV0, OmenaQueryStyleSourceInputV0,
    };

    fn request(
        style_path: &str,
        style_sources: Vec<OmenaQueryStyleSourceInputV0>,
    ) -> OmenaBundlerHostResolveModuleRequestV0 {
        OmenaBundlerHostResolveModuleRequestV0 {
            snapshot_id: crate::OmenaWorkspaceSnapshotIdV0::from_revision(IncrementalRevisionV0 {
                value: 7,
            }),
            style_path: style_path.to_string(),
            style_sources,
            package_manifests: Vec::<OmenaQueryStylePackageManifestV0>::new(),
        }
    }

    #[test]
    fn resolves_scoped_classes_named_exports_and_composes_from_one_interface_view() {
        let response = resolve_omena_bundler_host_module_v0(request(
            "/src/button.module.css",
            vec![
                OmenaQueryStyleSourceInputV0 {
                    style_path: "/src/base.module.css".to_string(),
                    style_source: ".base { color: red; }".to_string(),
                },
                OmenaQueryStyleSourceInputV0 {
                    style_path: "/src/button.module.css".to_string(),
                    style_source: ".button { composes: base from './base.module.css'; }"
                        .to_string(),
                },
            ],
        ));

        assert!(response.ready, "{:?}", response.diagnostics);
        assert_eq!(
            response.class_map.get("button"),
            Some(&"_button_0 _base_0".to_string())
        );
        assert_eq!(
            response.named_exports.get("button"),
            response.class_map.get("button")
        );
        assert_eq!(response.composes_edges.len(), 1);
        assert_eq!(response.composes_edges[0].class_name, "base");
    }

    #[test]
    fn keeps_non_identifier_exports_only_on_the_default_map() {
        let response = resolve_omena_bundler_host_module_v0(request(
            "/src/tokens.module.css",
            vec![OmenaQueryStyleSourceInputV0 {
                style_path: "/src/tokens.module.css".to_string(),
                style_source: ".foo-bar { color: red; } .class { color: blue; }".to_string(),
            }],
        ));

        assert!(response.ready, "{:?}", response.diagnostics);
        assert!(response.class_map.contains_key("foo-bar"));
        assert!(response.class_map.contains_key("class"));
        assert!(!response.named_exports.contains_key("foo-bar"));
        assert!(!response.named_exports.contains_key("class"));
    }

    #[test]
    fn fails_closed_when_the_requested_module_is_absent() {
        let response = resolve_omena_bundler_host_module_v0(request(
            "/src/missing.module.css",
            vec![OmenaQueryStyleSourceInputV0 {
                style_path: "/src/present.module.css".to_string(),
                style_source: ".present { color: red; }".to_string(),
            }],
        ));

        assert!(!response.ready);
        assert!(response.class_map.is_empty());
        assert_eq!(response.diagnostics[0].code, "moduleNotFound");
    }
}

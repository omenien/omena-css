use crate::{
    OmenaBundlerHostCapabilitiesV0, OmenaBundlerHostComposesEdgeV0, OmenaBundlerHostDiagnosticV0,
    OmenaBundlerHostResolveModuleRequestV0, OmenaBundlerHostResolveModuleResponseV0,
    render_omena_query_css_module_typescript_declaration,
    summarize_omena_query_css_modules_interface_bundle,
};
use omena_syntax::ident::ClassNameV0;
use std::collections::{BTreeMap, BTreeSet};

pub const OMENA_BUNDLER_HOST_PROTOCOL_VERSION_V0: &str = "0";
const CSS_MODULE_CLASS_NAME_WHITESPACE_DIAGNOSTIC_V0: &str = "unsupportedClassNameWhitespace";

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
    let mut has_blocking_diagnostic = false;
    let typescript_declaration = render_omena_query_css_module_typescript_declaration(&module);

    let mut spellings_by_identity = BTreeMap::new();
    for export in &module.class_exports {
        spellings_by_identity
            .entry(ClassNameV0::new(&export.name).canonical_key())
            .or_insert_with(BTreeSet::new)
            .insert(export.name.clone());
    }
    for spellings in spellings_by_identity
        .into_values()
        .filter(|spellings| spellings.len() > 1)
    {
        diagnostics.push(OmenaBundlerHostDiagnosticV0 {
            code: "decodeEquivalentClassNames".to_string(),
            message: format!(
                "CSS Module class spellings {} decode to one identifier and share an emitted name.",
                spellings
                    .iter()
                    .map(|name| format!("'{name}'"))
                    .collect::<Vec<_>>()
                    .join(", ")
            ),
        });
    }

    for export in module.class_exports {
        let decoded_name = ClassNameV0::new(&export.name);
        if decoded_name
            .decoded()
            .chars()
            .any(|ch| ch.is_ascii_whitespace())
        {
            has_blocking_diagnostic = true;
            diagnostics.push(OmenaBundlerHostDiagnosticV0 {
                code: CSS_MODULE_CLASS_NAME_WHITESPACE_DIAGNOSTIC_V0.to_string(),
                message: format!(
                    "CSS Module '{}' exports class name '{}' which decodes to ASCII whitespace and cannot be represented as one DOM class token.",
                    module.style_path, export.name
                ),
            });
            continue;
        }
        if export.emitted_classes.len() != export.resolved_classes.len() {
            has_blocking_diagnostic = true;
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
        ready: !has_blocking_diagnostic,
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
    fn rejects_decoded_ascii_whitespace_but_not_non_ascii_class_names() {
        let whitespace = resolve_omena_bundler_host_module_v0(request(
            "/src/whitespace.module.css",
            vec![OmenaQueryStyleSourceInputV0 {
                style_path: "/src/whitespace.module.css".to_string(),
                style_source: r".a\20 b { color: red; }".to_string(),
            }],
        ));
        assert!(!whitespace.ready);
        let diagnostic = whitespace
            .diagnostics
            .iter()
            .find(|diagnostic| diagnostic.code == CSS_MODULE_CLASS_NAME_WHITESPACE_DIAGNOSTIC_V0)
            .expect("the build boundary must report decoded ASCII whitespace");
        assert!(diagnostic.message.contains("/src/whitespace.module.css"));
        assert!(diagnostic.message.contains(r"a\20 b"));

        let non_ascii = resolve_omena_bundler_host_module_v0(request(
            "/src/korean.module.css",
            vec![OmenaQueryStyleSourceInputV0 {
                style_path: "/src/korean.module.css".to_string(),
                style_source: ".카드 { color: red; }".to_string(),
            }],
        ));
        assert!(
            non_ascii.diagnostics.iter().all(|diagnostic| diagnostic.code
                != CSS_MODULE_CLASS_NAME_WHITESPACE_DIAGNOSTIC_V0),
            "non-ASCII class names are not whitespace diagnostics: {:?}",
            non_ascii.diagnostics
        );
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

    #[test]
    fn resolves_every_raw_export_through_decoded_class_identity() {
        let response = resolve_omena_bundler_host_module_v0(request(
            "/src/names.module.css",
            vec![OmenaQueryStyleSourceInputV0 {
                style_path: "/src/names.module.css".to_string(),
                style_source: r".a\62 c { color: red; } .abc { color: blue; } .z { color: green; }"
                    .to_string(),
            }],
        ));

        // Either one-sided canonicalization leaves one of these source-produced
        // raw keys unresolved, so the assertion names the exact missing key.
        let emitted_total = response
            .class_map
            .values()
            .map(|value| value.split_ascii_whitespace().count())
            .sum::<usize>();
        for raw_key in [r"a\62 c", "abc", "z"] {
            assert!(
                response.class_map.contains_key(raw_key),
                "missing emitted class for raw export key {raw_key:?}; emitted total {emitted_total}: {:?}",
                response.diagnostics
            );
        }
        assert_eq!(emitted_total, 3);
        assert!(response.ready, "{:?}", response.diagnostics);
    }

    #[test]
    fn decode_equivalent_exports_share_one_emitted_token() {
        let response = resolve_omena_bundler_host_module_v0(request(
            "/src/card.module.css",
            vec![OmenaQueryStyleSourceInputV0 {
                style_path: "/src/card.module.css".to_string(),
                style_source: r".card { color: red; } .c\61 rd { color: blue; }".to_string(),
            }],
        ));

        assert!(response.ready, "{:?}", response.diagnostics);
        assert!(
            response
                .diagnostics
                .iter()
                .all(|diagnostic| diagnostic.code != "unresolvedEmittedClass")
        );
        assert!(
            response
                .diagnostics
                .iter()
                .any(|diagnostic| diagnostic.code == "decodeEquivalentClassNames")
        );
        let plain = response.class_map.get("card");
        let escaped = response.class_map.get(r"c\61 rd");
        assert!(
            plain.is_some() && escaped.is_some(),
            "both raw export keys must remain public"
        );
        if let (Some(plain), Some(escaped)) = (plain, escaped) {
            assert_eq!(plain, escaped);
            assert_eq!(
                [plain, escaped]
                    .into_iter()
                    .flat_map(|value| value.split_ascii_whitespace())
                    .collect::<BTreeSet<_>>()
                    .len(),
                1
            );
        }
    }

    #[test]
    fn composes_resolves_a_target_spelled_only_by_the_importer() {
        let response = resolve_omena_bundler_host_module_v0(request(
            "/src/b.module.css",
            vec![
                OmenaQueryStyleSourceInputV0 {
                    style_path: "/src/a.module.css".to_string(),
                    style_source: ".card { color: red; }".to_string(),
                },
                OmenaQueryStyleSourceInputV0 {
                    style_path: "/src/b.module.css".to_string(),
                    style_source: r#".x { composes: c\61 rd from "./a.module.css"; color: blue; }"#
                        .to_string(),
                },
            ],
        ));

        assert!(response.ready, "{:?}", response.diagnostics);
        let emitted = response.class_map.get("x");
        assert!(emitted.is_some(), "the local export must remain public");
        if let Some(emitted) = emitted {
            assert!(
                emitted
                    .split_ascii_whitespace()
                    .any(|name| name == "_card_0"),
                "cross-module canonical identity did not resolve the target token: {emitted:?}"
            );
        }
    }

    #[test]
    fn escape_free_interface_emitted_class_total_is_stable() {
        let response = resolve_omena_bundler_host_module_v0(request(
            "/src/button.module.css",
            vec![
                OmenaQueryStyleSourceInputV0 {
                    style_path: "/src/base.module.css".to_string(),
                    style_source: ".base {} .quiet {}".to_string(),
                },
                OmenaQueryStyleSourceInputV0 {
                    style_path: "/src/button.module.css".to_string(),
                    style_source: ".button { composes: base from './base.module.css'; } .label {}"
                        .to_string(),
                },
            ],
        ));

        assert!(response.ready, "{:?}", response.diagnostics);
        let emitted_total = response
            .class_map
            .values()
            .map(|value| value.split_ascii_whitespace().count())
            .sum::<usize>();
        // The fixture itself supplies two local exports and one composed edge;
        // dropping either side of the join changes this independently counted total.
        assert_eq!(emitted_total, 3);
        assert_eq!(response.class_map.len(), 2);
    }
}

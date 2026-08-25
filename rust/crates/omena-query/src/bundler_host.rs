use crate::{
    OmenaBundlerHostCapabilitiesV0, OmenaBundlerHostComposesEdgeV0,
    OmenaBundlerHostDiagnosticSourceAnchorV0, OmenaBundlerHostDiagnosticV0,
    OmenaBundlerHostExportKindV0, OmenaBundlerHostNamedExportV0,
    OmenaBundlerHostResolveModuleRequestV0, OmenaBundlerHostResolveModuleResponseV0,
    render_omena_query_css_module_typescript_declaration,
    summarize_omena_query_css_modules_interface_bundle_with_module_identity_root,
};
use omena_syntax::ident::ClassNameV0;
use std::collections::{BTreeMap, BTreeSet};

pub const OMENA_BUNDLER_HOST_PROTOCOL_VERSION_V0: &str = "0";
const CSS_MODULE_CLASS_NAME_WHITESPACE_DIAGNOSTIC_V0: &str = "unsupportedClassNameWhitespace";

pub fn current_omena_bundler_host_capabilities_v0() -> OmenaBundlerHostCapabilitiesV0 {
    OmenaBundlerHostCapabilitiesV0 {
        protocol_version: OMENA_BUNDLER_HOST_PROTOCOL_VERSION_V0.to_string(),
        capabilities: vec![
            "semanticClassExports".to_string(),
            "typedExportNamespaces".to_string(),
            "namedExports".to_string(),
            "composesEdges".to_string(),
        ],
    }
}

pub fn resolve_omena_bundler_host_module_v0(
    request: OmenaBundlerHostResolveModuleRequestV0,
) -> OmenaBundlerHostResolveModuleResponseV0 {
    let bundle = match summarize_omena_query_css_modules_interface_bundle_with_module_identity_root(
        request.workspace_root.as_str(),
        request.style_sources.as_slice(),
        request.package_manifests.as_slice(),
    ) {
        Ok(bundle) => bundle,
        Err(message) => {
            return OmenaBundlerHostResolveModuleResponseV0 {
                snapshot_id: request.snapshot_id,
                protocol_version: OMENA_BUNDLER_HOST_PROTOCOL_VERSION_V0.to_string(),
                module_id: request.style_path,
                class_exports: BTreeMap::new(),
                value_exports: BTreeMap::new(),
                named_exports: Vec::new(),
                typescript_declaration: String::new(),
                composes_edges: Vec::new(),
                diagnostics: vec![OmenaBundlerHostDiagnosticV0 {
                    code: "invalidModuleIdentityRoot".to_string(),
                    message,
                    source_anchors: Vec::new(),
                }],
                ready: false,
            };
        }
    };
    let Some(module) = bundle
        .modules
        .into_iter()
        .find(|module| module.style_path == request.style_path)
    else {
        return OmenaBundlerHostResolveModuleResponseV0 {
            snapshot_id: request.snapshot_id,
            protocol_version: OMENA_BUNDLER_HOST_PROTOCOL_VERSION_V0.to_string(),
            module_id: request.style_path.clone(),
            class_exports: BTreeMap::new(),
            value_exports: BTreeMap::new(),
            named_exports: Vec::new(),
            typescript_declaration: String::new(),
            composes_edges: Vec::new(),
            diagnostics: vec![OmenaBundlerHostDiagnosticV0 {
                code: "moduleNotFound".to_string(),
                message: format!(
                    "CSS Module '{}' is not present in the bundler host snapshot.",
                    request.style_path
                ),
                source_anchors: Vec::new(),
            }],
            ready: false,
        };
    };

    let mut class_exports = BTreeMap::new();
    let mut value_exports = BTreeMap::new();
    let mut named_exports = Vec::new();
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
                "CSS Module class spellings {} decode to one identifier but retain distinct raw-spelling emitted names.",
                spellings
                    .iter()
                    .map(|name| format!("'{name}'"))
                    .collect::<Vec<_>>()
                    .join(", ")
            ),
            source_anchors: Vec::new(),
        });
    }

    let class_export_names = module
        .class_exports
        .iter()
        .map(|export| export.name.as_str())
        .collect::<BTreeSet<_>>();
    let value_export_names = module
        .icss_exports
        .iter()
        .map(|export| export.name.as_str())
        .collect::<BTreeSet<_>>();
    for collision_name in class_export_names.intersection(&value_export_names) {
        let class_export = module
            .class_exports
            .iter()
            .find(|export| export.name == **collision_name);
        let value_export = module
            .icss_exports
            .iter()
            .find(|export| export.name == **collision_name);
        let mut source_anchors = Vec::new();
        if let Some(export) = class_export {
            source_anchors.extend(export.source_spans.iter().map(|span| {
                OmenaBundlerHostDiagnosticSourceAnchorV0 {
                    style_path: module.style_path.clone(),
                    kind: OmenaBundlerHostExportKindV0::Class,
                    start_byte: span.start as u64,
                    end_byte: span.end as u64,
                }
            }));
        }
        if let Some(export) = value_export {
            source_anchors.extend(export.source_spans.iter().map(|span| {
                OmenaBundlerHostDiagnosticSourceAnchorV0 {
                    style_path: module.style_path.clone(),
                    kind: OmenaBundlerHostExportKindV0::Value,
                    start_byte: span.start as u64,
                    end_byte: span.end as u64,
                }
            }));
        }
        source_anchors.sort_by_key(|anchor| {
            (
                anchor.style_path.clone(),
                anchor.start_byte,
                anchor.end_byte,
                anchor.kind,
            )
        });
        diagnostics.push(OmenaBundlerHostDiagnosticV0 {
            code: "exportNamespaceCollision".to_string(),
            message: format!(
                "CSS Module '{}' exports '{}' as both a class and an ICSS value; the typed families remain separate and no flat named export can represent both.",
                module.style_path, collision_name
            ),
            source_anchors,
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
                source_anchors: export
                    .source_spans
                    .iter()
                    .map(|span| OmenaBundlerHostDiagnosticSourceAnchorV0 {
                        style_path: module.style_path.clone(),
                        kind: OmenaBundlerHostExportKindV0::Class,
                        start_byte: span.start as u64,
                        end_byte: span.end as u64,
                    })
                    .collect(),
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
                source_anchors: export
                    .source_spans
                    .iter()
                    .map(|span| OmenaBundlerHostDiagnosticSourceAnchorV0 {
                        style_path: module.style_path.clone(),
                        kind: OmenaBundlerHostExportKindV0::Class,
                        start_byte: span.start as u64,
                        end_byte: span.end as u64,
                    })
                    .collect(),
            });
            continue;
        }
        let value = export.emitted_classes.join(" ");
        class_exports.insert(export.name.clone(), value.clone());
        if let Some(named_export) = export.named_export {
            named_exports.push(OmenaBundlerHostNamedExportV0 {
                exported_name: named_export,
                kind: OmenaBundlerHostExportKindV0::Class,
                value,
            });
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
        value_exports.insert(export.name.clone(), export.value.clone());
        if let Some(named_export) = export.named_export {
            named_exports.push(OmenaBundlerHostNamedExportV0 {
                exported_name: named_export,
                kind: OmenaBundlerHostExportKindV0::Value,
                value: export.value,
            });
        }
    }
    named_exports.sort_by(|left, right| {
        (&left.exported_name, left.kind).cmp(&(&right.exported_name, right.kind))
    });

    OmenaBundlerHostResolveModuleResponseV0 {
        snapshot_id: request.snapshot_id,
        protocol_version: OMENA_BUNDLER_HOST_PROTOCOL_VERSION_V0.to_string(),
        module_id: module.module_id.as_str().to_string(),
        class_exports,
        value_exports,
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
            workspace_root: if style_path.starts_with('/') {
                "/"
            } else {
                "."
            }
            .to_string(),
            style_path: style_path.to_string(),
            style_sources,
            package_manifests: Vec::<OmenaQueryStylePackageManifestV0>::new(),
        }
    }

    fn named_export_value<'response>(
        response: &'response OmenaBundlerHostResolveModuleResponseV0,
        exported_name: &str,
        kind: OmenaBundlerHostExportKindV0,
    ) -> Option<&'response str> {
        response
            .named_exports
            .iter()
            .find(|entry| entry.exported_name == exported_name && entry.kind == kind)
            .map(|entry| entry.value.as_str())
    }

    #[test]
    fn caller_workspace_root_makes_module_tokens_relocation_stable() {
        let source = ".card { color: red; }";
        let mut first = request(
            "/workspace-a/src/card.module.css",
            vec![OmenaQueryStyleSourceInputV0 {
                style_path: "/workspace-a/src/card.module.css".to_string(),
                style_source: source.to_string(),
            }],
        );
        first.workspace_root = "/workspace-a".to_string();
        let mut second = request(
            "/workspace-b/src/card.module.css",
            vec![OmenaQueryStyleSourceInputV0 {
                style_path: "/workspace-b/src/card.module.css".to_string(),
                style_source: source.to_string(),
            }],
        );
        second.workspace_root = "/workspace-b".to_string();

        let first = resolve_omena_bundler_host_module_v0(first);
        let second = resolve_omena_bundler_host_module_v0(second);
        assert!(first.ready, "{:?}", first.diagnostics);
        assert!(second.ready, "{:?}", second.diagnostics);
        assert_eq!(first.class_exports, second.class_exports);
    }

    #[test]
    fn caller_workspace_root_rejects_out_of_root_module_identity() {
        let mut outside = request(
            "/outside/card.module.css",
            vec![OmenaQueryStyleSourceInputV0 {
                style_path: "/outside/card.module.css".to_string(),
                style_source: ".card { color: red; }".to_string(),
            }],
        );
        outside.workspace_root = "/workspace".to_string();

        let response = resolve_omena_bundler_host_module_v0(outside);
        assert!(!response.ready);
        assert_eq!(response.diagnostics.len(), 1);
        assert_eq!(response.diagnostics[0].code, "invalidModuleIdentityRoot");
    }

    #[test]
    fn resolves_scoped_classes_named_exports_and_composes_from_one_interface_view()
    -> Result<(), String> {
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
        let emitted = response
            .class_exports
            .get("button")
            .ok_or_else(|| "the local export must remain public".to_string())?
            .split_ascii_whitespace()
            .collect::<Vec<_>>();
        assert_eq!(emitted.len(), 2);
        assert!(emitted[0].ends_with("_button"), "{emitted:?}");
        assert!(emitted[1].ends_with("_base"), "{emitted:?}");
        assert_ne!(emitted[0], emitted[1]);
        assert_eq!(
            named_export_value(&response, "button", OmenaBundlerHostExportKindV0::Class),
            response.class_exports.get("button").map(String::as_str)
        );
        assert_eq!(response.composes_edges.len(), 1);
        assert_eq!(response.composes_edges[0].class_name, "base");
        Ok(())
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
        assert!(response.class_exports.contains_key("foo-bar"));
        assert!(response.class_exports.contains_key("class"));
        assert!(
            !response
                .named_exports
                .iter()
                .any(|entry| entry.exported_name == "foo-bar")
        );
        assert!(
            !response
                .named_exports
                .iter()
                .any(|entry| entry.exported_name == "class")
        );
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
        let whitespace_diagnostics = whitespace
            .diagnostics
            .iter()
            .filter(|diagnostic| diagnostic.code == CSS_MODULE_CLASS_NAME_WHITESPACE_DIAGNOSTIC_V0)
            .collect::<Vec<_>>();
        assert_eq!(whitespace_diagnostics.len(), 1);
        let diagnostic = whitespace_diagnostics[0];
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
        assert!(response.class_exports.is_empty());
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
            .class_exports
            .values()
            .map(|value| value.split_ascii_whitespace().count())
            .sum::<usize>();
        for raw_key in [r"a\62 c", "abc", "z"] {
            assert!(
                response.class_exports.contains_key(raw_key),
                "missing emitted class for raw export key {raw_key:?}; emitted total {emitted_total}: {:?}",
                response.diagnostics
            );
        }
        assert_eq!(emitted_total, 3);
        assert!(response.ready, "{:?}", response.diagnostics);
    }

    #[test]
    fn decode_equivalent_exports_keep_distinct_raw_identity_tokens() {
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
        let plain = response.class_exports.get("card");
        let escaped = response.class_exports.get(r"c\61 rd");
        assert!(
            plain.is_some() && escaped.is_some(),
            "both raw export keys must remain public"
        );
        if let (Some(plain), Some(escaped)) = (plain, escaped) {
            assert_ne!(plain, escaped);
            assert_eq!(
                [plain, escaped]
                    .into_iter()
                    .flat_map(|value| value.split_ascii_whitespace())
                    .collect::<BTreeSet<_>>()
                    .len(),
                2
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
        let emitted = response.class_exports.get("x");
        assert!(emitted.is_some(), "the local export must remain public");
        if let Some(emitted) = emitted {
            assert!(
                emitted
                    .split_ascii_whitespace()
                    .any(|name| name.ends_with("_card")),
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
            .class_exports
            .values()
            .map(|value| value.split_ascii_whitespace().count())
            .sum::<usize>();
        // The fixture itself supplies two local exports and one composed edge;
        // dropping either side of the join changes this independently counted total.
        assert_eq!(emitted_total, 3);
        assert_eq!(response.class_exports.len(), 2);
    }

    #[test]
    fn same_named_class_and_value_exports_remain_separate_and_diagnostic() {
        let response = resolve_omena_bundler_host_module_v0(request(
            "/src/collision.module.css",
            vec![OmenaQueryStyleSourceInputV0 {
                style_path: "/src/collision.module.css".to_string(),
                style_source: ":export { button: #0af; } .button { color: red; }".to_string(),
            }],
        ));
        let payload = serde_json::to_value(&response).expect("bundler response must serialize");

        let class_value = payload["classExports"]["button"]
            .as_str()
            .expect("class family must retain the emitted class token");
        assert!(class_value.ends_with("_button"), "{class_value:?}");
        assert_eq!(payload["valueExports"]["button"], "#0af");
        assert!(payload.get("classMap").is_none());

        let named = payload["namedExports"]
            .as_array()
            .expect("named exports must be typed entries");
        assert!(named.iter().any(|entry| {
            entry["exportedName"] == "button"
                && entry["kind"] == "class"
                && entry["value"] == class_value
        }));
        assert!(named.iter().any(|entry| {
            entry["exportedName"] == "button"
                && entry["kind"] == "value"
                && entry["value"] == "#0af"
        }));

        let collision = payload["diagnostics"]
            .as_array()
            .and_then(|diagnostics| {
                diagnostics
                    .iter()
                    .find(|diagnostic| diagnostic["code"] == "exportNamespaceCollision")
            })
            .expect("same-name collision must be diagnosed");
        let anchors = collision["sourceAnchors"]
            .as_array()
            .expect("collision diagnostic must be source-anchored");
        assert_eq!(anchors.len(), 2);
        assert!(anchors.iter().any(|anchor| anchor["kind"] == "class"));
        assert!(anchors.iter().any(|anchor| anchor["kind"] == "value"));
        assert!(anchors.iter().all(|anchor| {
            anchor["stylePath"] == "/src/collision.module.css"
                && anchor["startByte"].as_u64() < anchor["endByte"].as_u64()
        }));
    }
}

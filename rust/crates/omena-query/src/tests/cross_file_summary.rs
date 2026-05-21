use std::collections::{BTreeMap, BTreeSet};

use crate::{
    OmenaQuerySourceDocumentInputV0, OmenaQueryStylePackageManifestV0,
    OmenaQueryStyleSourceInputV0,
    summarize_omena_query_source_selector_reference_cross_file_summary,
    summarize_omena_query_style_semantic_graph_batch_from_sources,
    summarize_omena_query_workspace_cross_file_summary,
};

use super::sample_input;

#[test]
fn source_selector_references_emit_cross_file_summary_edges() {
    let summary = summarize_omena_query_source_selector_reference_cross_file_summary(
        &[OmenaQueryStyleSourceInputV0 {
            style_path: "/tmp/Button.module.scss".to_string(),
            style_source: ".root { color: red; }".to_string(),
        }],
        &[OmenaQuerySourceDocumentInputV0 {
            source_path: "/tmp/Button.tsx".to_string(),
            source_source: "import styles from './Button.module.scss';\nconst cls = styles.root;\n"
                .to_string(),
        }],
        &[],
    );

    assert_eq!(summary.product, "omena-query.cross-file-summary");
    assert_eq!(summary.status, "sourceSelectorSummaryEdgeSeed");
    assert_eq!(summary.summary_scope, "sourceSelectorReferences");
    assert_eq!(summary.summary_edge_count, 1);
    assert_eq!(summary.summary_hash.len(), 16);
    assert!(summary.capabilities.source_selector_reference_edges_ready);
    assert!(!summary.capabilities.css_modules_composes_edges_ready);

    let edge = &summary.edges[0];
    assert_eq!(edge.edge_kind, "sourceSelectorReference");
    assert_eq!(edge.from_kind, "source");
    assert_eq!(edge.from_path, "/tmp/Button.tsx");
    assert_eq!(edge.target_kind, Some("style"));
    assert_eq!(edge.target_path.as_deref(), Some("/tmp/Button.module.scss"));
    assert_eq!(edge.local_name.as_deref(), Some("root"));
    assert_eq!(edge.target_names, vec!["root"]);
    assert_eq!(edge.status, "resolved");
    assert_eq!(
        edge.provenance,
        vec![
            "omena-query.source-selector-references",
            "omena-query.style-selector-definitions",
        ]
    );
    assert_eq!(edge.linear_provenance.labels(), edge.provenance);
    assert_eq!(edge.linear_provenance.semiring_identifier(), "lin01");
    assert_eq!(edge.linear_provenance.semiring_identifier, "lin01");
}

#[test]
fn workspace_cross_file_summary_merges_style_and_source_edge_sets() {
    let input = sample_input();
    let style_sources = vec![
        OmenaQueryStyleSourceInputV0 {
            style_path: "/tmp/base.module.scss".to_string(),
            style_source: ".base { display: block; }".to_string(),
        },
        OmenaQueryStyleSourceInputV0 {
            style_path: "/tmp/Button.module.scss".to_string(),
            style_source:
                ".root { composes: base from \"./base.module.scss\"; color: var(--brand); }"
                    .to_string(),
        },
    ];
    let source_documents = vec![OmenaQuerySourceDocumentInputV0 {
        source_path: "/tmp/Button.tsx".to_string(),
        source_source: "import styles from './Button.module.scss';\nconst cls = styles.root;\n"
            .to_string(),
    }];

    let style_batch = summarize_omena_query_style_semantic_graph_batch_from_sources(
        style_sources
            .iter()
            .map(|source| (source.style_path.as_str(), source.style_source.as_str())),
        &input,
    );
    let source_summary = summarize_omena_query_source_selector_reference_cross_file_summary(
        style_sources.as_slice(),
        source_documents.as_slice(),
        &[],
    );
    let workspace_summary = summarize_omena_query_workspace_cross_file_summary(
        style_sources.as_slice(),
        source_documents.as_slice(),
        &[],
    );

    assert_eq!(workspace_summary.product, "omena-query.cross-file-summary");
    assert_eq!(workspace_summary.status, "workspaceSummaryEdgeSeed");
    assert_eq!(workspace_summary.summary_scope, "workspaceStyleAndSource");
    assert_eq!(
        workspace_summary.summary_edge_count,
        style_batch.cross_file_summary.summary_edge_count + source_summary.summary_edge_count
    );
    assert!(
        workspace_summary
            .capabilities
            .css_modules_composes_edges_ready
    );
    assert!(
        workspace_summary
            .capabilities
            .style_design_token_reference_edges_ready
    );
    assert!(
        workspace_summary
            .capabilities
            .source_selector_reference_edges_ready
    );
    assert!(workspace_summary.capabilities.stable_summary_hash_ready);
    assert!(workspace_summary.capabilities.linear_provenance_ready);

    let workspace_edge_ids = workspace_summary
        .edges
        .iter()
        .map(|edge| edge.edge_id.as_str())
        .collect::<BTreeSet<_>>();
    assert!(
        style_batch
            .cross_file_summary
            .edges
            .iter()
            .all(|edge| workspace_edge_ids.contains(edge.edge_id.as_str()))
    );
    assert!(
        source_summary
            .edges
            .iter()
            .all(|edge| workspace_edge_ids.contains(edge.edge_id.as_str()))
    );
}

#[test]
fn workspace_cross_file_summary_linear_provenance_covers_merged_style_and_source_edges()
-> Result<(), serde_json::Error> {
    let style_sources = vec![
        OmenaQueryStyleSourceInputV0 {
            style_path: "/tmp/base.module.scss".to_string(),
            style_source: ".base { display: block; }".to_string(),
        },
        OmenaQueryStyleSourceInputV0 {
            style_path: "/tmp/Button.module.scss".to_string(),
            style_source: ".root { composes: base from \"./base.module.scss\"; }".to_string(),
        },
    ];
    let source_documents = vec![OmenaQuerySourceDocumentInputV0 {
        source_path: "/tmp/Button.tsx".to_string(),
        source_source: "import styles from './Button.module.scss';\nconst cls = styles.root;\n"
            .to_string(),
    }];
    let workspace_summary = summarize_omena_query_workspace_cross_file_summary(
        style_sources.as_slice(),
        source_documents.as_slice(),
        &[],
    );
    let serialized = serde_json::to_value(&workspace_summary)?;
    let serialized_edges = serialized
        .pointer("/edges")
        .and_then(|value| value.as_array())
        .expect("workspace summary edges must be serialized");

    assert!(serialized_edges.iter().any(|edge| {
        edge.pointer("/edgeKind").and_then(|value| value.as_str())
            == Some("cssModulesComposesImport")
    }));
    assert!(serialized_edges.iter().any(|edge| {
        edge.pointer("/edgeKind").and_then(|value| value.as_str())
            == Some("sourceSelectorReference")
    }));
    for edge in serialized_edges {
        let legacy_labels = edge
            .pointer("/provenance")
            .and_then(|value| value.as_array())
            .expect("legacy provenance vector must stay serialized")
            .iter()
            .map(|value| value.as_str().expect("provenance labels are strings"))
            .collect::<Vec<_>>();
        let typed_terms = edge
            .pointer("/linearProvenance/terms")
            .and_then(|value| value.as_array())
            .expect("typed linear provenance terms must be serialized");
        let typed_labels = typed_terms
            .iter()
            .map(|value| {
                value
                    .pointer("/label")
                    .and_then(|label| label.as_str())
                    .expect("linear provenance labels are strings")
            })
            .collect::<Vec<_>>();

        assert_eq!(typed_labels, legacy_labels);
        assert_eq!(
            edge.pointer("/linearProvenance/product")
                .and_then(|value| value.as_str()),
            Some("omena-abstract-value.linear-provenance")
        );
        assert_eq!(
            edge.pointer("/linearProvenance/semiringIdentifier")
                .and_then(|value| value.as_str()),
            Some("lin01")
        );
        assert_eq!(
            edge.pointer("/linearProvenance/termCount")
                .and_then(|value| value.as_u64()),
            Some(legacy_labels.len() as u64)
        );
        assert!(typed_terms.iter().all(|value| {
            value
                .pointer("/coefficient")
                .and_then(|coefficient| coefficient.as_u64())
                == Some(1)
        }));
    }
    Ok(())
}

#[test]
fn workspace_cross_file_summary_hash_tracks_source_selector_changes() {
    let style_sources = vec![OmenaQueryStyleSourceInputV0 {
        style_path: "/tmp/Button.module.scss".to_string(),
        style_source: ".root { color: red; }".to_string(),
    }];
    let baseline = summarize_omena_query_workspace_cross_file_summary(
        style_sources.as_slice(),
        &[OmenaQuerySourceDocumentInputV0 {
            source_path: "/tmp/Button.tsx".to_string(),
            source_source: "import styles from './Button.module.scss';\nconst cls = styles.root;\n"
                .to_string(),
        }],
        &[],
    );
    let changed = summarize_omena_query_workspace_cross_file_summary(
        style_sources.as_slice(),
        &[OmenaQuerySourceDocumentInputV0 {
            source_path: "/tmp/Button.tsx".to_string(),
            source_source:
                "import styles from './Button.module.scss';\nconst cls = styles.missing;\n"
                    .to_string(),
        }],
        &[],
    );

    assert_ne!(baseline.summary_hash, changed.summary_hash);
    assert!(baseline.edges.iter().any(|edge| {
        edge.edge_kind == "sourceSelectorReference"
            && edge.local_name.as_deref() == Some("root")
            && edge.status == "resolved"
    }));
    assert!(changed.edges.iter().any(|edge| {
        edge.edge_kind == "sourceSelectorReference"
            && edge.local_name.as_deref() == Some("missing")
            && edge.status == "unresolved"
    }));
}

#[test]
fn workspace_cross_file_summary_hash_tracks_style_edge_changes() {
    let source_documents = vec![OmenaQuerySourceDocumentInputV0 {
        source_path: "/tmp/Button.tsx".to_string(),
        source_source: "import styles from './Button.module.scss';\nconst cls = styles.root;\n"
            .to_string(),
    }];
    let baseline = summarize_omena_query_workspace_cross_file_summary(
        &[
            OmenaQueryStyleSourceInputV0 {
                style_path: "/tmp/base.module.scss".to_string(),
                style_source: ".base { display: block; }".to_string(),
            },
            OmenaQueryStyleSourceInputV0 {
                style_path: "/tmp/Button.module.scss".to_string(),
                style_source: ".root { composes: base from \"./base.module.scss\"; color: red; }"
                    .to_string(),
            },
        ],
        source_documents.as_slice(),
        &[],
    );
    let changed = summarize_omena_query_workspace_cross_file_summary(
        &[
            OmenaQueryStyleSourceInputV0 {
                style_path: "/tmp/base.module.scss".to_string(),
                style_source: ".base { display: block; }".to_string(),
            },
            OmenaQueryStyleSourceInputV0 {
                style_path: "/tmp/Button.module.scss".to_string(),
                style_source:
                    ".root { composes: base from \"./missing.module.scss\"; color: red; }"
                        .to_string(),
            },
        ],
        source_documents.as_slice(),
        &[],
    );

    assert_ne!(baseline.summary_hash, changed.summary_hash);
    assert!(baseline.edges.iter().any(|edge| {
        edge.edge_kind == "cssModulesComposesImport"
            && edge.target_path.as_deref() == Some("/tmp/base.module.scss")
            && edge.status == "resolved"
    }));
    assert!(changed.edges.iter().any(|edge| {
        edge.edge_kind == "cssModulesComposesImport"
            && edge.target_path.is_none()
            && edge.status == "unresolvedSource"
    }));
    assert!(baseline.edges.iter().any(|edge| {
        edge.edge_kind == "sourceSelectorReference"
            && edge.local_name.as_deref() == Some("root")
            && edge.status == "resolved"
    }));
    assert!(changed.edges.iter().any(|edge| {
        edge.edge_kind == "sourceSelectorReference"
            && edge.local_name.as_deref() == Some("root")
            && edge.status == "resolved"
    }));
}

#[test]
fn workspace_cross_file_summary_hash_tracks_package_manifest_changes() {
    let style_sources = vec![
        OmenaQueryStyleSourceInputV0 {
            style_path: "/fake/workspace/node_modules/@design/tokens/dist/theme.css".to_string(),
            style_source: ":root { --brand: theme; }".to_string(),
        },
        OmenaQueryStyleSourceInputV0 {
            style_path: "/fake/workspace/node_modules/@design/tokens/dist/alt.css".to_string(),
            style_source: ":root { --brand: alt; }".to_string(),
        },
        OmenaQueryStyleSourceInputV0 {
            style_path: "/fake/workspace/src/Button.module.scss".to_string(),
            style_source: "@use \"@design/tokens/theme\";\n.root { color: var(--brand); }"
                .to_string(),
        },
    ];
    let source_documents = vec![OmenaQuerySourceDocumentInputV0 {
        source_path: "/fake/workspace/src/Button.tsx".to_string(),
        source_source: "import styles from './Button.module.scss';\nconst cls = styles.root;\n"
            .to_string(),
    }];
    let baseline_manifest = vec![OmenaQueryStylePackageManifestV0 {
        package_json_path: "/fake/workspace/node_modules/@design/tokens/package.json".to_string(),
        package_json_source: r#"{"exports":{"./theme":{"style":"./dist/theme.css"}}}"#.to_string(),
    }];
    let changed_manifest = vec![OmenaQueryStylePackageManifestV0 {
        package_json_path: "/fake/workspace/node_modules/@design/tokens/package.json".to_string(),
        package_json_source: r#"{"exports":{"./theme":{"style":"./dist/alt.css"}}}"#.to_string(),
    }];
    let baseline = summarize_omena_query_workspace_cross_file_summary(
        style_sources.as_slice(),
        source_documents.as_slice(),
        baseline_manifest.as_slice(),
    );
    let changed = summarize_omena_query_workspace_cross_file_summary(
        style_sources.as_slice(),
        source_documents.as_slice(),
        changed_manifest.as_slice(),
    );

    assert_ne!(baseline.summary_hash, changed.summary_hash);
    assert!(baseline.edges.iter().any(|edge| {
        edge.edge_kind == "sassUse"
            && edge.source.as_deref() == Some("@design/tokens/theme")
            && edge.target_path.as_deref()
                == Some("/fake/workspace/node_modules/@design/tokens/dist/theme.css")
            && edge.status == "resolved"
    }));
    assert!(changed.edges.iter().any(|edge| {
        edge.edge_kind == "sassUse"
            && edge.source.as_deref() == Some("@design/tokens/theme")
            && edge.target_path.as_deref()
                == Some("/fake/workspace/node_modules/@design/tokens/dist/alt.css")
            && edge.status == "resolved"
    }));
    assert!(baseline.edges.iter().any(|edge| {
        edge.edge_kind == "sourceSelectorReference"
            && edge.local_name.as_deref() == Some("root")
            && edge.status == "resolved"
    }));
    assert!(changed.edges.iter().any(|edge| {
        edge.edge_kind == "sourceSelectorReference"
            && edge.local_name.as_deref() == Some("root")
            && edge.status == "resolved"
    }));
}

#[test]
fn workspace_cross_file_summary_resolves_imported_design_token_references() {
    let style_sources = vec![
        OmenaQueryStyleSourceInputV0 {
            style_path: "/fake/workspace/node_modules/@design/tokens/dist/theme.css".to_string(),
            style_source: ":root { --brand: theme; }".to_string(),
        },
        OmenaQueryStyleSourceInputV0 {
            style_path: "/fake/workspace/node_modules/@design/tokens/dist/alt.css".to_string(),
            style_source: ":root { --brand: alt; }".to_string(),
        },
        OmenaQueryStyleSourceInputV0 {
            style_path: "/fake/workspace/src/Button.module.scss".to_string(),
            style_source: "@use \"@design/tokens/theme\";\n.root { color: var(--brand); }"
                .to_string(),
        },
    ];
    let source_documents = vec![OmenaQuerySourceDocumentInputV0 {
        source_path: "/fake/workspace/src/Button.tsx".to_string(),
        source_source: "import styles from './Button.module.scss';\nconst cls = styles.root;\n"
            .to_string(),
    }];
    let baseline_manifest = vec![OmenaQueryStylePackageManifestV0 {
        package_json_path: "/fake/workspace/node_modules/@design/tokens/package.json".to_string(),
        package_json_source: r#"{"exports":{"./theme":{"style":"./dist/theme.css"}}}"#.to_string(),
    }];
    let changed_manifest = vec![OmenaQueryStylePackageManifestV0 {
        package_json_path: "/fake/workspace/node_modules/@design/tokens/package.json".to_string(),
        package_json_source: r#"{"exports":{"./theme":{"style":"./dist/alt.css"}}}"#.to_string(),
    }];

    let baseline = summarize_omena_query_workspace_cross_file_summary(
        style_sources.as_slice(),
        source_documents.as_slice(),
        baseline_manifest.as_slice(),
    );
    let changed = summarize_omena_query_workspace_cross_file_summary(
        style_sources.as_slice(),
        source_documents.as_slice(),
        changed_manifest.as_slice(),
    );

    assert_ne!(baseline.summary_hash, changed.summary_hash);
    assert!(baseline.edges.iter().any(|edge| {
        edge.edge_kind == "styleDesignTokenReference"
            && edge.from_path == "/fake/workspace/src/Button.module.scss"
            && edge.local_name.as_deref() == Some("--brand")
            && edge.target_path.as_deref()
                == Some("/fake/workspace/node_modules/@design/tokens/dist/theme.css")
            && edge.status == "importResolved"
            && edge.provenance
                == vec![
                    "omena-query.style-semantic-graph-batch",
                    "omena-parser.custom-property-facts",
                    "omena-query.sass-module-cross-file-resolution",
                ]
    }));
    assert!(changed.edges.iter().any(|edge| {
        edge.edge_kind == "styleDesignTokenReference"
            && edge.from_path == "/fake/workspace/src/Button.module.scss"
            && edge.local_name.as_deref() == Some("--brand")
            && edge.target_path.as_deref()
                == Some("/fake/workspace/node_modules/@design/tokens/dist/alt.css")
            && edge.status == "importResolved"
    }));
    assert!(baseline.edges.iter().all(|edge| {
        edge.edge_kind != "styleDesignTokenReference"
            || edge.local_name.as_deref() != Some("--brand")
            || edge.status != "unresolvedReference"
    }));
}

#[test]
fn workspace_cross_file_summary_reports_edge_kind_counts_for_m4_vocabulary() {
    let summary = summarize_omena_query_workspace_cross_file_summary(
        &[
            OmenaQueryStyleSourceInputV0 {
                style_path: "/tmp/base.module.scss".to_string(),
                style_source: ".base { display: block; }".to_string(),
            },
            OmenaQueryStyleSourceInputV0 {
                style_path: "/tmp/_tokens.scss".to_string(),
                style_source: ":root { --brand: red; }".to_string(),
            },
            OmenaQueryStyleSourceInputV0 {
                style_path: "/tmp/_palette.scss".to_string(),
                style_source: "$tone: red;".to_string(),
            },
            OmenaQueryStyleSourceInputV0 {
                style_path: "/tmp/_legacy.scss".to_string(),
                style_source: "$legacy: blue;".to_string(),
            },
            OmenaQueryStyleSourceInputV0 {
                style_path: "/tmp/Button.module.scss".to_string(),
                style_source: r#"@use "./tokens" as tokens;
@forward "./palette";
@import "./legacy";
.root { composes: base from "./base.module.scss"; color: var(--brand); }"#
                    .to_string(),
            },
        ],
        &[OmenaQuerySourceDocumentInputV0 {
            source_path: "/tmp/Button.tsx".to_string(),
            source_source: "import styles from './Button.module.scss';\nconst cls = styles.root;\n"
                .to_string(),
        }],
        &[],
    );

    let declared_counts = summary
        .edge_kind_counts
        .iter()
        .map(|entry| (entry.edge_kind, entry.count))
        .collect::<BTreeMap<_, _>>();
    let observed_counts =
        summary
            .edges
            .iter()
            .fold(BTreeMap::<&str, usize>::new(), |mut counts, edge| {
                *counts.entry(edge.edge_kind).or_default() += 1;
                counts
            });

    assert_eq!(declared_counts, observed_counts);
    assert_eq!(
        declared_counts.values().copied().sum::<usize>(),
        summary.summary_edge_count
    );
    for required_edge_kind in [
        "cssModulesComposesImport",
        "cssModulesComposesClosure",
        "sassUse",
        "sassForward",
        "sassImport",
        "sassModuleGraphClosure",
        "styleDesignTokenReference",
        "sourceSelectorReference",
    ] {
        assert!(
            declared_counts
                .get(required_edge_kind)
                .copied()
                .unwrap_or(0)
                > 0,
            "missing M4 summary-edge vocabulary count for {required_edge_kind}"
        );
    }
}

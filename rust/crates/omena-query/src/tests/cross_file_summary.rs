#![allow(clippy::expect_used)]

use std::collections::{BTreeMap, BTreeSet};

#[cfg(feature = "hypergraph-ifds")]
use crate::{
    OmenaQueryCrossFileSummaryV0, UnifiedHypergraphEdgeKindV0,
    summarize_omena_query_unified_cross_file_hypergraph,
    summarize_omena_query_unified_cross_file_scc_report,
};
use crate::{
    OmenaQuerySourceDocumentInputV0, OmenaQueryStylePackageManifestV0,
    OmenaQueryStyleSourceInputV0,
    read_workspace_cross_file_summary_direct_recompute_count_for_test,
    reset_workspace_cross_file_summary_direct_recompute_count_for_test,
    summarize_omena_query_categorical_design_system_cross_project_summary,
    summarize_omena_query_m4_axis_c_readiness,
    summarize_omena_query_source_selector_reference_cross_file_summary,
    summarize_omena_query_style_document,
    summarize_omena_query_style_semantic_graph_batch_from_sources,
    summarize_omena_query_workspace_cross_file_summary,
};
#[cfg(feature = "salsa-memo")]
use crate::{
    read_committed_style_semantic_graph_compute_count_for_test,
    reset_committed_style_semantic_graph_compute_count_for_test,
};

use super::support::sample_input;

#[cfg(feature = "hypergraph-ifds")]
#[test]
fn cross_file_hypergraph_projects_summary_edges_byte_equal_by_id() {
    let summary = hypergraph_summary_fixture();
    let hypergraph = summarize_omena_query_unified_cross_file_hypergraph(&summary);

    assert_eq!(hypergraph.schema_version, "0");
    assert_eq!(hypergraph.layer_marker, "hypergraph-ifds");
    assert_eq!(hypergraph.summary_edge_count, summary.summary_edge_count);
    assert!(
        hypergraph
            .gate_predicates
            .contains(&"P6.closureBodySwitchOver")
    );
    assert_eq!(
        hypergraph
            .projection_edge_ids
            .iter()
            .map(String::as_str)
            .collect::<BTreeSet<_>>(),
        summary
            .edges
            .iter()
            .map(|edge| edge.edge_id.as_str())
            .collect::<BTreeSet<_>>()
    );
}

#[cfg(feature = "hypergraph-ifds")]
#[test]
fn cross_file_hypergraph_composes_tail_preserves_target_name_order() {
    let mut summary = hypergraph_summary_fixture();
    let edge = summary
        .edges
        .iter_mut()
        .find(|edge| edge.edge_kind == "cssModulesComposesImport")
        .expect("composes summary edge");
    edge.target_names = ["a", "b", "c"].into_iter().map(str::to_string).collect();
    let edge_id = edge.edge_id.clone();

    let hypergraph = summarize_omena_query_unified_cross_file_hypergraph(&summary);
    let tail = &hypergraph
        .hyperedges
        .iter()
        .find(|edge| {
            edge.edge_kind == UnifiedHypergraphEdgeKindV0::ComposesExternal
                && edge.source_summary_edge_id == edge_id
        })
        .expect("composes hyperedge")
        .tail_node_ids;

    assert_eq!(tail.len(), 3);
    assert!(tail[0].ends_with("|a"));
    assert!(tail[1].ends_with("|b"));
    assert!(tail[2].ends_with("|c"));
}

#[cfg(feature = "hypergraph-ifds")]
#[test]
fn cross_file_hypergraph_reports_exact_tarjan_scc_for_composes_cycle() -> Result<(), &'static str> {
    let summary = summarize_omena_query_workspace_cross_file_summary(
        &[
            OmenaQueryStyleSourceInputV0 {
                style_path: "/tmp/a.module.scss".to_string(),
                style_source: r#".a { composes: b from "./b.module.scss"; }"#.to_string(),
            },
            OmenaQueryStyleSourceInputV0 {
                style_path: "/tmp/b.module.scss".to_string(),
                style_source: r#".b { composes: a from "./a.module.scss"; }"#.to_string(),
            },
        ],
        &[],
        &[],
    );
    let hypergraph = summarize_omena_query_unified_cross_file_hypergraph(&summary);
    let report = summarize_omena_query_unified_cross_file_scc_report(&hypergraph);

    assert_eq!(report.product, "omena-query.unified-cross-file-scc-report");
    assert_eq!(report.feature_gate, "cross-file-scc-v0");
    assert_eq!(report.claim_level, "fixtureWitnessExactTarjanScc");
    assert!(!report.theorem_claimed);
    assert_eq!(report.connectivity_backend, "exactTarjanScc");
    assert_eq!(report.polylog_bound_scope, "notClaimedExactTraversal");

    let composes_scc = report
        .sccs
        .iter()
        .find(|scc| {
            scc.cross_file
                && scc.edge_kinds.contains(&"composesExternal")
                && scc.style_paths.contains(&"/tmp/a.module.scss".to_string())
                && scc.style_paths.contains(&"/tmp/b.module.scss".to_string())
        })
        .ok_or("cross-file composes SCC")?;
    assert_eq!(composes_scc.node_count, 2);
    assert_eq!(composes_scc.connectivity_backend, "exactTarjanScc");
    assert_eq!(composes_scc.polylog_bound_scope, "notClaimedExactTraversal");
    assert!(!composes_scc.theorem_claimed);
    Ok(())
}

#[cfg(feature = "hypergraph-ifds")]
fn hypergraph_summary_fixture() -> OmenaQueryCrossFileSummaryV0 {
    summarize_omena_query_workspace_cross_file_summary(
        &[
            OmenaQueryStyleSourceInputV0 {
                style_path: "/tmp/base.module.scss".into(),
                style_source: ".base { color: red; }".into(),
            },
            OmenaQueryStyleSourceInputV0 {
                style_path: "/tmp/Button.module.scss".into(),
                style_source: ".root { composes: base from \"./base.module.scss\"; }".into(),
            },
        ],
        &[OmenaQuerySourceDocumentInputV0 {
            source_path: "/tmp/Button.tsx".into(),
            source_source: "import styles from './Button.module.scss';\nconst cls = styles.root;\n"
                .into(),
            source_syntax_index: None,
            has_unresolved_style_import: false,
        }],
        &[],
    )
}

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
            source_syntax_index: None,
            has_unresolved_style_import: false,
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
    assert_eq!(edge.linear_provenance.semiring_identifier(), "naturalCount");
    assert_eq!(edge.linear_provenance.semiring_identifier, "naturalCount");
    assert!(summary.capabilities.linear_provenance_semiring_laws_hold);
}

#[test]
fn source_selector_prefix_summary_uses_semiring_support_count_for_targets() {
    let one_target = summarize_omena_query_source_selector_reference_cross_file_summary(
        &[OmenaQueryStyleSourceInputV0 {
            style_path: "/tmp/Button.module.scss".to_string(),
            style_source: ".btn-primary { color: red; }".to_string(),
        }],
        &[OmenaQuerySourceDocumentInputV0 {
            source_path: "/tmp/Button.tsx".to_string(),
            source_source: r#"import bind from "classnames/bind";
import styles from "./Button.module.scss";
const cx = bind.bind(styles);
export function Button({ variant }) {
  return <div className={cx(`btn-${variant}`)} />;
}"#
            .to_string(),
            source_syntax_index: None,
            has_unresolved_style_import: false,
        }],
        &[],
    );
    let two_targets = summarize_omena_query_source_selector_reference_cross_file_summary(
        &[OmenaQueryStyleSourceInputV0 {
            style_path: "/tmp/Button.module.scss".to_string(),
            style_source: ".btn-primary { color: red; } .btn-secondary { color: blue; }"
                .to_string(),
        }],
        &[OmenaQuerySourceDocumentInputV0 {
            source_path: "/tmp/Button.tsx".to_string(),
            source_source: r#"import bind from "classnames/bind";
import styles from "./Button.module.scss";
const cx = bind.bind(styles);
export function Button({ variant }) {
  return <div className={cx(`btn-${variant}`)} />;
}"#
            .to_string(),
            source_syntax_index: None,
            has_unresolved_style_import: false,
        }],
        &[],
    );

    let one_target_edge = one_target
        .edges
        .iter()
        .find(|edge| edge.edge_kind == "sourceSelectorPrefixReference")
        .expect("one-target prefix edge");
    let two_target_edge = two_targets
        .edges
        .iter()
        .find(|edge| edge.edge_kind == "sourceSelectorPrefixReference")
        .expect("two-target prefix edge");

    assert_eq!(one_target_edge.target_names, vec!["btn-primary"]);
    assert_eq!(
        two_target_edge.target_names,
        vec!["btn-primary", "btn-secondary"]
    );
    assert!(
        one_target_edge
            .linear_provenance
            .terms
            .iter()
            .all(|term| term.coefficient == 1)
    );
    assert!(
        two_target_edge
            .linear_provenance
            .terms
            .iter()
            .all(|term| term.coefficient == 2)
    );

    let mut coefficient_changed = two_targets.clone();
    coefficient_changed
        .edges
        .iter_mut()
        .find(|edge| edge.edge_kind == "sourceSelectorPrefixReference")
        .expect("coefficient-mutated prefix edge")
        .linear_provenance
        .terms[0]
        .coefficient = 1;
    assert_eq!(
        two_targets.summary_hash,
        two_targets.recompute_stable_summary_hash()
    );
    assert_ne!(
        two_targets.summary_hash,
        coefficient_changed.recompute_stable_summary_hash()
    );
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
        source_syntax_index: None,
        has_unresolved_style_import: false,
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
    assert!(
        workspace_summary
            .capabilities
            .linear_provenance_round_trip_ready
    );
    assert!(workspace_summary.linear_provenance_round_trips_legacy_labels());

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
fn workspace_cross_file_summary_feeds_categorical_cross_project_theory() {
    let project_a = workspace_summary_with_source_selector_refs(
        "/tmp/project-a/Button.module.scss",
        "/tmp/project-a/Button.tsx",
        ".root { color: red; }",
        "import styles from './Button.module.scss';\nconst cls = styles.root;\n",
    );
    let project_b = workspace_summary_with_source_selector_refs(
        "/tmp/project-b/Card.module.scss",
        "/tmp/project-b/Card.tsx",
        ".root { color: blue; }",
        "import styles from './Card.module.scss';\nconst cls = styles.root;\n",
    );
    let changed_project = workspace_summary_with_source_selector_refs(
        "/tmp/project-c/Button.module.scss",
        "/tmp/project-c/Button.tsx",
        ".root { color: red; } .icon { color: blue; }",
        "import styles from './Button.module.scss';\nconst cls = `${styles.root} ${styles.icon}`;\n",
    );

    let accepted = summarize_omena_query_categorical_design_system_cross_project_summary(&[
        ("project-a", &project_a),
        ("project-b", &project_b),
    ]);
    let rejected = summarize_omena_query_categorical_design_system_cross_project_summary(&[
        ("project-a", &project_a),
        ("project-c", &changed_project),
    ]);

    assert_eq!(
        accepted.product,
        "omena-query.categorical-design-system-cross-project-summary"
    );
    assert_eq!(
        accepted.claim_scope,
        "productPathComputedCrossProjectSummary"
    );
    assert!(accepted.product_path_evidence_ready);
    assert_eq!(accepted.project_count, 2);
    assert_eq!(accepted.models.len(), 2);
    assert!(
        accepted
            .models
            .iter()
            .all(|model| model.source_product == "omena-query.cross-file-summary")
    );
    assert!(accepted.invariant_summary.accepted);
    assert_eq!(
        accepted.invariant_summary.invariant_kind,
        "crossProjectEdgeKindSymmetry"
    );
    assert!(
        accepted
            .deferred_residuals
            .contains(&"rust/omena-categorical/verify-cross-project-symmetry")
    );

    assert!(rejected.product_path_evidence_ready);
    assert!(!rejected.invariant_summary.accepted);
    assert!(
        rejected
            .invariant_summary
            .differing_sort_names
            .contains(&"edgeKind:sourceSelectorReference".to_string())
    );
}

fn workspace_summary_with_source_selector_refs(
    style_path: &str,
    source_path: &str,
    style_source: &str,
    source_source: &str,
) -> crate::OmenaQueryCrossFileSummaryV0 {
    summarize_omena_query_workspace_cross_file_summary(
        &[OmenaQueryStyleSourceInputV0 {
            style_path: style_path.to_string(),
            style_source: style_source.to_string(),
        }],
        &[OmenaQuerySourceDocumentInputV0 {
            source_path: source_path.to_string(),
            source_source: source_source.to_string(),
            source_syntax_index: None,
            has_unresolved_style_import: false,
        }],
        &[],
    )
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
        source_syntax_index: None,
        has_unresolved_style_import: false,
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
            Some("naturalCount")
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
    assert!(workspace_summary.linear_provenance_round_trips_legacy_labels());
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
            source_syntax_index: None,
            has_unresolved_style_import: false,
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
            source_syntax_index: None,
            has_unresolved_style_import: false,
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
        source_syntax_index: None,
        has_unresolved_style_import: false,
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
fn workspace_cross_file_summary_hash_is_input_order_stable() {
    let style_sources = vec![
        OmenaQueryStyleSourceInputV0 {
            style_path: "/fake/workspace/node_modules/@design/tokens/dist/theme.css".to_string(),
            style_source: ":root { --brand: red; }".to_string(),
        },
        OmenaQueryStyleSourceInputV0 {
            style_path: "/fake/workspace/node_modules/@design/palette/dist/palette.css"
                .to_string(),
            style_source: ":root { --accent: blue; }".to_string(),
        },
        OmenaQueryStyleSourceInputV0 {
            style_path: "/fake/workspace/src/base.module.scss".to_string(),
            style_source: ".base { display: block; }".to_string(),
        },
        OmenaQueryStyleSourceInputV0 {
            style_path: "/fake/workspace/src/_tokens.scss".to_string(),
            style_source: ":root { --brand: red; }".to_string(),
        },
        OmenaQueryStyleSourceInputV0 {
            style_path: "/fake/workspace/src/Button.module.scss".to_string(),
            style_source:
                "@use \"@design/tokens/theme\";\n@use \"@design/palette/theme\";\n.root { composes: base from \"./base.module.scss\"; color: var(--brand); border-color: var(--accent); }"
                    .to_string(),
        },
    ];
    let reordered_style_sources = vec![
        style_sources[4].clone(),
        style_sources[2].clone(),
        style_sources[0].clone(),
        style_sources[3].clone(),
        style_sources[1].clone(),
    ];
    let source_documents = vec![
        OmenaQuerySourceDocumentInputV0 {
            source_path: "/fake/workspace/src/Button.tsx".to_string(),
            source_source: "import styles from './Button.module.scss';\nconst cls = styles.root;\n"
                .to_string(),
            source_syntax_index: None,
            has_unresolved_style_import: false,
        },
        OmenaQuerySourceDocumentInputV0 {
            source_path: "/fake/workspace/src/Card.tsx".to_string(),
            source_source:
                "import buttonStyles from './Button.module.scss';\nconst cls = buttonStyles.root;\n"
                    .to_string(),
            source_syntax_index: None,
            has_unresolved_style_import: false,
        },
    ];
    let reordered_source_documents = vec![source_documents[1].clone(), source_documents[0].clone()];
    let package_manifests = vec![
        OmenaQueryStylePackageManifestV0 {
            package_json_path: "/fake/workspace/node_modules/@design/tokens/package.json"
                .to_string(),
            package_json_source: r#"{"exports":{"./theme":{"style":"./dist/theme.css"}}}"#
                .to_string(),
        },
        OmenaQueryStylePackageManifestV0 {
            package_json_path: "/fake/workspace/node_modules/@design/palette/package.json"
                .to_string(),
            package_json_source: r#"{"exports":{"./theme":{"style":"./dist/palette.css"}}}"#
                .to_string(),
        },
    ];
    let reordered_package_manifests =
        vec![package_manifests[1].clone(), package_manifests[0].clone()];

    let baseline = summarize_omena_query_workspace_cross_file_summary(
        style_sources.as_slice(),
        source_documents.as_slice(),
        package_manifests.as_slice(),
    );
    let reordered = summarize_omena_query_workspace_cross_file_summary(
        reordered_style_sources.as_slice(),
        reordered_source_documents.as_slice(),
        reordered_package_manifests.as_slice(),
    );
    let baseline_edge_ids = baseline
        .edges
        .iter()
        .map(|edge| edge.edge_id.as_str())
        .collect::<Vec<_>>();
    let reordered_edge_ids = reordered
        .edges
        .iter()
        .map(|edge| edge.edge_id.as_str())
        .collect::<Vec<_>>();

    assert_eq!(baseline.summary_hash, reordered.summary_hash);
    assert_eq!(baseline_edge_ids, reordered_edge_ids);
    assert_eq!(baseline.edge_kind_counts, reordered.edge_kind_counts);
    assert!(baseline.edges.iter().any(|edge| {
        edge.edge_kind == "styleDesignTokenReference"
            && edge.local_name.as_deref() == Some("--accent")
            && edge.target_path.as_deref()
                == Some("/fake/workspace/node_modules/@design/palette/dist/palette.css")
    }));
    assert!(baseline.edges.iter().any(|edge| {
        edge.edge_kind == "sourceSelectorReference"
            && edge.from_path == "/fake/workspace/src/Card.tsx"
    }));
    assert!(baseline.linear_provenance_round_trips_legacy_labels());
    assert!(reordered.linear_provenance_round_trips_legacy_labels());
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
        source_syntax_index: None,
        has_unresolved_style_import: false,
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
        source_syntax_index: None,
        has_unresolved_style_import: false,
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
            source_syntax_index: None,
            has_unresolved_style_import: false,
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

#[test]
fn workspace_cross_file_summary_reports_less_module_edges_with_less_vocabulary() {
    let summary = summarize_omena_query_workspace_cross_file_summary(
        &[
            OmenaQueryStyleSourceInputV0 {
                style_path: "/tmp/tokens.less".to_string(),
                style_source: "@brand: red;".to_string(),
            },
            OmenaQueryStyleSourceInputV0 {
                style_path: "/tmp/Button.module.less".to_string(),
                style_source: r#"@import "./tokens.less"; .button { color: @brand; }"#.to_string(),
            },
        ],
        &[],
        &[],
    );

    let declared_counts = summary
        .edge_kind_counts
        .iter()
        .map(|entry| (entry.edge_kind, entry.count))
        .collect::<BTreeMap<_, _>>();

    assert!(summary.capabilities.sass_module_edges_ready);
    assert_eq!(declared_counts.get("lessImport").copied(), Some(1));
    assert_eq!(
        declared_counts
            .get("lessModuleGraphClosure")
            .copied()
            .unwrap_or(0),
        1
    );
    assert!(!declared_counts.contains_key("sassImport"));
    assert!(summary.edges.iter().any(|edge| {
        edge.edge_kind == "lessImport"
            && edge.from_path == "/tmp/Button.module.less"
            && edge.source.as_deref() == Some("./tokens.less")
            && edge.target_path.as_deref() == Some("/tmp/tokens.less")
            && edge.status == "resolved"
    }));
    assert!(summary.edges.iter().any(|edge| {
        edge.edge_kind == "lessModuleGraphClosure"
            && edge.from_path == "/tmp/Button.module.less"
            && edge.target_path.as_deref() == Some("/tmp/tokens.less")
            && edge.status == "reachable"
    }));
}

#[cfg(feature = "hypergraph-ifds")]
#[test]
fn cross_file_hypergraph_projects_less_module_edges_to_less_edge_kinds() {
    let summary = summarize_omena_query_workspace_cross_file_summary(
        &[
            OmenaQueryStyleSourceInputV0 {
                style_path: "/tmp/tokens.less".to_string(),
                style_source: "@brand: red;".to_string(),
            },
            OmenaQueryStyleSourceInputV0 {
                style_path: "/tmp/Button.module.less".to_string(),
                style_source: r#"@import "./tokens.less"; .button { color: @brand; }"#.to_string(),
            },
        ],
        &[],
        &[],
    );
    let hypergraph = summarize_omena_query_unified_cross_file_hypergraph(&summary);

    assert!(hypergraph.hyperedges.iter().any(|edge| {
        edge.edge_kind == UnifiedHypergraphEdgeKindV0::LessImport
            && edge.source_edge_kind == "lessImport"
    }));
    assert!(hypergraph.hyperedges.iter().any(|edge| {
        edge.edge_kind == UnifiedHypergraphEdgeKindV0::LessModuleGraphClosure
            && edge.source_edge_kind == "lessModuleGraphClosure"
    }));
}

#[test]
fn cross_file_summary_edges_are_equivalent_to_resolution_products() {
    let input = sample_input();
    let style_sources = [
        (
            "/tmp/base.module.scss",
            ".foundation { display: block; } .base { composes: foundation; --brand: red; }",
        ),
        (
            "/tmp/tokens.module.scss",
            "@value primary: red; :export { raw: red; exported: raw; }",
        ),
        ("/tmp/_palette.scss", "$tone: red;"),
        ("/tmp/_theme.scss", "@forward \"./palette\" show $tone;"),
        (
            "/tmp/App.module.scss",
            "@use \"./theme\"; @value primary as localPrimary from \"./tokens.module.scss\"; @value accent: localPrimary; :import(\"./tokens.module.scss\") { imported: exported; } :export { forwarded: imported; } .btn { composes: base from \"./base.module.scss\"; color: var(--brand); }",
        ),
    ];
    let batch =
        summarize_omena_query_style_semantic_graph_batch_from_sources(style_sources, &input);
    let summary = &batch.cross_file_summary;

    let custom_property_reference_count = style_sources
        .iter()
        .filter_map(|(path, source)| summarize_omena_query_style_document(path, source))
        .map(|summary| summary.custom_property_ref_names.len())
        .sum::<usize>();
    let expected_summary_edge_count = batch.css_modules_resolution.edges.len()
        + batch.css_modules_resolution.composes_closure_edges.len()
        + batch.css_modules_resolution.value_closure_edges.len()
        + batch.css_modules_resolution.icss_closure_edges.len()
        + batch.sass_module_resolution.edges.len()
        + batch.sass_module_resolution.graph_closure_edges.len()
        + custom_property_reference_count;

    assert_eq!(summary.summary_edge_count, expected_summary_edge_count);
    assert_eq!(summary.edges.len(), expected_summary_edge_count);

    for edge in &batch.css_modules_resolution.edges {
        let edge_kind = match edge.import_kind {
            "composes" => "cssModulesComposesImport",
            "value" => "cssModulesValueImport",
            "icss" => "cssModulesIcssImport",
            _ => "cssModulesImport",
        };
        assert!(
            summary.edges.iter().any(|summary_edge| {
                summary_edge.edge_kind == edge_kind
                    && summary_edge.from_path == edge.from_style_path
                    && summary_edge.source.as_deref() == Some(edge.source.as_str())
                    && summary_edge.target_path == edge.resolved_style_path
                    && summary_edge.target_names == edge.imported_names
                    && summary_edge.status == edge.status
            }),
            "missing CSS Modules import summary edge for {edge_kind} {}",
            edge.source
        );
    }

    for edge in &batch.css_modules_resolution.composes_closure_edges {
        assert!(summary.edges.iter().any(|summary_edge| {
            summary_edge.edge_kind == "cssModulesComposesClosure"
                && summary_edge.from_path == edge.from_style_path
                && summary_edge.target_path.as_deref() == Some(edge.target_style_path.as_str())
                && summary_edge.owner_selector_name.as_deref()
                    == Some(edge.owner_selector_name.as_str())
                && summary_edge.remote_name.as_deref() == Some(edge.target_selector_name.as_str())
                && summary_edge.target_names == vec![edge.target_selector_name.clone()]
                && summary_edge.status == "reachable"
        }));
    }

    for edge in &batch.css_modules_resolution.value_closure_edges {
        assert!(summary.edges.iter().any(|summary_edge| {
            summary_edge.edge_kind == "cssModulesValueClosure"
                && summary_edge.from_path == edge.from_style_path
                && summary_edge.target_path.as_deref() == Some(edge.target_style_path.as_str())
                && summary_edge.local_name.as_deref() == Some(edge.value_name.as_str())
                && summary_edge.remote_name.as_deref() == Some(edge.target_value_name.as_str())
                && summary_edge.target_names == vec![edge.target_value_name.clone()]
                && summary_edge.status == "reachable"
        }));
    }

    for edge in &batch.css_modules_resolution.icss_closure_edges {
        assert!(summary.edges.iter().any(|summary_edge| {
            summary_edge.edge_kind == "cssModulesIcssClosure"
                && summary_edge.from_path == edge.from_style_path
                && summary_edge.target_path.as_deref() == Some(edge.target_style_path.as_str())
                && summary_edge.local_name.as_deref() == Some(edge.name.as_str())
                && summary_edge.remote_name.as_deref() == Some(edge.target_name.as_str())
                && summary_edge.target_names == vec![edge.target_name.clone()]
                && summary_edge.status == "reachable"
        }));
    }

    for edge in &batch.sass_module_resolution.edges {
        assert!(summary.edges.iter().any(|summary_edge| {
            summary_edge.edge_kind == edge.edge_kind
                && summary_edge.from_path == edge.from_style_path
                && summary_edge.source.as_deref() == Some(edge.source.as_str())
                && summary_edge.target_path == edge.resolved_style_path
                && summary_edge.local_name == edge.namespace
                && summary_edge.remote_name == edge.forward_prefix
                && summary_edge.target_names == edge.visibility_filter_names
                && summary_edge.status == edge.status
        }));
    }

    for edge in &batch.sass_module_resolution.graph_closure_edges {
        assert!(summary.edges.iter().any(|summary_edge| {
            summary_edge.edge_kind == "sassModuleGraphClosure"
                && summary_edge.from_path == edge.from_style_path
                && summary_edge.target_path.as_deref() == Some(edge.target_style_path.as_str())
                && summary_edge.local_name == edge.namespace
                && summary_edge.remote_name == edge.forward_prefix
                && summary_edge.target_names == edge.visibility_filter_names
                && summary_edge.status == "reachable"
        }));
    }

    assert_eq!(
        summary
            .edges
            .iter()
            .filter(|edge| edge.edge_kind == "styleDesignTokenReference")
            .count(),
        custom_property_reference_count
    );
    assert!(summary.edges.iter().all(|edge| {
        edge.linear_provenance.semiring_identifier() == "naturalCount"
            && edge.linear_provenance.labels() == edge.provenance
    }));
}

#[test]
fn style_semantic_graph_batch_cross_file_summary_hash_tracks_edge_changes() {
    let input = sample_input();
    let baseline = summarize_omena_query_style_semantic_graph_batch_from_sources(
        [
            ("/tmp/base.module.scss", ".base { display: block; }"),
            (
                "/tmp/App.module.scss",
                ".btn { composes: base from \"./base.module.scss\"; }",
            ),
        ],
        &input,
    );
    let changed = summarize_omena_query_style_semantic_graph_batch_from_sources(
        [
            ("/tmp/base.module.scss", ".base { display: block; }"),
            (
                "/tmp/App.module.scss",
                ".btn { composes: base from \"./missing.module.scss\"; }",
            ),
        ],
        &input,
    );

    assert_ne!(
        baseline.cross_file_summary.summary_hash,
        changed.cross_file_summary.summary_hash
    );
    assert!(baseline.cross_file_summary.edges.iter().any(|edge| {
        edge.edge_kind == "cssModulesComposesImport"
            && edge.target_path.as_deref() == Some("/tmp/base.module.scss")
            && edge.status == "resolved"
    }));
    assert!(changed.cross_file_summary.edges.iter().any(|edge| {
        edge.edge_kind == "cssModulesComposesImport"
            && edge.target_path.is_none()
            && edge.status == "unresolvedSource"
    }));
}

#[test]
fn cross_file_summary_linear_provenance_serializes_as_strict_superset()
-> Result<(), serde_json::Error> {
    let input = sample_input();
    let batch = summarize_omena_query_style_semantic_graph_batch_from_sources(
        [
            ("/tmp/base.module.scss", ".base { display: block; }"),
            (
                "/tmp/App.module.scss",
                ".btn { composes: base from \"./base.module.scss\"; }",
            ),
        ],
        &input,
    );
    let edge = batch
        .cross_file_summary
        .edges
        .iter()
        .find(|edge| edge.edge_kind == "cssModulesComposesImport")
        .expect("expected CSS Modules composes summary edge");
    let serialized = serde_json::to_value(edge)?;
    let legacy_labels = serialized
        .pointer("/provenance")
        .and_then(|value| value.as_array())
        .expect("legacy provenance vector must stay serialized")
        .iter()
        .map(|value| value.as_str().expect("provenance labels are strings"))
        .collect::<Vec<_>>();
    let typed_labels = serialized
        .pointer("/linearProvenance/terms")
        .and_then(|value| value.as_array())
        .expect("typed linear provenance terms must be serialized")
        .iter()
        .map(|value| {
            value
                .pointer("/label")
                .and_then(|label| label.as_str())
                .expect("linear provenance labels are strings")
        })
        .collect::<Vec<_>>();

    assert_eq!(legacy_labels, edge.provenance);
    assert_eq!(typed_labels, legacy_labels);
    assert_eq!(
        serialized
            .pointer("/linearProvenance/product")
            .and_then(|value| value.as_str()),
        Some("omena-abstract-value.linear-provenance")
    );
    assert_eq!(
        serialized
            .pointer("/linearProvenance/layerMarker")
            .and_then(|value| value.as_str()),
        Some("qtt-graded")
    );
    assert_eq!(
        serialized
            .pointer("/linearProvenance/featureGate")
            .and_then(|value| value.as_str()),
        Some("qtt-provenance")
    );
    assert_eq!(
        serialized
            .pointer("/linearProvenance/semiringIdentifier")
            .and_then(|value| value.as_str()),
        Some("naturalCount")
    );
    assert_eq!(
        serialized
            .pointer("/linearProvenance/termCount")
            .and_then(|value| value.as_u64()),
        Some(edge.provenance.len() as u64)
    );
    assert!(
        serialized
            .pointer("/linearProvenance/terms")
            .and_then(|value| value.as_array())
            .expect("typed linear provenance terms must be serialized")
            .iter()
            .all(|value| value
                .pointer("/coefficient")
                .and_then(|coefficient| coefficient.as_u64())
                == Some(1))
    );
    Ok(())
}

#[test]
fn m4_axis_c_readiness_summary_proves_exit_predicate_slice() {
    let summary = summarize_omena_query_m4_axis_c_readiness();

    assert_eq!(summary.product, "omena-query.m4-axis-c-readiness");
    assert_eq!(summary.status, "m4AxisCReady");
    assert_eq!(summary.required_edge_kind_count, 14);
    assert!(summary.issue_63_provenance_round_trip_ready);
    assert!(summary.issue_65_summary_edge_equivalence_ready);
    assert!(summary.summary_hash_invalidation_ready);
    assert!(summary.next_priorities.is_empty());
    assert!(
        summary
            .required_edge_kind_counts
            .iter()
            .all(|entry| entry.count > 0),
        "all M4 Axis C edge kinds must have fixture evidence: {summary:#?}"
    );
    let required_counts = summary
        .required_edge_kind_counts
        .iter()
        .map(|entry| (entry.edge_kind, entry.count))
        .collect::<BTreeMap<_, _>>();
    assert_eq!(required_counts.get("lessImport").copied(), Some(1));
    assert_eq!(
        required_counts
            .get("lessModuleGraphClosure")
            .copied()
            .unwrap_or(0),
        1
    );
    assert_ne!(
        summary.summary_hash_samples.baseline,
        summary.summary_hash_samples.source_selector_change
    );
    assert_ne!(
        summary.summary_hash_samples.baseline,
        summary.summary_hash_samples.style_edge_change
    );
    assert_ne!(
        summary.summary_hash_samples.baseline,
        summary.summary_hash_samples.package_manifest_change
    );
}

#[test]
fn workspace_cross_file_summary_direct_api_uses_expected_execution_path() {
    let style_sources = vec![
        OmenaQueryStyleSourceInputV0 {
            style_path: "/tmp/base.module.scss".to_string(),
            style_source: ".base { color: red; }".to_string(),
        },
        OmenaQueryStyleSourceInputV0 {
            style_path: "/tmp/button.module.scss".to_string(),
            style_source: r#".button { composes: base from "./base.module.scss"; }"#.to_string(),
        },
    ];

    reset_workspace_cross_file_summary_direct_recompute_count_for_test();
    #[cfg(feature = "salsa-memo")]
    reset_committed_style_semantic_graph_compute_count_for_test();
    let first = summarize_omena_query_workspace_cross_file_summary(&style_sources, &[], &[]);
    let second = summarize_omena_query_workspace_cross_file_summary(&style_sources, &[], &[]);

    assert_eq!(first.summary_hash, second.summary_hash);
    #[cfg(not(feature = "salsa-memo"))]
    assert_eq!(
        read_workspace_cross_file_summary_direct_recompute_count_for_test(),
        2,
        "the direct workspace summary API records one recompute per request",
    );
    #[cfg(feature = "salsa-memo")]
    {
        assert_eq!(
            read_workspace_cross_file_summary_direct_recompute_count_for_test(),
            0,
            "feature-enabled workspace summary should read committed graph selectors",
        );
        assert_eq!(
            read_committed_style_semantic_graph_compute_count_for_test(),
            2,
            "each public direct request should commit one graph before lookup",
        );
    }
}

#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    reason = "Snapshot fixtures fail immediately on an unexpected success, refusal, or stale read"
)]

use std::collections::BTreeMap;

use omena_query::{
    IncrementalRevisionV0, OmenaQuerySourceDocumentInputV0, OmenaQueryStylePackageManifestV0,
    OmenaQueryStyleResolutionInputsV0, OmenaQueryStyleSourceInputV0,
    OmenaWorkspaceSnapshotBindingV0, OmenaWorkspaceSnapshotIdV0, OmenaWorkspaceSnapshotInputsV0,
    OmenaWorkspaceSnapshotPublisherV0, OmenaWorkspaceSnapshotSettingsV0,
};

fn revision(value: u64) -> OmenaWorkspaceSnapshotIdV0 {
    OmenaWorkspaceSnapshotIdV0::from_revision(IncrementalRevisionV0 { value })
}

fn admitted_edge(
    importer: omena_query::OmenaQueryExternalSifImportOriginV0,
    specifier: &str,
    target: &omena_query::OmenaQueryExternalSifInputV0,
) -> omena_query::OmenaQueryExternalSifResolutionEdgeV0 {
    omena_query::OmenaQueryExternalSifResolutionEdgeV0 {
        importer,
        specifier: specifier.to_string(),
        resolved_style_url: target.sif.canonical_url.clone(),
        sif_canonical_url: target.sif.canonical_url.clone(),
        trust: omena_query::OmenaQueryExternalSifTrustV1 {
            canonical_url: target.sif.canonical_url.clone(),
            trust_tier: omena_sif::OmenaSifTrustTierV1::T1,
            trust_source: omena_query::OmenaQueryExternalSifTrustSourceV1::UnsignedLegacy,
        },
        sif_artifact_hash: omena_sif::compute_omena_sif_artifact_hash_v1(&target.sif)
            .unwrap()
            .as_str()
            .to_string(),
    }
}

#[test]
fn sif_projection_follows_admitted_targets_and_transitive_dependencies_without_disk()
-> Result<(), Box<dyn std::error::Error>> {
    fn fact(url: &str, source: &str) -> omena_query::OmenaQueryExternalSifInputV0 {
        omena_query::OmenaQueryExternalSifInputV0 {
            admitted_resolution_edges: Vec::new(),
            canonical_url: "./tokens".to_string(),
            sif: omena_sif::generate_static_omena_sif_v1(
                omena_sif::OmenaSifStaticGeneratorInputV1 {
                    canonical_url: url,
                    source,
                    syntax: omena_sif::OmenaSifSourceSyntaxV1::Scss,
                },
            )
            .expect("static fixture must generate"),
        }
    }
    let a_url = "file:///snapshot-no-disk/a/_tokens.scss";
    let b_url = "file:///snapshot-no-disk/b/_tokens.scss";
    let forward_url = "file:///snapshot-no-disk/shared/_forward.scss";
    let dependency_url = "file:///snapshot-no-disk/shared/_dependency.scss";
    let dependency = fact(dependency_url, "$gap: 4px;");
    let mut a = fact(a_url, "@forward '../shared/forward';\n$brand: red;");
    a.sif
        .dependencies
        .push(omena_sif::OmenaSifDependencyInterfaceHashV1 {
            canonical_url: dependency_url.to_string(),
            interface_hash: dependency.sif.fingerprints.interface_hash.clone(),
        });
    let mut facts = vec![
        fact(b_url, "$brand: blue;"),
        dependency,
        a,
        fact(forward_url, "$accent: green;"),
    ];
    let trust = facts
        .iter()
        .map(|input| {
            (
                input.sif.canonical_url.clone(),
                omena_query::OmenaQueryExternalSifTrustV1 {
                    canonical_url: input.sif.canonical_url.clone(),
                    trust_tier: omena_sif::OmenaSifTrustTierV1::T1,
                    trust_source: omena_query::OmenaQueryExternalSifTrustSourceV1::UnsignedLegacy,
                },
            )
        })
        .collect();
    let styles = vec![OmenaQueryStyleSourceInputV0 {
        style_path: "file:///snapshot-no-disk/a/App.scss".to_string(),
        style_source: "@use './tokens' as tokens; .card { color: tokens.$brand; }".to_string(),
    }];
    use omena_query::OmenaQueryExternalSifImportOriginV0 as Origin;
    let root_edge = admitted_edge(
        Origin::Document {
            style_path: styles[0].style_path.clone(),
        },
        "./tokens",
        &facts[2],
    );
    let parent = Origin::Sif {
        initiating_document: Some(styles[0].style_path.clone()),
        resolved_style_url: a_url.to_string(),
        artifact_hash: root_edge.sif_artifact_hash.clone(),
    };
    let edges = vec![
        root_edge,
        admitted_edge(parent.clone(), "../shared/forward", &facts[3]),
        admitted_edge(parent, dependency_url, &facts[1]),
    ];
    let select = |facts: &[omena_query::OmenaQueryExternalSifInputV0]| {
        omena_query::select_omena_workspace_snapshot_external_sifs_v0(
            &styles,
            &Default::default(),
            facts,
            &trust,
            &edges,
        )
    };
    let selected = select(&facts)?;
    assert_eq!(
        selected
            .0
            .iter()
            .map(|input| input.sif.canonical_url.as_str())
            .collect::<std::collections::BTreeSet<_>>(),
        [a_url, forward_url, dependency_url].into_iter().collect()
    );
    assert_eq!(selected.1.len(), 3);
    assert!(!selected.1.contains_key(b_url));
    facts.reverse();
    assert_eq!(
        select(&facts)?,
        selected,
        "global admission order cannot change the root commitment"
    );
    facts.retain(|input| input.sif.canonical_url != a_url);
    facts.push(fact(a_url, "$brand: contradictory;"));
    assert_eq!(
        select(&facts)
            .expect_err("the admitted edge must match the full artifact digest")
            .context
            .code,
        "workspace.snapshot-sif-admission"
    );
    Ok(())
}

#[test]
fn repeated_relative_sif_aliases_preserve_each_contexts_complete_diagnostic_payload()
-> Result<(), Box<dyn std::error::Error>> {
    let mut styles = Vec::new();
    let mut facts = Vec::new();
    for (directory, symbol) in [("one", "red"), ("two", "blue")] {
        styles.push(OmenaQueryStyleSourceInputV0 {
            style_path: format!("file:///snapshot-alias-root/{directory}/App.scss"),
            style_source: format!("@use './tokens' as t; .{directory} {{ color: t.${symbol}; }}"),
        });
        facts.push(omena_query::OmenaQueryExternalSifInputV0 {
            admitted_resolution_edges: Vec::new(),
            canonical_url: "./tokens".to_string(),
            sif: omena_sif::generate_static_omena_sif_v1(
                omena_sif::OmenaSifStaticGeneratorInputV1 {
                    canonical_url: &format!("file:///snapshot-alias-root/{directory}/_tokens.scss"),
                    source: &format!("${symbol}: {symbol};"),
                    syntax: omena_sif::OmenaSifSourceSyntaxV1::Scss,
                },
            )?,
        });
    }
    let resolution = OmenaQueryStyleResolutionInputsV0::default();
    let edges = styles
        .iter()
        .zip(&facts)
        .map(|(style, fact)| {
            admitted_edge(
                omena_query::OmenaQueryExternalSifImportOriginV0::Document {
                    style_path: style.style_path.clone(),
                },
                "./tokens",
                fact,
            )
        })
        .collect::<Vec<_>>();
    for (fact, edge) in facts.iter_mut().zip(&edges) {
        fact.admitted_resolution_edges = vec![edge.clone()];
    }
    let (selected, _, _) = omena_query::select_omena_workspace_snapshot_external_sifs_v0(
        &styles,
        &resolution,
        &facts,
        &BTreeMap::new(),
        &edges,
    )?;
    assert_eq!(
        selected.len(),
        2,
        "both full target facts survive alias disambiguation"
    );
    for (index, style) in styles.iter().enumerate() {
        let query = |sifs: &[omena_query::OmenaQueryExternalSifInputV0]| {
            omena_query::summarize_omena_query_style_diagnostics_for_workspace_file_with_external_mode_and_sifs_and_resolution_inputs(
                &style.style_path, &styles, &[], &[], None,
                omena_query::OmenaQueryExternalModuleModeV0::Sif, sifs, &resolution,
            ).expect("actual query consumer must find the target style")
        };
        let expected = serde_json::to_vec(&query(std::slice::from_ref(&facts[index])))?;
        let actual = serde_json::to_vec(&query(&selected))?;
        let actual_value = serde_json::to_value(query(&selected))?;
        assert!(
            actual_value["diagnostics"]
                .as_array()
                .unwrap()
                .iter()
                .all(|diagnostic| {
                    !matches!(
                        diagnostic["code"].as_str(),
                        Some("missing-module" | "missingSassSymbol")
                    )
                }),
            "valid admitted relative consumers must resolve their actual symbols"
        );
        let wrong = serde_json::to_vec(&query(std::slice::from_ref(&facts[1 - index])))?;
        if let Some(directory) = std::env::var_os("OMENA_WORKSPACE_SNAPSHOT_TEST_RECEIPTS") {
            let file = std::fs::OpenOptions::new()
                .write(true)
                .create_new(true)
                .open(std::path::PathBuf::from(directory).join(format!("alias-{index}.json")))?;
            serde_json::to_writer_pretty(
                file,
                &serde_json::json!({
                    "stylePath":style.style_path, "styleSources":styles, "selectedSifs":selected,
                    "expectedPayloadBytes":String::from_utf8(expected.clone())?,
                    "actualPayloadBytes":String::from_utf8(actual.clone())?,
                    "wrongTargetPayloadBytes":String::from_utf8(wrong.clone())?,
                }),
            )?;
        }
        assert_eq!(
            actual, expected,
            "full diagnostics must match the independently unambiguous target oracle"
        );
        assert_ne!(
            actual, wrong,
            "this fixture must detect selecting the other directory's SIF"
        );
    }
    assert!(
        selected
            .iter()
            .all(|fact| fact.canonical_url == fact.sif.canonical_url)
    );
    Ok(())
}

#[derive(Default)]
struct Fixture {
    styles: Vec<OmenaQueryStyleSourceInputV0>,
    sources: Vec<OmenaQuerySourceDocumentInputV0>,
    languages: BTreeMap<String, String>,
    providers: BTreeMap<String, omena_query::OmenaWorkspaceSourceProviderInputsV0>,
    manifests: Vec<OmenaQueryStylePackageManifestV0>,
    resolution: OmenaQueryStyleResolutionInputsV0,
    trust: BTreeMap<String, omena_query::OmenaQueryExternalSifTrustV1>,
    settings: OmenaWorkspaceSnapshotSettingsV0,
    complete: bool,
}

impl Fixture {
    fn normal() -> Self {
        Self {
            styles: vec![OmenaQueryStyleSourceInputV0 {
                style_path: "file:///workspace/card.module.css".to_string(),
                style_source: ".card { color: red; }".to_string(),
            }],
            ..Self::default()
        }
    }

    fn inputs(&self) -> OmenaWorkspaceSnapshotInputsV0<'_> {
        OmenaWorkspaceSnapshotInputsV0 {
            workspace_root: "file:///workspace",
            style_sources: &self.styles,
            source_documents: &self.sources,
            source_language_ids: &self.languages,
            source_provider_inputs: &self.providers,
            package_manifests: &self.manifests,
            external_sifs: &[],
            external_sif_trust_records: &self.trust,
            external_sif_resolution_edges: &[],
            resolution_inputs: &self.resolution,
            settings: &self.settings,
            source_corpus_complete: self.complete,
        }
    }
}

#[test]
fn imported_binding_reads_actual_input_and_refuses_changed_receivers() -> Result<(), String> {
    let fixture = Fixture::normal();
    let mut issuer = OmenaWorkspaceSnapshotPublisherV0::default();
    let binding = issuer
        .publish(fixture.inputs(), revision(17))
        .map_err(|error| error.to_string())?;
    let mut receiver = OmenaWorkspaceSnapshotPublisherV0::default();
    receiver
        .import(&binding, fixture.inputs())
        .map_err(|error| error.to_string())?;
    let read_view = receiver
        .reader()
        .read_view(&binding, fixture.inputs())
        .map_err(|error| error.to_string())?;
    assert_eq!(read_view.binding(), &binding);
    assert_eq!(
        read_view.document_bytes("file:///workspace/card.module.css"),
        Some(".card { color: red; }".as_bytes()),
    );

    let mut changed = Fixture::normal();
    changed.styles[0].style_source = ".card { color: blue; }".to_string();
    let refused = OmenaWorkspaceSnapshotPublisherV0::default()
        .import(&binding, changed.inputs())
        .expect_err("a receiver cannot substitute different actual bytes");
    assert_eq!(refused.context.code, "workspace.snapshot-binding-mismatch",);
    let mut other_root = fixture.inputs();
    other_root.workspace_root = "file:///other";
    assert!(
        OmenaWorkspaceSnapshotPublisherV0::default()
            .import(&binding, other_root)
            .is_err()
    );
    Ok(())
}

#[test]
fn input_families_advance_revision_and_reject_the_previous_binding() -> Result<(), String> {
    let mutations: [fn(&mut Fixture); 7] = [
        |fixture| fixture.styles[0].style_source.push_str(".next {}"),
        |fixture| {
            fixture.sources.push(OmenaQuerySourceDocumentInputV0 {
                source_path: "file:///workspace/App.tsx".to_string(),
                source_source: "export const value = 1".to_string(),
                source_syntax_index: None,
                has_unresolved_style_import: false,
            });
        },
        |fixture| {
            fixture.manifests.push(OmenaQueryStylePackageManifestV0 {
                package_json_path: "file:///workspace/package.json".to_string(),
                package_json_source: "{\"name\":\"example\"}".to_string(),
            });
        },
        |fixture| {
            fixture.resolution.external_sif_cache_fingerprint = Some("changed".to_string());
        },
        |fixture| fixture.settings.deep_analysis = true,
        |fixture| fixture.settings.config_content_digest = Some("changed".to_string()),
        |fixture| fixture.complete = true,
    ];
    for mutate in mutations {
        let mut fixture = Fixture::normal();
        let mut publisher = OmenaWorkspaceSnapshotPublisherV0::default();
        let previous = publisher
            .publish(fixture.inputs(), revision(7))
            .map_err(|error| error.to_string())?;
        let unchanged = publisher
            .publish(fixture.inputs(), revision(7))
            .map_err(|error| error.to_string())?;
        assert_eq!(previous, unchanged);
        publisher
            .begin_mutation(&previous)
            .map_err(|error| error.to_string())?;
        mutate(&mut fixture);
        let current = publisher
            .publish(fixture.inputs(), revision(7))
            .map_err(|error| error.to_string())?;
        assert_eq!(current.snapshot_id(), revision(8));
        assert_ne!(current.input_commitment(), previous.input_commitment());
        assert!(
            publisher
                .reader()
                .with_current_binding(&previous, || panic!("stale read must not run"))
                .is_err()
        );
        assert!(
            publisher
                .reader()
                .read_view(&previous, fixture.inputs())
                .is_err()
        );
        assert!(
            publisher
                .reader()
                .read_view(&current, fixture.inputs())
                .is_ok()
        );
    }
    Ok(())
}

#[test]
fn numeric_revision_and_current_input_commitment_cannot_forge_owner_binding() -> Result<(), String>
{
    let fixture = Fixture::normal();
    let mut publisher = OmenaWorkspaceSnapshotPublisherV0::default();
    let binding = publisher
        .publish(fixture.inputs(), revision(7))
        .map_err(|error| error.to_string())?;
    let forged = OmenaWorkspaceSnapshotBindingV0::new(
        binding.workspace_root(),
        revision(8),
        binding.input_commitment().clone(),
    );
    assert!(
        publisher
            .reader()
            .read_view(&forged, fixture.inputs())
            .is_err()
    );
    assert!(publisher.import(&forged, fixture.inputs()).is_err());
    assert!(
        publisher
            .reader()
            .read_view(&binding, fixture.inputs())
            .is_ok()
    );
    Ok(())
}

#[test]
fn exhausted_revision_does_not_reuse_an_id_for_changed_inputs() -> Result<(), String> {
    let fixture = Fixture::normal();
    let mut publisher = OmenaWorkspaceSnapshotPublisherV0::default();
    let binding = publisher
        .publish(fixture.inputs(), revision(u64::MAX))
        .map_err(|error| error.to_string())?;
    let error = publisher
        .begin_mutation(&binding)
        .expect_err("revision exhaustion must refuse mutation before inputs change");
    assert_eq!(error.context.code, "workspace.snapshot-revision-exhausted");
    assert_eq!(
        publisher
            .reader()
            .current_binding()
            .map_err(|error| error.to_string())?,
        Some(binding)
    );
    Ok(())
}

#[test]
fn malformed_commitment_is_rejected_during_wire_decode() {
    let malformed = serde_json::json!({
        "workspaceRoot": "file:///workspace",
        "snapshotId": { "value": 1 },
        "inputCommitment": "not-a-digest"
    });
    assert!(serde_json::from_value::<OmenaWorkspaceSnapshotBindingV0>(malformed).is_err());
}

#[test]
fn invalidation_cannot_be_revived_by_an_import_and_only_owner_generation_can_publish()
-> Result<(), String> {
    let mut fixture = Fixture::normal();
    let mut owner = OmenaWorkspaceSnapshotPublisherV0::default();
    let first = owner
        .publish(fixture.inputs(), revision(1))
        .map_err(|e| e.to_string())?;
    let reader = owner.reader();
    owner.begin_mutation(&first).map_err(|e| e.to_string())?;
    fixture.styles[0].style_source.push_str(".updated {}");
    assert!(reader.with_current_binding(&first, || ()).is_err());
    assert!(owner.import(&first, Fixture::normal().inputs()).is_err());
    assert!(owner.begin_mutation(&first).is_err());
    let second = owner
        .publish(fixture.inputs(), revision(1))
        .map_err(|e| e.to_string())?;
    assert_eq!(second.snapshot_id(), revision(2));
    assert!(reader.with_current_binding(&first, || ()).is_err());
    assert!(reader.with_current_binding(&second, || ()).is_ok());
    assert!(owner.begin_mutation(&first).is_err());
    assert_eq!(
        owner
            .publish(fixture.inputs(), revision(9))
            .map_err(|e| e.to_string())?,
        second
    );
    Ok(())
}

#[test]
fn current_reader_blocks_actual_owner_change_and_remains_stale_after_barrier() -> Result<(), String>
{
    use std::sync::{Arc, Barrier, mpsc};
    let mut fixture = Fixture::normal();
    let mut owner = OmenaWorkspaceSnapshotPublisherV0::default();
    let first = owner
        .publish(fixture.inputs(), revision(1))
        .map_err(|e| e.to_string())?;
    let reader = owner.reader();
    let worker_binding = first.clone();
    let barrier = Arc::new(Barrier::new(2));
    let worker_barrier = barrier.clone();
    let (starting, started) = mpsc::channel();
    let (mutated, mutation) = mpsc::channel();
    let worker = std::thread::spawn(move || {
        worker_barrier.wait();
        starting.send(()).map_err(|e| e.to_string())?;
        owner
            .begin_mutation(&worker_binding)
            .map_err(|e| e.to_string())?;
        fixture.styles[0].style_source.push_str(".updated {}");
        mutated.send(()).map_err(|e| e.to_string())?;
        owner
            .publish(fixture.inputs(), revision(1))
            .map_err(|e| e.to_string())
    });
    reader
        .with_current_binding(&first, || {
            barrier.wait();
            started.recv().map_err(|e| e.to_string())?;
            assert!(matches!(
                mutation.recv_timeout(std::time::Duration::from_millis(30)),
                Err(mpsc::RecvTimeoutError::Timeout)
            ));
            Ok::<_, String>(())
        })
        .map_err(|e| e.to_string())??;
    mutation.recv().map_err(|e| e.to_string())?;
    let second = worker.join().map_err(|_| "owner panicked".to_string())??;
    assert!(reader.with_current_binding(&first, || ()).is_err());
    assert!(reader.with_current_binding(&second, || ()).is_ok());
    Ok(())
}

#[test]
fn sdk_request_clones_cannot_publish_and_stale_clones_fail_before_mutation() -> Result<(), String> {
    let fixture = Fixture::normal();
    let mut issuer = OmenaWorkspaceSnapshotPublisherV0::default();
    let binding = issuer
        .publish(fixture.inputs(), revision(4))
        .map_err(|e| e.to_string())?;
    let transfer = omena_query::OmenaWorkspaceSnapshotTransferV0 {
        sources: Vec::new(),
        package_manifests: fixture.manifests.clone(),
        resolution_inputs: fixture.resolution.clone(),
        settings: fixture.settings.clone(),
        source_corpus_complete: fixture.complete,
    };
    let mut workspace = omena_query::OmenaSdkWorkspaceV0::open_imported_snapshot(
        omena_query::OmenaSdkSnapshotRequestV0 {
            workspace_root: "file:///workspace".to_string(),
        },
        fixture.styles.clone(),
        transfer,
        binding.clone(),
        &omena_query::OmenaQueryUtilityClassIntelligenceReportV0::default(),
    )
    .map_err(|e| e.to_string())?;
    let mut stale = workspace.clone();
    let changed = vec![OmenaQueryStyleSourceInputV0 {
        style_path: fixture.styles[0].style_path.clone(),
        style_source: ".updated {}".to_string(),
    }];
    workspace
        .replace_style_sources(changed.clone())
        .map_err(|e| e.to_string())?;
    let current = workspace.snapshot_binding().cloned();
    let error = stale
        .replace_style_sources(changed)
        .expect_err("stale clone must not acquire publication authority");
    assert_eq!(error.context.code, "workspace.snapshot-mismatch");
    let mut reader_clone = workspace.clone();
    let error = reader_clone
        .replace_style_sources(fixture.styles.clone())
        .expect_err("current request clone has no mutation authority");
    assert_eq!(error.context.code, "workspace.snapshot-owner-required");
    assert_eq!(workspace.snapshot_binding(), current.as_ref());
    workspace
        .replace_style_sources(fixture.styles)
        .map_err(|e| e.to_string())?;
    assert_eq!(workspace.snapshot_id(), revision(6));
    Ok(())
}

#[test]
fn contextual_sif_mapping_is_a_complete_input_even_when_target_facts_are_equal()
-> Result<(), Box<dyn std::error::Error>> {
    use omena_query::OmenaQueryExternalSifImportOriginV0 as Origin;
    let fixture = Fixture::default();
    let styles = ["a", "b"].map(|part| OmenaQueryStyleSourceInputV0 {
        style_path: format!("file:///workspace/{part}/App.scss"),
        style_source: "@use './tokens';".to_string(),
    });
    let mut facts = ["first", "second"].map(|part| omena_query::OmenaQueryExternalSifInputV0 {
        admitted_resolution_edges: Vec::new(),
        canonical_url: format!("file:///outside/{part}.scss"),
        sif: omena_sif::generate_static_omena_sif_v1(omena_sif::OmenaSifStaticGeneratorInputV1 {
            canonical_url: &format!("file:///outside/{part}.scss"),
            source: "$brand: red;",
            syntax: omena_sif::OmenaSifSourceSyntaxV1::Scss,
        })
        .unwrap(),
    });
    let edges = styles
        .iter()
        .zip(&facts)
        .map(|(style, fact)| {
            admitted_edge(
                Origin::Document {
                    style_path: style.style_path.clone(),
                },
                "./tokens",
                fact,
            )
        })
        .collect::<Vec<_>>();
    let swapped = styles
        .iter()
        .zip(facts.iter().rev())
        .map(|(style, fact)| {
            admitted_edge(
                Origin::Document {
                    style_path: style.style_path.clone(),
                },
                "./tokens",
                fact,
            )
        })
        .collect::<Vec<_>>();
    let mut swapped_facts = facts.clone();
    for fact in &mut facts {
        fact.admitted_resolution_edges = edges
            .iter()
            .filter(|edge| edge.sif_canonical_url == fact.sif.canonical_url)
            .cloned()
            .collect();
    }
    for fact in &mut swapped_facts {
        fact.admitted_resolution_edges = swapped
            .iter()
            .filter(|edge| edge.sif_canonical_url == fact.sif.canonical_url)
            .cloned()
            .collect();
    }
    assert_eq!(
        serde_json::to_vec(&facts)?,
        serde_json::to_vec(&swapped_facts)?,
        "legacy full SIF payload bytes are unchanged"
    );
    let selected = omena_query::select_omena_workspace_snapshot_external_sifs_v0(
        &styles,
        &fixture.resolution,
        &facts,
        &BTreeMap::new(),
        &edges,
    )?;
    let swapped_selected = omena_query::select_omena_workspace_snapshot_external_sifs_v0(
        &styles,
        &fixture.resolution,
        &swapped_facts,
        &BTreeMap::new(),
        &swapped,
    )?;
    let mut inputs = fixture.inputs();
    inputs.style_sources = &styles;
    inputs.external_sifs = &facts;
    inputs.external_sif_resolution_edges = &edges;
    inputs.external_sif_trust_records = &selected.1;
    let before = inputs.input_commitment()?;
    let mut wrong_verdict_map = selected.1.clone();
    for verdict in wrong_verdict_map.values_mut() {
        verdict.trust_tier = omena_sif::OmenaSifTrustTierV1::T3;
    }
    inputs.external_sif_trust_records = &wrong_verdict_map;
    assert_eq!(
        inputs
            .input_commitment()
            .expect_err("a supplied trust map cannot replace the actual edge verdict")
            .context
            .code,
        "workspace.snapshot-sif-admission"
    );
    inputs.external_sif_trust_records = &selected.1;
    assert_eq!(
        before,
        inputs.input_commitment()?,
        "unchanged mapping is stable"
    );
    inputs.external_sif_resolution_edges = &swapped;
    assert_eq!(
        inputs
            .input_commitment()
            .expect_err("attached mapping must match the declared commitment inputs")
            .context
            .code,
        "workspace.snapshot-sif-admission"
    );
    inputs.external_sifs = &swapped_facts;
    inputs.external_sif_trust_records = &swapped_selected.1;
    assert_ne!(
        before,
        inputs.input_commitment()?,
        "same full SIF set must bind effective per-importer resolution"
    );
    Ok(())
}

#[test]
fn contextual_sif_mixed_legacy_alias_cannot_replace_missing_or_mismatched_mapping()
-> Result<(), Box<dyn std::error::Error>> {
    let styles = vec![OmenaQueryStyleSourceInputV0 {
        style_path: "file:///context-safety/App.scss".to_string(),
        style_source: "@use 'pkg:tokens' as tokens; .card { color: tokens.$brand; }".to_string(),
    }];
    let mut target = omena_query::OmenaQueryExternalSifInputV0 {
        admitted_resolution_edges: Vec::new(),
        canonical_url: "file:///context-safety/target.scss".to_string(),
        sif: omena_sif::generate_static_omena_sif_v1(omena_sif::OmenaSifStaticGeneratorInputV1 {
            canonical_url: "file:///context-safety/target.scss",
            source: "$brand: blue;",
            syntax: omena_sif::OmenaSifSourceSyntaxV1::Scss,
        })?,
    };
    let edge = admitted_edge(
        omena_query::OmenaQueryExternalSifImportOriginV0::Document {
            style_path: styles[0].style_path.clone(),
        },
        "pkg:tokens",
        &target,
    );
    let mut legacy = target.clone();
    legacy.canonical_url = "pkg:tokens".to_string();
    target.admitted_resolution_edges = vec![edge];
    let query = |inputs: &[omena_query::OmenaQueryExternalSifInputV0]| {
        omena_query::summarize_omena_query_style_diagnostics_for_workspace_file_with_external_mode_and_sifs_and_resolution_inputs(
            &styles[0].style_path, &styles, &[], &[], None,
            omena_query::OmenaQueryExternalModuleModeV0::Sif, inputs, &Default::default(),
        ).unwrap()
    };
    let normal = serde_json::to_value(query(&[target.clone(), legacy.clone()]))?;
    assert!(
        !normal["diagnostics"]
            .as_array()
            .unwrap()
            .iter()
            .any(|d| matches!(
                d["code"].as_str(),
                Some("missingSassSymbol" | "missingExternalSif" | "unresolvedExternalReference")
            ))
    );
    let mut missing = target.clone();
    missing.admitted_resolution_edges[0].specifier = "pkg:other".to_string();
    let mut mismatched = target;
    mismatched.admitted_resolution_edges[0].sif_artifact_hash = "blake3:wrong-artifact".to_string();
    for input in [missing, mismatched] {
        let result = serde_json::to_value(query(&[input, legacy.clone()]))?;
        assert!(
            result["diagnostics"]
                .as_array()
                .unwrap()
                .iter()
                .any(|d| matches!(
                    d["code"].as_str(),
                    Some("missingExternalSif" | "unresolvedExternalReference")
                )),
            "a contextual lookup failure must not borrow the matching legacy global alias"
        );
    }
    let decoded: omena_query::OmenaQueryExternalSifInputV0 =
        serde_json::from_value(serde_json::json!({
            "canonicalUrl":legacy.canonical_url, "sif":legacy.sif,
            "admittedResolutionEdges":[{"forged":true}],
        }))?;
    assert!(
        decoded.admitted_resolution_edges.is_empty(),
        "wire claims cannot supply local admission metadata"
    );
    Ok(())
}

fn replacement_resolution() -> OmenaQueryStyleResolutionInputsV0 {
    OmenaQueryStyleResolutionInputsV0 {
        external_sif_cache_fingerprint: Some("replacement-fixture".to_string()),
        ..Default::default()
    }
}

fn imported_sdk_fixture(
    fixture: &Fixture,
    value: u64,
) -> Result<omena_query::OmenaSdkWorkspaceV0, omena_query::OmenaError> {
    let mut issuer = OmenaWorkspaceSnapshotPublisherV0::default();
    let binding = issuer.publish(fixture.inputs(), revision(value))?;
    omena_query::OmenaSdkWorkspaceV0::open_imported_snapshot(
        omena_query::OmenaSdkSnapshotRequestV0 {
            workspace_root: "file:///workspace".to_string(),
        },
        fixture.styles.clone(),
        omena_query::OmenaWorkspaceSnapshotTransferV0 {
            sources: Vec::new(),
            package_manifests: fixture.manifests.clone(),
            resolution_inputs: fixture.resolution.clone(),
            settings: fixture.settings.clone(),
            source_corpus_complete: fixture.complete,
        },
        binding,
        &omena_query::OmenaQueryUtilityClassIntelligenceReportV0::default(),
    )
}

#[test]
fn sdk_unbound_replacements_advance_once_and_return_complete_noop_payloads()
-> Result<(), Box<dyn std::error::Error>> {
    let fixture = Fixture::normal();
    let mut workspace = omena_query::OmenaSdkWorkspaceV0::open_at_snapshot(
        omena_query::OmenaSdkSnapshotRequestV0 {
            workspace_root: "file:///workspace".to_string(),
        },
        fixture.styles.clone(),
        revision(7),
    )?;
    let before = serde_json::to_value(workspace.snapshot())?;
    assert_eq!(
        serde_json::to_value(workspace.replace_style_sources(fixture.styles.clone())?)?,
        before
    );
    assert_eq!(
        serde_json::to_value(workspace.replace_style_resolution_inputs(Default::default())?)?,
        before
    );
    let changed_styles = vec![OmenaQueryStyleSourceInputV0 {
        style_source: ".changed {}".to_string(),
        ..fixture.styles[0].clone()
    }];
    let mut expected = before;
    expected["snapshotId"]["value"] = serde_json::json!(8);
    assert_eq!(
        serde_json::to_value(workspace.replace_style_sources(changed_styles.clone())?)?,
        expected
    );
    assert_eq!(
        serde_json::to_value(workspace.replace_style_sources(changed_styles)?)?,
        expected
    );
    expected["snapshotId"]["value"] = serde_json::json!(9);
    assert_eq!(
        serde_json::to_value(workspace.replace_style_resolution_inputs(replacement_resolution())?)?,
        expected
    );
    assert_eq!(
        serde_json::to_value(workspace.replace_style_resolution_inputs(replacement_resolution())?)?,
        expected
    );
    assert_eq!(serde_json::to_value(workspace.snapshot())?, expected);
    Ok(())
}

#[test]
fn sdk_exhaustion_refuses_both_changed_replacements_before_mutation_but_allows_noops()
-> Result<(), Box<dyn std::error::Error>> {
    let fixture = Fixture::normal();
    let mut workspace = omena_query::OmenaSdkWorkspaceV0::open_at_snapshot(
        omena_query::OmenaSdkSnapshotRequestV0 {
            workspace_root: "file:///workspace".to_string(),
        },
        fixture.styles.clone(),
        revision(u64::MAX),
    )?;
    let before = serde_json::to_value(workspace.snapshot())?;
    let changed_styles = vec![OmenaQueryStyleSourceInputV0 {
        style_source: ".changed {}".to_string(),
        ..fixture.styles[0].clone()
    }];
    for error in [
        workspace
            .replace_style_sources(changed_styles)
            .expect_err("changed styles must not saturate"),
        workspace
            .replace_style_resolution_inputs(replacement_resolution())
            .expect_err("changed resolver must not saturate"),
    ] {
        assert_eq!(error.context.code, "workspace.snapshot-revision-exhausted");
        assert_eq!(error.class, omena_query::OmenaErrorClassV0::Workspace);
        assert_eq!(
            error.context.recoverability,
            omena_query::OmenaErrorRecoverabilityV0::Retry
        );
    }
    assert_eq!(serde_json::to_value(workspace.snapshot())?, before);
    // At MAX, either original input would now fail if a refused mutation had stored its replacement.
    assert_eq!(
        serde_json::to_value(workspace.replace_style_sources(fixture.styles)?)?,
        before
    );
    assert_eq!(
        serde_json::to_value(workspace.replace_style_resolution_inputs(Default::default())?)?,
        before
    );
    Ok(())
}

#[test]
fn sdk_bound_replacement_refusals_preserve_owner_view_and_stale_reader_semantics()
-> Result<(), Box<dyn std::error::Error>> {
    let fixture = Fixture::normal();
    for value in [4, u64::MAX] {
        let mut owner = imported_sdk_fixture(&fixture, value)?;
        let before = serde_json::to_value(owner.snapshot())?;
        let binding = owner.snapshot_binding().cloned();
        let changed_styles = vec![OmenaQueryStyleSourceInputV0 {
            style_source: ".changed {}".to_string(),
            ..fixture.styles[0].clone()
        }];
        assert_eq!(
            serde_json::to_value(owner.replace_style_sources(fixture.styles.clone())?)?,
            before
        );
        assert_eq!(
            serde_json::to_value(owner.replace_style_resolution_inputs(Default::default())?)?,
            before
        );
        assert_eq!(
            owner
                .replace_style_resolution_inputs(replacement_resolution())
                .unwrap_err()
                .context
                .code,
            "workspace.snapshot-full-replacement-required"
        );
        let mut reader = owner.clone();
        assert_eq!(
            reader
                .replace_style_sources(changed_styles.clone())
                .unwrap_err()
                .context
                .code,
            if value == u64::MAX {
                "workspace.snapshot-revision-exhausted"
            } else {
                "workspace.snapshot-owner-required"
            }
        );
        assert_eq!(owner.snapshot_binding(), binding.as_ref());
        assert_eq!(serde_json::to_value(owner.snapshot())?, before);
        if value == u64::MAX {
            assert_eq!(
                owner
                    .replace_style_sources(changed_styles)
                    .unwrap_err()
                    .context
                    .code,
                "workspace.snapshot-revision-exhausted"
            );
            assert_eq!(
                serde_json::to_value(owner.replace_style_sources(fixture.styles.clone())?)?,
                before
            );
            assert_eq!(owner.snapshot_binding(), binding.as_ref());
        } else {
            let mut expected = before.clone();
            expected["snapshotId"]["value"] = serde_json::json!(5);
            assert_eq!(
                serde_json::to_value(owner.replace_style_sources(changed_styles.clone())?)?,
                expected
            );
            assert_eq!(
                serde_json::to_value(owner.replace_style_sources(changed_styles)?)?,
                expected
            );
            assert_ne!(owner.snapshot_binding(), binding.as_ref());
            assert_eq!(
                reader
                    .replace_style_sources(fixture.styles.clone())
                    .unwrap_err()
                    .context
                    .code,
                "workspace.snapshot-mismatch",
                "even a style no-op checks the live owner binding"
            );
            assert_eq!(
                reader
                    .replace_style_resolution_inputs(replacement_resolution())
                    .unwrap_err()
                    .context
                    .code,
                "workspace.snapshot-full-replacement-required"
            );
            // Resolver no-ops retain their existing behavior: they return the clone's snapshot.
            assert_eq!(
                serde_json::to_value(reader.replace_style_resolution_inputs(Default::default())?)?,
                before
            );
            assert_eq!(serde_json::to_value(owner.snapshot())?, expected);
        }
    }
    Ok(())
}

#[test]
fn admitted_sif_legacy_json_roundtrip_loses_equality_and_contextual_selection()
-> Result<(), Box<dyn std::error::Error>> {
    let styles = vec![OmenaQueryStyleSourceInputV0 {
        style_path: "file:///serde-context/App.scss".to_string(),
        style_source: "@use 'pkg:tokens' as tokens; .card { color: tokens.$brand; }".to_string(),
    }];
    let mut admitted = omena_query::OmenaQueryExternalSifInputV0 {
        admitted_resolution_edges: Vec::new(),
        canonical_url: "file:///serde-context/tokens.scss".to_string(),
        sif: omena_sif::generate_static_omena_sif_v1(omena_sif::OmenaSifStaticGeneratorInputV1 {
            canonical_url: "file:///serde-context/tokens.scss",
            source: "$brand: blue;",
            syntax: omena_sif::OmenaSifSourceSyntaxV1::Scss,
        })?,
    };
    let mut legacy = admitted.clone();
    legacy.canonical_url = "pkg:tokens".to_string();
    admitted.admitted_resolution_edges.push(admitted_edge(
        omena_query::OmenaQueryExternalSifImportOriginV0::Document {
            style_path: styles[0].style_path.clone(),
        },
        "pkg:other",
        &admitted,
    ));
    let wire = serde_json::to_value(&admitted)?;
    assert!(wire.get("admittedResolutionEdges").is_none());
    let decoded: omena_query::OmenaQueryExternalSifInputV0 = serde_json::from_value(wire.clone())?;
    assert!(decoded.admitted_resolution_edges.is_empty());
    assert_ne!(
        admitted, decoded,
        "admission provenance participates in Eq despite serde(skip)"
    );
    assert_eq!(wire, serde_json::to_value(&decoded)?);
    let legacy_roundtrip: omena_query::OmenaQueryExternalSifInputV0 =
        serde_json::from_value(serde_json::to_value(&legacy)?)?;
    assert_eq!(
        legacy, legacy_roundtrip,
        "empty legacy metadata is a lossless legacy roundtrip"
    );
    let trusted = omena_query::OmenaQueryBridgeExternalSifTrustedResolutionV1 {
        resolution_edges: admitted.admitted_resolution_edges.clone(),
        ..Default::default()
    };
    let cleared = omena_query::OmenaQueryBridgeExternalSifTrustedResolutionV1::default();
    assert_ne!(trusted, cleared);
    assert_eq!(
        serde_json::to_value(&trusted)?,
        serde_json::to_value(&cleared)?,
        "the Serialize-only trusted result also omits local resolution edges"
    );
    let query = |fact: omena_query::OmenaQueryExternalSifInputV0| {
        omena_query::summarize_omena_query_style_diagnostics_for_workspace_file_with_external_mode_and_sifs_and_resolution_inputs(
            &styles[0].style_path, &styles, &[], &[], None,
            omena_query::OmenaQueryExternalModuleModeV0::Sif, &[fact, legacy.clone()], &Default::default(),
        )
    };
    let before = serde_json::to_value(
        query(admitted).expect("the admitted style must produce a diagnostic report"),
    )?;
    let after = serde_json::to_value(
        query(decoded).expect("the decoded style must produce a diagnostic report"),
    )?;
    let missing = |value: &serde_json::Value| {
        value["diagnostics"].as_array().unwrap().iter().any(|d| {
            matches!(
                d["code"].as_str(),
                Some("missingExternalSif" | "unresolvedExternalReference")
            )
        })
    };
    assert!(
        missing(&before),
        "admitted context cannot borrow the legacy matching alias"
    );
    assert!(
        !missing(&after),
        "losing all admission metadata restores legacy alias selection"
    );
    assert_ne!(before, after);
    Ok(())
}

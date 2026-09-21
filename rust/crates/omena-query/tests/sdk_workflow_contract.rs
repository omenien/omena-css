use omena_query::{
    IncrementalRevisionV0, OmenaError, OmenaErrorClassV0, OmenaQueryStyleResolutionInputsV0,
    OmenaQueryStyleSourceInputV0, OmenaQueryTsconfigPathMappingV0, OmenaSdkBuildRequestV0,
    OmenaSdkBuildVerificationProfileV0, OmenaSdkBuildVerificationReasonV0,
    OmenaSdkDiagnosticsRequestV0, OmenaSdkExplainPositionV0, OmenaSdkExplainRequestV0,
    OmenaSdkQueryRequestV0, OmenaSdkResponsePartitionV0, OmenaSdkSnapshotRequestV0,
    OmenaSdkSnapshotResponseV0, OmenaSdkWorkspaceV0, OmenaWorkspaceSnapshotIdV0,
    execute_omena_sdk_diagnostics_debug_workflow, execute_omena_sdk_diagnostics_workflow,
    omena_error_from_boundary_encoding,
};

#[test]
fn sdk_workflow_contract_round_trips_existing_snapshot_identity() -> Result<(), serde_json::Error> {
    let snapshot_id =
        OmenaWorkspaceSnapshotIdV0::from_revision(IncrementalRevisionV0 { value: 41 });
    let response = OmenaSdkSnapshotResponseV0 {
        snapshot_id,
        partition: OmenaSdkResponsePartitionV0::Public,
        workspace_root: "/workspace".to_string(),
    };

    let encoded = serde_json::to_string(&response)?;
    let decoded: OmenaSdkSnapshotResponseV0 = serde_json::from_str(&encoded)?;

    assert_eq!(decoded, response);
    assert_eq!(decoded.snapshot_id.revision().value, 41);
    Ok(())
}

#[test]
fn unified_error_round_trips_with_typed_context() -> Result<(), serde_json::Error> {
    let error = omena_error_from_boundary_encoding(
        "unsupported-mode",
        "external mode is not available",
        "build",
    );

    let encoded = serde_json::to_string(&error)?;
    let decoded: OmenaError = serde_json::from_str(&encoded)?;

    assert_eq!(decoded, error);
    assert_eq!(decoded.class, OmenaErrorClassV0::Unsupported);
    assert_eq!(decoded.context.code, "boundary.build.unsupported-mode");
    Ok(())
}

#[test]
fn diagnostics_workflow_preserves_the_read_snapshot_identity() -> Result<(), OmenaError> {
    let snapshot_id =
        OmenaWorkspaceSnapshotIdV0::from_revision(IncrementalRevisionV0 { value: 23 });
    let response = execute_omena_sdk_diagnostics_workflow(
        OmenaSdkDiagnosticsRequestV0 {
            snapshot_id,
            style_path: "src/card.module.scss".to_string(),
            style_source: ".card { --tone: red; color: var(--tone); }".to_string(),
        },
        snapshot_id,
    )?;

    assert_eq!(response.snapshot_id, snapshot_id);
    assert_eq!(response.partition, OmenaSdkResponsePartitionV0::Public);
    assert_eq!(response.summary.style_path, "src/card.module.scss");
    assert_eq!(response.summary.dialect, "scss");
    assert_eq!(response.summary.class_selector_count, 1);
    assert_eq!(response.summary.custom_property_count, 1);
    Ok(())
}

#[test]
fn diagnostics_workflow_rejects_a_stale_snapshot() {
    let requested = OmenaWorkspaceSnapshotIdV0::from_revision(IncrementalRevisionV0 { value: 8 });
    let current = OmenaWorkspaceSnapshotIdV0::from_revision(IncrementalRevisionV0 { value: 9 });

    let result = execute_omena_sdk_diagnostics_workflow(
        OmenaSdkDiagnosticsRequestV0 {
            snapshot_id: requested,
            style_path: "src/card.module.css".to_string(),
            style_source: ".card { color: red; }".to_string(),
        },
        current,
    );
    assert!(
        result.is_err(),
        "stale snapshot must not produce a diagnostics response"
    );
    let Some(error) = result.err() else {
        return;
    };

    assert_eq!(error.class, OmenaErrorClassV0::Workspace);
    assert_eq!(error.context.code, "workspace.snapshot-mismatch");
}

#[test]
fn diagnostics_debug_report_is_opt_in_and_keeps_the_public_response() -> Result<(), OmenaError> {
    let snapshot_id =
        OmenaWorkspaceSnapshotIdV0::from_revision(IncrementalRevisionV0 { value: 31 });
    let report = execute_omena_sdk_diagnostics_debug_workflow(
        OmenaSdkDiagnosticsRequestV0 {
            snapshot_id,
            style_path: "src/debug.module.css".to_string(),
            style_source: ".debug { color: green; }".to_string(),
        },
        snapshot_id,
    )?;

    assert_eq!(report.partition, OmenaSdkResponsePartitionV0::Debug);
    assert_eq!(
        report.public_response.partition,
        OmenaSdkResponsePartitionV0::Public
    );
    assert_eq!(report.public_response.snapshot_id, report.snapshot_id);
    assert!(report.analysis.get("readySurfaces").is_some());
    Ok(())
}

#[test]
fn diagnostics_request_carries_snapshot_identity() -> Result<(), serde_json::Error> {
    let request = OmenaSdkDiagnosticsRequestV0 {
        snapshot_id: OmenaWorkspaceSnapshotIdV0::from_revision(IncrementalRevisionV0 { value: 7 }),
        style_path: "src/card.module.scss".to_string(),
        style_source: ".card { color: red; }".to_string(),
    };

    let encoded = serde_json::to_value(&request)?;
    assert_eq!(encoded["snapshotId"]["value"], 7);
    assert_eq!(encoded["stylePath"], "src/card.module.scss");
    Ok(())
}

fn workspace() -> Result<OmenaSdkWorkspaceV0, OmenaError> {
    OmenaSdkWorkspaceV0::open(
        OmenaSdkSnapshotRequestV0 {
            workspace_root: "/workspace".to_string(),
        },
        [OmenaQueryStyleSourceInputV0 {
            style_path: "src/card.module.scss".to_string(),
            style_source: ".card { --tone: red; color: var(--tone); }".to_string(),
        }],
    )
}

#[test]
fn workspace_runtime_executes_every_typed_workflow() -> Result<(), OmenaError> {
    let workspace = workspace()?;
    let snapshot = workspace.snapshot();
    let query = workspace.execute_query(OmenaSdkQueryRequestV0 {
        snapshot_id: snapshot.snapshot_id,
        query_kind: "styleSummary".to_string(),
        input: Some(serde_json::json!({ "stylePath": "src/card.module.scss" })),
    })?;
    let diagnostics = workspace.execute_diagnostics(OmenaSdkDiagnosticsRequestV0 {
        snapshot_id: snapshot.snapshot_id,
        style_path: "src/card.module.scss".to_string(),
        style_source: ".card { --tone: red; color: var(--tone); }".to_string(),
    })?;
    let build = workspace.execute_build(OmenaSdkBuildRequestV0 {
        snapshot_id: snapshot.snapshot_id,
        style_path: "src/card.module.scss".to_string(),
        style_source: ".card { --tone: red; color: var(--tone); }".to_string(),
        pass_ids: vec!["whitespace-normalize".to_string()],
        verification_profile: None,
        context: None,
    })?;
    let strict_build = workspace.execute_build(OmenaSdkBuildRequestV0 {
        snapshot_id: snapshot.snapshot_id,
        style_path: "src/card.module.scss".to_string(),
        style_source: ".card { --tone: red; color: var(--tone); }".to_string(),
        pass_ids: vec!["rule-merging".to_string()],
        verification_profile: Some(OmenaSdkBuildVerificationProfileV0::Strict),
        context: None,
    })?;
    let explain = workspace.execute_explain(OmenaSdkExplainRequestV0 {
        snapshot_id: snapshot.snapshot_id,
        style_path: "src/card.module.scss".to_string(),
        position: OmenaSdkExplainPositionV0 {
            line: 0,
            character: 9,
        },
    })?;

    assert_eq!(query.snapshot_id, snapshot.snapshot_id);
    assert_eq!(diagnostics.snapshot_id, snapshot.snapshot_id);
    assert_eq!(build.snapshot_id, snapshot.snapshot_id);
    assert_eq!(strict_build.snapshot_id, snapshot.snapshot_id);
    assert_eq!(explain.snapshot_id, snapshot.snapshot_id);
    assert_eq!(query.payload["language"], "scss");
    assert!(build.summary["sourceMapV3"]["sources"].is_array());
    assert_eq!(
        build.summary["requestedPassIds"],
        serde_json::json!(["whitespace-normalize"])
    );
    assert_eq!(
        build.summary["effectivePassIds"],
        build.summary["requestedPassIds"]
    );
    assert_eq!(
        strict_build.summary["execution"]["strictPolicy"]["refusedCount"],
        1
    );
    assert_eq!(
        strict_build.summary["execution"]["strictPolicy"]["rolledBackCount"],
        0
    );
    assert_eq!(strict_build.verification.refused_count, 1);
    assert_eq!(strict_build.verification.rolled_back_count, 0);
    assert_eq!(
        strict_build.verification.refusal_reasons[0].reasons,
        vec![OmenaSdkBuildVerificationReasonV0::CascadeEnvironmentUnavailable]
    );
    assert_eq!(
        explain.report["sourceIdentity"]["originalSource"],
        "src/card.module.scss"
    );
    Ok(())
}

#[test]
fn workspace_runtime_advances_only_for_changed_sources() -> Result<(), OmenaError> {
    let mut workspace = workspace()?;
    let initial = workspace.snapshot();
    let unchanged = workspace.replace_style_sources([OmenaQueryStyleSourceInputV0 {
        style_path: "src/card.module.scss".to_string(),
        style_source: ".card { --tone: red; color: var(--tone); }".to_string(),
    }])?;
    assert_eq!(unchanged.snapshot_id, initial.snapshot_id);

    let changed = workspace.replace_style_sources([OmenaQueryStyleSourceInputV0 {
        style_path: "src/card.module.scss".to_string(),
        style_source: ".card { color: blue; }".to_string(),
    }])?;
    assert_ne!(changed.snapshot_id, initial.snapshot_id);
    let stale = workspace.execute_query(OmenaSdkQueryRequestV0 {
        snapshot_id: initial.snapshot_id,
        query_kind: "styleSummary".to_string(),
        input: Some(serde_json::json!({ "stylePath": "src/card.module.scss" })),
    });
    let error = stale
        .err()
        .ok_or_else(|| OmenaError::unknown("stale query succeeded", "test.unexpected-success"))?;
    assert_eq!(error.class, OmenaErrorClassV0::Workspace);
    Ok(())
}

#[test]
fn workspace_runtime_normalizes_empty_style_paths() -> Result<(), OmenaError> {
    let workspace = OmenaSdkWorkspaceV0::open(
        OmenaSdkSnapshotRequestV0 {
            workspace_root: "/workspace".to_string(),
        },
        [OmenaQueryStyleSourceInputV0 {
            style_path: String::new(),
            style_source: ".root { color: red; }".to_string(),
        }],
    )?;
    let response = workspace.execute_diagnostics(OmenaSdkDiagnosticsRequestV0 {
        snapshot_id: workspace.snapshot_id(),
        style_path: String::new(),
        style_source: ".root { color: red; }".to_string(),
    })?;
    assert_eq!(response.summary.style_path, "style.css");
    Ok(())
}

#[test]
fn workspace_alias_resolution_surface_uses_snapshot_inputs() -> Result<(), OmenaError> {
    let workspace = OmenaSdkWorkspaceV0::open_with_resolution_inputs(
        OmenaSdkSnapshotRequestV0 {
            workspace_root: "/workspace".to_string(),
        },
        [OmenaQueryStyleSourceInputV0 {
            style_path: "/workspace/src/styles/Card.module.css".to_string(),
            style_source: ".card {}".to_string(),
        }],
        OmenaQueryStyleResolutionInputsV0 {
            tsconfig_path_mappings: vec![OmenaQueryTsconfigPathMappingV0 {
                base_path: "/workspace".to_string(),
                pattern: "@styles/*".to_string(),
                target_patterns: vec!["src/styles/*".to_string()],
            }],
            ..OmenaQueryStyleResolutionInputsV0::default()
        },
    )?;

    let summary = workspace.execute_source_diagnostics(
        workspace.snapshot_id(),
        "/workspace/src/App.tsx",
        r#"import styles from "@styles/Card.module.css";
export const app = <div className={styles.card} />;"#,
        &[],
    )?;

    assert!(
        summary
            .diagnostics
            .iter()
            .all(|diagnostic| diagnostic.code != "missing-module"),
        "{summary:?}"
    );
    Ok(())
}

#[test]
fn bridge_admission_keeps_reused_import_contexts_and_package_forward_backing()
-> Result<(), Box<dyn std::error::Error>> {
    let root =
        std::env::temp_dir().join(format!("omena-contextual-package-{}", std::process::id()));
    struct Cleanup(std::path::PathBuf);
    impl Drop for Cleanup {
        fn drop(&mut self) {
            let _ = std::fs::remove_dir_all(&self.0);
        }
    }
    std::fs::create_dir(&root)?;
    let _cleanup = Cleanup(root.clone());
    let package = root.join("node_modules/@design/tokens");
    std::fs::create_dir_all(&package)?;
    let manifest = package.join("package.json");
    let manifest_source = r#"{"name":"@design/tokens","version":"1.0.0","sass":"./_index.scss"}"#;
    std::fs::write(&manifest, manifest_source)?;
    std::fs::write(package.join("_index.scss"), "@forward './middle';\n")?;
    std::fs::write(package.join("_middle.scss"), "@forward './leaf';\n")?;
    std::fs::write(package.join("_leaf.scss"), "$brand: blue;\n")?;
    let root = std::fs::canonicalize(&root)?;
    let package = root.join("node_modules/@design/tokens");
    let styles = ["a", "b"].map(|part| omena_query::OmenaQueryStyleSourceInputV0 {
        style_path: format!("file://{}/{part}.scss", root.display()),
        style_source: "@use 'pkg:@design/tokens' as tokens; .card { color: tokens.$brand; }"
            .to_string(),
    });
    let resolution = omena_query::OmenaQueryStyleResolutionInputsV0 {
        package_manifests: vec![omena_query::OmenaQueryStylePackageManifestV0 {
            package_json_path: package.join("package.json").to_string_lossy().into_owned(),
            package_json_source: manifest_source.to_string(),
        }],
        ..Default::default()
    };
    let admitted =
        omena_query::resolve_omena_query_bridge_external_sifs_for_style_sources_with_trust(
            &styles,
            &[],
            &resolution,
        );
    let direct = admitted
        .resolution_edges
        .iter()
        .filter(|edge| {
            matches!(
                edge.importer,
                omena_query::OmenaQueryExternalSifImportOriginV0::Document { .. }
            )
        })
        .collect::<Vec<_>>();
    assert_eq!(
        direct.len(),
        2,
        "reused target must retain both document origins"
    );
    assert_eq!(
        admitted.resolution.generation_count, 3,
        "one invocation reuses each actual target"
    );
    assert_eq!(direct[0].sif_canonical_url, "pkg:@design/tokens");
    assert!(admitted.resolution_edges.iter().any(|edge| {
        matches!(&edge.importer, omena_query::OmenaQueryExternalSifImportOriginV0::Sif { resolved_style_url, .. } if resolved_style_url == &format!("file://{}", package.join("_index.scss").display()))
            && edge.resolved_style_url == format!("file://{}", package.join("_middle.scss").display())
    }), "package SIF forwards must resolve from the actual backing file");
    let trust = admitted
        .trust_records
        .iter()
        .map(|record| (record.canonical_url.clone(), record.clone()))
        .collect();
    let selected = omena_query::select_omena_workspace_snapshot_external_sifs_v0(
        &styles,
        &resolution,
        &admitted.resolution.external_sifs,
        &trust,
        &admitted.resolution_edges,
    )?;
    assert_eq!(
        selected.0.len(),
        3,
        "the complete forward chain stays reachable"
    );
    assert_eq!(
        selected.2.len(),
        6,
        "two document edges and each document's full two-edge transitive context survive"
    );
    for style in &styles {
        assert_eq!(admitted.resolution_edges.iter().filter(|edge| {
            matches!(&edge.importer, omena_query::OmenaQueryExternalSifImportOriginV0::Sif { initiating_document: Some(document), .. } if document == &style.style_path)
        }).count(), 2, "reused package closure must retain each initiating document");
    }
    assert!(
        admitted.resolution_edges.iter().all(|edge| {
            edge.trust.canonical_url == edge.sif_canonical_url
                && edge.trust.trust_tier == omena_sif::OmenaSifTrustTierV1::T1
                && edge.trust.trust_source
                    == omena_query::OmenaQueryExternalSifTrustSourceV1::UnsignedLegacy
        }),
        "each edge must carry its actual independently generated unsigned verdict"
    );
    assert!(
        serde_json::to_value(&admitted)?
            .get("resolutionEdges")
            .is_none(),
        "legacy result wire must not transport admission provenance"
    );
    #[cfg(unix)]
    {
        let alias = root.join("alias.scss");
        let leaf = package.join("_leaf.scss");
        std::os::unix::fs::symlink(&leaf, &alias)?;
        let alias_uri = format!("file://{}", alias.display());
        let source = omena_query::OmenaQueryStyleSourceInputV0 {
            style_path: format!("file://{}/direct.scss", root.display()),
            style_source: format!("@use '{alias_uri}' as tokens;"),
        };
        let direct_file =
            omena_query::resolve_omena_query_bridge_external_sifs_for_style_sources_with_trust(
                &[source],
                &[],
                &resolution,
            );
        assert_eq!(direct_file.resolution_edges.len(), 1);
        assert_eq!(direct_file.resolution_edges[0].specifier, alias_uri);
        assert_eq!(
            direct_file.resolution_edges[0].resolved_style_url,
            format!("file://{}", leaf.display()),
            "file URI seeds must retain the existing resolver's actual confirmed target"
        );
    }
    Ok(())
}

#[test]
#[cfg(feature = "sif-attestation")]
fn contextual_package_trust_keeps_independent_root_verdicts_reconstructable()
-> Result<(), Box<dyn std::error::Error>> {
    exercise_contextual_package_trust(false)
}

#[test]
#[cfg(all(feature = "sif-attestation", unix))]
fn contextual_package_trust_shared_backing_uses_importing_root_storage()
-> Result<(), Box<dyn std::error::Error>> {
    exercise_contextual_package_trust(true)
}

#[cfg(feature = "sif-attestation")]
fn exercise_contextual_package_trust(
    shared_backing: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    use omena_query::{
        OmenaQueryBridgeExternalSifTrustedResolutionV1, OmenaQueryExternalSifTrustSourceV1,
        OmenaWorkspaceSnapshotInputsV0, OmenaWorkspaceSnapshotPublisherV0,
        OmenaWorkspaceSnapshotSettingsV0, OmenaWorkspaceSnapshotTransferV0,
        select_omena_workspace_snapshot_external_sifs_v0,
    };
    use std::collections::BTreeMap;
    const BUNDLE_SHA256: &str = "0c99e37ac1b1d3cbfd677416a74218c9a1ca8e28c3aac95c7614549f3b3b0ce1";
    const PUBLISHED_URL: &str = "pkg:omena-fixture/external-sif-trust.css";
    let fixture = std::env::temp_dir().join(format!(
        "omena-contextual-package-trust-{}-{shared_backing}",
        std::process::id()
    ));
    struct Cleanup(std::path::PathBuf);
    impl Drop for Cleanup {
        fn drop(&mut self) {
            let _ = std::fs::remove_dir_all(&self.0);
        }
    }
    std::fs::create_dir(&fixture)?;
    let _cleanup = Cleanup(fixture.clone());
    let fixture = std::fs::canonicalize(fixture)?;
    let published_source =
        include_str!("../../../../examples/src/scenarios/08-css-only/CssOnly.module.css");
    let published_sif = omena_sif::read_omena_sif_json_v1(
        include_str!("../../omena-bridge/tests/fixtures/published-sif-attestation.sif.json")
            .trim_end(),
    )?;
    let artifact_hash = omena_sif::compute_omena_sif_artifact_hash_v1(&published_sif)?;
    let manifest_source = r#"{"name":"omena-fixture","exports":{"./external-sif-trust.css":"./external-sif-trust.css"}}"#;
    let mut contexts = Vec::new();
    for part in ["a", "b"] {
        let root = fixture.join(part);
        let package = root.join("node_modules/omena-fixture");
        if shared_backing && part == "b" {
            std::fs::create_dir_all(package.parent().ok_or("package parent")?)?;
            #[cfg(unix)]
            std::os::unix::fs::symlink(fixture.join("a/node_modules/omena-fixture"), &package)?;
            #[cfg(not(unix))]
            return Err("shared package fixture requires symlinks".into());
        } else {
            std::fs::create_dir_all(&package)?;
        }
        std::fs::write(package.join("package.json"), manifest_source)?;
        let backing = package.join("external-sif-trust.css");
        std::fs::write(&backing, published_source)?;
        let style = OmenaQueryStyleSourceInputV0 {
            style_path: format!("file://{}", root.join("App.scss").display()),
            style_source: format!("@use '{PUBLISHED_URL}' as published; .card {{ color: red; }}"),
        };
        std::fs::write(root.join("App.scss"), &style.style_source)?;
        let resolution = OmenaQueryStyleResolutionInputsV0 {
            package_manifests: vec![omena_query::OmenaQueryStylePackageManifestV0 {
                package_json_path: package.join("package.json").to_string_lossy().into_owned(),
                package_json_source: manifest_source.to_string(),
            }],
            ..Default::default()
        };
        let storage = omena_query::OmenaQueryExternalSifStorageV0::from_workspace_cache_root(
            root.join(".cache/omena"),
        );
        if part == "a" {
            let verdict_dir = storage.recorded_verdict_dir().ok_or("verdict dir")?;
            let reference = format!("bundles-v1/{BUNDLE_SHA256}.sigstore.json");
            let bundle_path = verdict_dir.join(&reference);
            std::fs::create_dir_all(bundle_path.parent().ok_or("bundle parent")?)?;
            std::fs::write(
                &bundle_path,
                include_bytes!(
                    "../../omena-bridge/tests/fixtures/published-sif-attestation.sigstore.json"
                ),
            )?;
            let verdict = omena_sif::OmenaSifShardRecordedVerdictV1 {
                schema_version: omena_sif::OMENA_SIF_SHARD_RECORDED_VERDICT_SCHEMA_VERSION_V1
                    .to_string(),
                product: omena_sif::OMENA_SIF_SHARD_RECORDED_VERDICT_PRODUCT_V1.to_string(),
                verification_owner: omena_sif::OMENA_SIF_SHARD_VERIFICATION_OWNER_V1.to_string(),
                canonical_url: PUBLISHED_URL.to_string(),
                sif_hash: artifact_hash.clone(),
                trust_tier: omena_sif::OmenaSifTrustTierV1::T3,
                signature: omena_sif::OmenaSifShardSignatureV1 {
                    algorithm_version: omena_sif::OMENA_SIF_SHARD_SIGNATURE_ALGORITHM_VERSION_V1
                        .to_string(),
                    reference,
                    signed_payload_digest: artifact_hash.clone(),
                },
            };
            let address = omena_sif::compute_omena_sif_shard_recorded_verdict_address_v1(
                PUBLISHED_URL,
                &artifact_hash,
            )?;
            let name = address
                .as_str()
                .strip_prefix("blake3:")
                .ok_or("verdict address")?;
            std::fs::write(
                verdict_dir.join(format!("{name}.json")),
                omena_sif::write_omena_sif_shard_recorded_verdict_json_v1(&verdict)?,
            )?;
        }
        let admitted = omena_query::resolve_omena_query_bridge_external_sifs_for_style_sources_with_cache_storage_and_trust(
            std::slice::from_ref(&style), &[], &resolution, &storage,
        );
        assert_eq!(
            admitted.resolution.external_sifs.len(),
            1,
            "actual package admission: {admitted:?}"
        );
        assert_eq!(admitted.resolution.external_sifs[0].sif, published_sif);
        assert_eq!(admitted.trust_records.len(), 1);
        assert_eq!(admitted.trust_records[0].canonical_url, PUBLISHED_URL);
        let (tier, source) = if part == "a" {
            (
                omena_sif::OmenaSifTrustTierV1::T3,
                OmenaQueryExternalSifTrustSourceV1::RecordedVerdict,
            )
        } else {
            (
                omena_sif::OmenaSifTrustTierV1::T1,
                OmenaQueryExternalSifTrustSourceV1::UnsignedLegacy,
            )
        };
        assert_eq!(
            admitted.trust_records[0].trust_tier, tier,
            "actual bridge-produced verdict"
        );
        assert_eq!(admitted.trust_records[0].trust_source, source);
        assert_eq!(admitted.resolution_edges.len(), 1);
        let edge = &admitted.resolution_edges[0];
        assert_eq!(
            edge.importer,
            omena_query::OmenaQueryExternalSifImportOriginV0::Document {
                style_path: style.style_path.clone()
            }
        );
        assert_eq!(
            edge.resolved_style_url,
            format!("file://{}", std::fs::canonicalize(&backing)?.display())
        );
        assert_eq!(edge.sif_canonical_url, PUBLISHED_URL);
        assert_eq!(edge.sif_artifact_hash, artifact_hash.as_str());
        assert_eq!(
            edge.trust, admitted.trust_records[0],
            "context must retain the actual bridge-produced verdict"
        );
        contexts.push((
            format!("file://{}", root.display()),
            style,
            resolution,
            admitted,
        ));
    }
    assert_eq!(
        contexts[0].3.resolution_edges[0].resolved_style_url
            == contexts[1].3.resolution_edges[0].resolved_style_url,
        shared_backing,
        "the fixture must distinguish shared physical backing from independent copies"
    );
    assert_eq!(
        serde_json::to_vec(&contexts[0].3.resolution.external_sifs)?,
        serde_json::to_vec(&contexts[1].3.resolution.external_sifs)?
    );
    let mut combined = OmenaQueryBridgeExternalSifTrustedResolutionV1::default();
    for (_, _, _, admitted) in &contexts {
        combined
            .resolution
            .external_sifs
            .extend(admitted.resolution.external_sifs.clone());
        combined
            .resolution_edges
            .extend(admitted.resolution_edges.clone());
        combined
            .trust_records
            .extend(admitted.trust_records.clone());
    }
    let inspect = |root: &str,
                   style: &OmenaQueryStyleSourceInputV0,
                   resolution: &OmenaQueryStyleResolutionInputsV0,
                   admitted: &OmenaQueryBridgeExternalSifTrustedResolutionV1,
                   snapshot_revision: u64|
     -> Result<serde_json::Value, Box<dyn std::error::Error>> {
        // This is the actual canonical-key merge used by LSP and SDK admission.
        let trust = admitted
            .trust_records
            .iter()
            .map(|record| (record.canonical_url.clone(), record.clone()))
            .collect::<BTreeMap<_, _>>();
        let (sifs, trust, edges) = select_omena_workspace_snapshot_external_sifs_v0(
            std::slice::from_ref(style),
            resolution,
            &admitted.resolution.external_sifs,
            &trust,
            &admitted.resolution_edges,
        )?;
        let settings = OmenaWorkspaceSnapshotSettingsV0::default();
        let empty = BTreeMap::new();
        let providers = BTreeMap::new();
        let mut owner = OmenaWorkspaceSnapshotPublisherV0::default();
        let binding = owner.publish(
            OmenaWorkspaceSnapshotInputsV0 {
                workspace_root: root,
                style_sources: std::slice::from_ref(style),
                source_documents: &[],
                source_language_ids: &empty,
                source_provider_inputs: &providers,
                package_manifests: &resolution.package_manifests,
                external_sifs: &sifs,
                external_sif_resolution_edges: &edges,
                external_sif_trust_records: &trust,
                resolution_inputs: resolution,
                settings: &settings,
                source_corpus_complete: true,
            },
            OmenaWorkspaceSnapshotIdV0::from_revision(IncrementalRevisionV0 {
                value: snapshot_revision,
            }),
        )?;
        let transfer = OmenaWorkspaceSnapshotTransferV0 {
            sources: Vec::new(),
            package_manifests: resolution.package_manifests.clone(),
            resolution_inputs: resolution.clone(),
            settings,
            source_corpus_complete: true,
        };
        let received = OmenaSdkWorkspaceV0::open_imported_snapshot(
            OmenaSdkSnapshotRequestV0 {
                workspace_root: root.to_string(),
            },
            vec![style.clone()],
            transfer.clone(),
            binding.clone(),
            &omena_query::OmenaQueryUtilityClassIntelligenceReportV0::default(),
        );
        let (accepted, error, payload) = match received {
            Ok(workspace) => (
                true,
                None,
                Some(serde_json::to_string(&workspace.execute_diagnostics(
                    OmenaSdkDiagnosticsRequestV0 {
                        snapshot_id: binding.snapshot_id(),
                        style_path: style.style_path.clone(),
                        style_source: style.style_source.clone(),
                    },
                )?)?),
            ),
            Err(error) => (false, Some(error), None),
        };
        Ok(
            serde_json::json!({"root":root,"fullSifs":sifs,"selectedTrust":trust,"selectedEdges":edges,
            "binding":binding,"transfer":transfer,"receiverAccepted":accepted,"receiverError":error,"receiverFullPayloadBytes":payload}),
        )
    };
    let mut observations = Vec::new();
    for (root, style, resolution, admitted) in &contexts {
        let isolated = inspect(root, style, resolution, admitted, 7)?;
        let repeated = inspect(root, style, resolution, admitted, 7)?;
        let merged = inspect(root, style, resolution, &combined, 7)?;
        let mut reversed_legacy = combined.clone();
        reversed_legacy.trust_records.reverse();
        let reordered_legacy = inspect(root, style, resolution, &reversed_legacy, 7)?;
        observations.push(serde_json::json!({"actualIndependentAdmission":admitted,"actualResolutionEdges":admitted.resolution_edges,
            "isolated":isolated,"unchanged":repeated,"merged":merged,"reorderedLegacyMap":reordered_legacy}));
    }
    // The disabled disk-cache sequence shares the physical artifact memory key:
    // every admission must still verify the actual importing root's verdict.
    let mut disabled_cache_observations = Vec::new();
    for index in [0usize, 1, 0] {
        let (root, style, resolution, expected) = &contexts[index];
        let verdict_root = fixture.join(if index == 0 { "a" } else { "b" });
        let storage = omena_query::OmenaQueryExternalSifStorageV0::from_optional_workspace_cache_root_and_verdict_dir(
            None, root, verdict_root.join(".cache/omena").join(omena_sif::OMENA_SIF_SHARD_VERDICT_DIR_V1),
        );
        let actual = omena_query::resolve_omena_query_bridge_external_sifs_for_style_sources_with_cache_storage_and_trust(
            std::slice::from_ref(style), &[], resolution, &storage,
        );
        disabled_cache_observations.push(serde_json::json!({
            "root":root, "cacheRoot":storage.workspace_cache_root(), "verdictDir":storage.recorded_verdict_dir(),
            "actual":actual, "actualEdges":actual.resolution_edges,
            "expectedCached":expected, "expectedCachedEdges":expected.resolution_edges,
        }));
    }
    let report = serde_json::json!({"feature":"sif-attestation","sharedPhysicalBacking":shared_backing,"publishedArtifactHash":artifact_hash,
        "publishedBundleSha256":BUNDLE_SHA256,"contexts":observations,"disabledCacheSequence":disabled_cache_observations,
        "memoryCacheKillSwitch":std::env::var("OMENA_BRIDGE_EXTERNAL_SIF_CACHE").ok(),
        "lspLimit":"The normal LSP binary has no attestation verifier and must refuse recorded verdicts; this is the supported feature-enabled query/CLI admission boundary."});
    if let Some(directory) = std::env::var_os("OMENA_WORKSPACE_SNAPSHOT_TEST_RECEIPTS") {
        let file = std::fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(std::path::PathBuf::from(directory).join(if shared_backing {
                "contextual-package-trust-shared.json"
            } else {
                "contextual-package-trust.json"
            }))?;
        serde_json::to_writer_pretty(file, &report)?;
    }
    for disabled in &disabled_cache_observations {
        assert!(disabled["cacheRoot"].is_null());
        assert_eq!(
            disabled["actual"], disabled["expectedCached"],
            "disabled cache retains the actual importing-root bridge verdict: {disabled}"
        );
        assert_eq!(
            disabled["actualEdges"], disabled["expectedCachedEdges"],
            "memory reuse cannot replace the actual context verdict: {disabled}"
        );
    }
    for context in &observations {
        assert_eq!(
            context["isolated"]["receiverAccepted"], true,
            "actual isolated admission must reconstruct: {context}"
        );
        assert_eq!(
            context["isolated"], context["unchanged"],
            "same context and artifact remain unchanged"
        );
        assert_eq!(
            context["isolated"], context["merged"],
            "unrelated admitted root facts must not change the selected actual verdict or binding"
        );
        assert_eq!(
            context["merged"], context["reorderedLegacyMap"],
            "legacy canonical-map merge order cannot replace contextual actual verdicts"
        );
    }
    assert_eq!(
        observations[1]["merged"]["receiverAccepted"], true,
        "the independently unsigned root B control must reconstruct"
    );
    assert_eq!(
        observations[0]["merged"]["receiverAccepted"], true,
        "unrelated root B trust must not replace root A actual artifact/context verdict during projection; full observations: {report}"
    );
    if shared_backing {
        let mut mutations = Vec::new();
        for (index, (root, style, resolution, admitted)) in contexts.iter().enumerate() {
            let initial = &observations[index]["isolated"];
            let binding: omena_query::OmenaWorkspaceSnapshotBindingV0 =
                serde_json::from_value(initial["binding"].clone())?;
            let transfer: OmenaWorkspaceSnapshotTransferV0 =
                serde_json::from_value(initial["transfer"].clone())?;
            let mut workspace = OmenaSdkWorkspaceV0::open_imported_snapshot(
                OmenaSdkSnapshotRequestV0 {
                    workspace_root: root.clone(),
                },
                vec![style.clone()],
                transfer,
                binding.clone(),
                &omena_query::OmenaQueryUtilityClassIntelligenceReportV0::default(),
            )?;
            let stale_clone = workspace.clone();
            let reader = workspace
                .snapshot_read_view()?
                .owner()
                .cloned()
                .ok_or("imported reader")?;
            let mut before_write_entered = false;
            let before_write_error = reader
                .with_current_write_binding(&binding, || before_write_entered = true)
                .err();
            let replacement = OmenaQueryStyleSourceInputV0 {
                style_path: style.style_path.clone(),
                style_source: format!("{} .changed {{ color: blue; }}", style.style_source),
            };
            let replaced = workspace.replace_style_sources([replacement.clone()])?;
            let actual_binding = workspace
                .snapshot_binding()
                .ok_or("mutated bound snapshot")?
                .clone();
            let expected = inspect(
                root,
                &replacement,
                resolution,
                admitted,
                replaced.snapshot_id.value,
            )?;
            let actual_payload = serde_json::to_string(&workspace.execute_diagnostics(
                OmenaSdkDiagnosticsRequestV0 {
                    snapshot_id: replaced.snapshot_id,
                    style_path: replacement.style_path.clone(),
                    style_source: replacement.style_source.clone(),
                },
            )?)?;
            let mut after_write_entered = false;
            let after_write_error = reader
                .with_current_write_binding(&actual_binding, || after_write_entered = true)
                .err();
            let stale_error = stale_clone.ensure_snapshot_binding(&binding).err();
            mutations.push(serde_json::json!({
                "root":root,"initial":initial,"replacement":replacement,"actualBinding":actual_binding,
                "expected":expected,"actualFullPayloadBytes":actual_payload,"staleCloneError":stale_error,
                "beforeWriteError":before_write_error,"afterWriteError":after_write_error,
                "beforeWriteEntered":before_write_entered,"afterWriteEntered":after_write_entered,
            }));
        }
        if let Some(directory) = std::env::var_os("OMENA_WORKSPACE_SNAPSHOT_TEST_RECEIPTS") {
            let file = std::fs::OpenOptions::new()
                .write(true)
                .create_new(true)
                .open(
                    std::path::PathBuf::from(directory)
                        .join("contextual-shared-backing-mutation.json"),
                )?;
            serde_json::to_writer_pretty(file, &mutations)?;
        }
        for mutation in &mutations {
            assert_eq!(
                mutation["actualBinding"], mutation["expected"]["binding"],
                "imported mutation must retain the importing-root verdict: {mutation}"
            );
            assert_eq!(
                mutation["actualFullPayloadBytes"],
                mutation["expected"]["receiverFullPayloadBytes"]
            );
            assert_ne!(
                mutation["actualFullPayloadBytes"],
                mutation["initial"]["receiverFullPayloadBytes"]
            );
            assert_ne!(mutation["actualBinding"], mutation["initial"]["binding"]);
            assert_eq!(
                mutation["staleCloneError"]["context"]["code"],
                "workspace.snapshot-mismatch"
            );
            assert_eq!(
                mutation["beforeWriteError"]["context"]["code"],
                "workspace.snapshot-write-owner-required"
            );
            assert_eq!(
                mutation["afterWriteError"]["context"]["code"],
                "workspace.snapshot-write-owner-required"
            );
            assert_eq!(mutation["beforeWriteEntered"], false);
            assert_eq!(mutation["afterWriteEntered"], false);
        }
    }
    if !shared_backing {
        let (root, style, resolution, before) = &contexts[0];
        let storage = omena_query::OmenaQueryExternalSifStorageV0::from_workspace_cache_root(
            fixture.join("a/.cache/omena"),
        );
        let bundle = storage
            .recorded_verdict_dir()
            .ok_or("actual verdict directory")?
            .join(format!("bundles-v1/{BUNDLE_SHA256}.sigstore.json"));
        std::fs::write(bundle, b"{}")?;
        let after = omena_query::resolve_omena_query_bridge_external_sifs_for_style_sources_with_cache_storage_and_trust(
            std::slice::from_ref(style), &[], resolution, &storage,
        );
        assert_eq!(after.resolution.external_sifs.len(), 1);
        assert_eq!(
            after.resolution.external_sifs[0].sif,
            before.resolution.external_sifs[0].sif
        );
        assert_eq!(after.resolution_edges.len(), 1);
        assert_eq!(
            after.resolution_edges[0].trust.trust_tier,
            omena_sif::OmenaSifTrustTierV1::T1
        );
        assert_eq!(
            after.resolution_edges[0].trust.trust_source,
            OmenaQueryExternalSifTrustSourceV1::UnsignedLegacy
        );
        let mut combined_sifs = before.resolution.external_sifs.clone();
        combined_sifs.extend(after.resolution.external_sifs.clone());
        let mut combined_edges = before.resolution_edges.clone();
        combined_edges.extend(after.resolution_edges.clone());
        let selected_before = select_omena_workspace_snapshot_external_sifs_v0(
            std::slice::from_ref(style),
            resolution,
            &before.resolution.external_sifs,
            &BTreeMap::new(),
            &before.resolution_edges,
        )?;
        let selector = select_omena_workspace_snapshot_external_sifs_v0(
            std::slice::from_ref(style),
            resolution,
            &combined_sifs,
            &selected_before.1,
            &combined_edges,
        );
        let settings = OmenaWorkspaceSnapshotSettingsV0::default();
        let empty = BTreeMap::new();
        let providers = BTreeMap::new();
        let direct = OmenaWorkspaceSnapshotInputsV0 {
            workspace_root: root,
            style_sources: std::slice::from_ref(style),
            source_documents: &[],
            source_language_ids: &empty,
            source_provider_inputs: &providers,
            package_manifests: &resolution.package_manifests,
            external_sifs: &combined_sifs,
            external_sif_resolution_edges: &combined_edges,
            external_sif_trust_records: &selected_before.1,
            resolution_inputs: resolution,
            settings: &settings,
            source_corpus_complete: true,
        }
        .input_commitment();
        if let Some(directory) = std::env::var_os("OMENA_WORKSPACE_SNAPSHOT_TEST_RECEIPTS") {
            let file = std::fs::OpenOptions::new()
                .write(true)
                .create_new(true)
                .open(std::path::PathBuf::from(directory).join("contextual-trust-conflict.json"))?;
            serde_json::to_writer_pretty(
                file,
                &serde_json::json!({
                    "actualPriorEdges":before.resolution_edges,"actualNextEdges":after.resolution_edges,
                    "actualPriorSifs":before.resolution.external_sifs,"actualNextSifs":after.resolution.external_sifs,
                    "suppliedLastValueTrustMap":selected_before.1,"rootSelectorError":selector.as_ref().err(),
                    "directCommitment":direct.as_ref().ok(),"directCommitmentError":direct.as_ref().err(),
                    "scope":"Two genuine sequential admissions intentionally combined through public borrowed inputs; no native LSP owner publication of this conflicting pair is established."
                }),
            )?;
        }
        assert_eq!(
            selector
                .expect_err("root selector must reject conflicting actual admissions")
                .context
                .code,
            "workspace.snapshot-sif-admission"
        );
        assert_eq!(direct.expect_err("direct borrowed inputs must not choose the last verdict for an ambiguous actual context").context.code, "workspace.snapshot-sif-admission");
    }
    Ok(())
}

#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    reason = "This module is compiled only for the WASM replacement refusal fixture"
)]

use super::*;

// This no-argument fixture exists only in an explicitly selected WASM test build.
// It adds no production constructor or state-seeding capability.
#[wasm_bindgen(js_name = __omenaPrivateReplacementRefusalFixture)]
pub fn private_replacement_refusal_fixture() -> Result<JsValue, JsValue> {
    use omena_query::{
        IncrementalRevisionV0, OmenaWorkspaceSnapshotIdV0, OmenaWorkspaceSnapshotInputsV0,
        OmenaWorkspaceSnapshotPublisherV0, OmenaWorkspaceSnapshotSettingsV0,
        OmenaWorkspaceSnapshotTransferV0,
    };
    use std::collections::BTreeMap;

    #[derive(Serialize)]
    #[serde(rename_all = "camelCase")]
    struct Row {
        scenario: &'static str,
        operation: &'static str,
        envelope: OmenaSdkErrorEnvelopeV0,
        preserved_revision: String,
    }
    fn mapped(error: JsValue, code: &str, message: &str) -> OmenaSdkErrorEnvelopeV0 {
        let envelope: OmenaSdkErrorEnvelopeV0 = serde_wasm_bindgen::from_value(error).unwrap();
        assert_eq!(
            envelope,
            OmenaSdkErrorEnvelopeV0 {
                error: OmenaError::new(
                    if code == "sdk.response-serialization" {
                        OmenaErrorClassV0::Internal
                    } else {
                        OmenaErrorClassV0::Workspace
                    },
                    message,
                    OmenaErrorContextV0 {
                        code: code.to_string(),
                        severity: OmenaErrorSeverityV0::Error,
                        recoverability: OmenaErrorRecoverabilityV0::Retry,
                        evidence: Vec::new(),
                    },
                )
            }
        );
        envelope
    }
    let mut rows = Vec::new();
    for bound in [false, true] {
        let request = OmenaSdkSnapshotRequestV0 {
            workspace_root: "file:///wasm-replacements".to_string(),
        };
        let styles = vec![OmenaQueryStyleSourceInputV0 {
            style_path: "file:///wasm-replacements/Button.module.css".to_string(),
            style_source: ".button { color: red; }".to_string(),
        }];
        let changed = vec![OmenaQueryStyleSourceInputV0 {
            style_source: ".button { color: blue; }".to_string(),
            ..styles[0].clone()
        }];
        let revision = OmenaWorkspaceSnapshotIdV0::from_revision(IncrementalRevisionV0 {
            value: if bound { 4 } else { u64::MAX },
        });
        let resolution = OmenaQueryStyleResolutionInputsV0::default();
        let changed_resolution = OmenaQueryStyleResolutionInputsV0 {
            external_sif_cache_fingerprint: Some("replacement-fixture".to_string()),
            ..Default::default()
        };
        let settings = OmenaWorkspaceSnapshotSettingsV0::default();
        let mut publisher = OmenaWorkspaceSnapshotPublisherV0::default();
        let inner = if bound {
            let binding = publisher
                .publish(
                    OmenaWorkspaceSnapshotInputsV0 {
                        workspace_root: &request.workspace_root,
                        style_sources: &styles,
                        source_documents: &[],
                        source_language_ids: &BTreeMap::new(),
                        source_provider_inputs: &BTreeMap::new(),
                        package_manifests: &[],
                        external_sifs: &[],
                        external_sif_trust_records: &BTreeMap::new(),
                        external_sif_resolution_edges: &[],
                        resolution_inputs: &resolution,
                        settings: &settings,
                        source_corpus_complete: false,
                    },
                    revision,
                )
                .map_err(browser_error)?;
            OmenaSdkWorkspaceV0::open_owned_snapshot(
                request,
                styles.clone(),
                OmenaWorkspaceSnapshotTransferV0 {
                    sources: Vec::new(),
                    package_manifests: Vec::new(),
                    resolution_inputs: resolution.clone(),
                    settings,
                    source_corpus_complete: false,
                },
                Vec::new(),
                Vec::new(),
                BTreeMap::new(),
                Vec::new(),
                binding,
                publisher.reader(),
            )
            .map_err(browser_error)?
        } else {
            OmenaSdkWorkspaceV0::open_at_snapshot(request, styles.clone(), revision)
                .map_err(browser_error)?
        };
        let before = inner.snapshot();
        let binding = inner.snapshot_binding().cloned();
        let mut workspace = OmenaWasmWorkspaceV0 { inner };
        for (operation, error, code, message) in [
            (
                "replaceStyleSources",
                workspace
                    .replace_style_sources(to_value(&changed)?)
                    .unwrap_err(),
                if bound {
                    "workspace.snapshot-owner-required"
                } else {
                    "workspace.snapshot-revision-exhausted"
                },
                if bound {
                    "a request clone must submit mutations to its existing workspace owner"
                } else {
                    "workspace snapshot revision space is exhausted"
                },
            ),
            (
                "replaceStyleResolutionInputs",
                workspace
                    .replace_style_resolution_inputs(to_value(&changed_resolution)?)
                    .unwrap_err(),
                if bound {
                    "workspace.snapshot-full-replacement-required"
                } else {
                    "workspace.snapshot-revision-exhausted"
                },
                if bound {
                    "resolver mutation requires replacement of the complete admitted source-fact view"
                } else {
                    "workspace snapshot revision space is exhausted"
                },
            ),
        ] {
            rows.push(Row {
                scenario: if bound {
                    "bound-reader-change"
                } else {
                    "max-change"
                },
                operation,
                envelope: mapped(error, code, message),
                preserved_revision: revision.revision().value.to_string(),
            });
        }
        assert_eq!(workspace.inner.snapshot(), before);
        assert_eq!(workspace.inner.snapshot_binding(), binding.as_ref());
        // These are actual unchanged inner calls, so they also detect stored-input mutation.
        assert_eq!(
            workspace
                .inner
                .replace_style_sources(styles.clone())
                .unwrap(),
            before
        );
        assert_eq!(
            workspace
                .inner
                .replace_style_resolution_inputs(resolution.clone())
                .unwrap(),
            before
        );
        if bound {
            for payload in [
                workspace.replace_style_sources(to_value(&styles)?)?,
                workspace.replace_style_resolution_inputs(to_value(&resolution)?)?,
            ] {
                let decoded: omena_query::OmenaSdkSnapshotResponseV0 =
                    serde_wasm_bindgen::from_value(payload).unwrap();
                assert_eq!(decoded, before);
            }
        } else {
            // The Rust no-op succeeds, but the unchanged JSON-compatible serializer cannot encode MAX.
            for (operation, error) in [
                (
                    "replaceStyleSources",
                    workspace
                        .replace_style_sources(to_value(&styles)?)
                        .unwrap_err(),
                ),
                (
                    "replaceStyleResolutionInputs",
                    workspace
                        .replace_style_resolution_inputs(to_value(&resolution)?)
                        .unwrap_err(),
                ),
            ] {
                rows.push(Row { scenario: "max-noop-wire-limit", operation,
                    envelope: mapped(error, "sdk.response-serialization",
                        "failed to serialize SDK response: Error: 18446744073709551615 can't be represented as a JavaScript number"),
                    preserved_revision: revision.revision().value.to_string() });
            }
        }
        assert_eq!(workspace.inner.snapshot(), before);
        assert_eq!(workspace.inner.snapshot_binding(), binding.as_ref());
    }
    to_value(&rows)
}

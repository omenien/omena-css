use std::{
    collections::BTreeMap,
    fs,
    path::Path,
    sync::{Arc, Mutex, MutexGuard, OnceLock, Weak},
};

use napi_derive::napi;
use omena_query::{
    OmenaError, OmenaErrorClassV0, OmenaErrorContextV0, OmenaErrorRecoverabilityV0,
    OmenaErrorSeverityV0, OmenaQueryStylePackageManifestV0, OmenaQueryStyleResolutionInputsV0,
    OmenaQueryStyleSourceInputV0, OmenaSdkBuildRequestV0, OmenaSdkDiagnosticsRequestV0,
    OmenaSdkErrorEnvelopeV0, OmenaSdkExplainRequestV0, OmenaSdkQueryRequestV0,
    OmenaSdkSnapshotRequestV0, OmenaSdkWorkspaceV0,
};
use serde::{Serialize, de::DeserializeOwned};

type SharedWorkspace = Arc<Mutex<OmenaSdkWorkspaceV0>>;

#[derive(Default)]
struct WorkspaceSessionCache {
    entries: BTreeMap<String, Weak<Mutex<OmenaSdkWorkspaceV0>>>,
    hit_count: u64,
    miss_count: u64,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct WorkspaceSessionCacheReport {
    schema_version: &'static str,
    product: &'static str,
    live_entry_count: usize,
    hit_count: u64,
    miss_count: u64,
}

static WORKSPACE_SESSION_CACHE: OnceLock<Mutex<WorkspaceSessionCache>> = OnceLock::new();

#[napi(js_name = "Workspace")]
pub struct OmenaNapiWorkspaceV0 {
    inner: OmenaSdkWorkspaceV0,
}

#[napi(js_name = "CachedWorkspace")]
pub struct OmenaNapiCachedWorkspaceV0 {
    inner: SharedWorkspace,
}

#[napi]
impl OmenaNapiWorkspaceV0 {
    #[napi(constructor)]
    pub fn new(workspace_root: String, style_sources_json: String) -> napi::Result<Self> {
        let style_sources = parse_json::<Vec<OmenaQueryStyleSourceInputV0>>(
            style_sources_json.as_str(),
            "workspace style sources",
        )?;
        let inner =
            OmenaSdkWorkspaceV0::open(OmenaSdkSnapshotRequestV0 { workspace_root }, style_sources)
                .map_err(native_error)?;
        Ok(Self { inner })
    }

    #[napi(js_name = "snapshotJson")]
    pub fn snapshot_json(&self) -> napi::Result<String> {
        to_json(&self.inner.snapshot())
    }

    #[napi(js_name = "replaceStyleSourcesJson")]
    pub fn replace_style_sources_json(
        &mut self,
        style_sources_json: String,
    ) -> napi::Result<String> {
        let style_sources = parse_json::<Vec<OmenaQueryStyleSourceInputV0>>(
            style_sources_json.as_str(),
            "workspace style sources",
        )?;
        let snapshot = self
            .inner
            .replace_style_sources(style_sources)
            .map_err(native_error)?;
        to_json(&snapshot)
    }

    #[napi(js_name = "replaceStyleResolutionInputsJson")]
    pub fn replace_style_resolution_inputs_json(
        &mut self,
        resolution_inputs_json: String,
    ) -> napi::Result<String> {
        let resolution_inputs = parse_json::<OmenaQueryStyleResolutionInputsV0>(
            resolution_inputs_json.as_str(),
            "style resolution inputs",
        )?;
        to_json(
            &self
                .inner
                .replace_style_resolution_inputs(resolution_inputs),
        )
    }

    #[napi(js_name = "queryJson")]
    pub fn query_json(&self, request_json: String) -> napi::Result<String> {
        let request = parse_json::<OmenaSdkQueryRequestV0>(request_json.as_str(), "query request")?;
        to_json(&self.inner.execute_query(request).map_err(native_error)?)
    }

    #[napi(js_name = "diagnosticsJson")]
    pub fn diagnostics_json(&self, request_json: String) -> napi::Result<String> {
        let request = parse_json::<OmenaSdkDiagnosticsRequestV0>(
            request_json.as_str(),
            "diagnostics request",
        )?;
        to_json(
            &self
                .inner
                .execute_diagnostics(request)
                .map_err(native_error)?,
        )
    }

    #[napi(js_name = "buildJson")]
    pub fn build_json(&self, request_json: String) -> napi::Result<String> {
        let request = parse_json::<OmenaSdkBuildRequestV0>(request_json.as_str(), "build request")?;
        to_json(&self.inner.execute_build(request).map_err(native_error)?)
    }

    #[napi(js_name = "explainJson")]
    pub fn explain_json(&self, request_json: String) -> napi::Result<String> {
        let request =
            parse_json::<OmenaSdkExplainRequestV0>(request_json.as_str(), "explain request")?;
        to_json(&self.inner.execute_explain(request).map_err(native_error)?)
    }
}

#[napi]
impl OmenaNapiCachedWorkspaceV0 {
    #[napi(constructor)]
    pub fn new(
        workspace_root: String,
        config_content_digest: String,
        style_sources_json: String,
    ) -> napi::Result<Self> {
        let style_sources = parse_json::<Vec<OmenaQueryStyleSourceInputV0>>(
            style_sources_json.as_str(),
            "workspace style sources",
        )?;
        let inner = open_cached_workspace(
            workspace_root.as_str(),
            config_content_digest.as_str(),
            style_sources,
        )?;
        Ok(Self { inner })
    }

    #[napi(js_name = "snapshotJson")]
    pub fn snapshot_json(&self) -> napi::Result<String> {
        to_json(&lock_workspace(&self.inner)?.snapshot())
    }

    #[napi(js_name = "replaceStyleSourcesJson")]
    pub fn replace_style_sources_json(&self, style_sources_json: String) -> napi::Result<String> {
        let style_sources = parse_json::<Vec<OmenaQueryStyleSourceInputV0>>(
            style_sources_json.as_str(),
            "workspace style sources",
        )?;
        let snapshot = lock_workspace(&self.inner)?
            .replace_style_sources(style_sources)
            .map_err(native_error)?;
        to_json(&snapshot)
    }

    #[napi(js_name = "replaceStyleResolutionInputsJson")]
    pub fn replace_style_resolution_inputs_json(
        &self,
        resolution_inputs_json: String,
    ) -> napi::Result<String> {
        let resolution_inputs = parse_json::<OmenaQueryStyleResolutionInputsV0>(
            resolution_inputs_json.as_str(),
            "style resolution inputs",
        )?;
        let snapshot =
            lock_workspace(&self.inner)?.replace_style_resolution_inputs(resolution_inputs);
        to_json(&snapshot)
    }

    #[napi(js_name = "sourceDiagnosticsJson")]
    pub fn source_diagnostics_json(
        &self,
        source_path: String,
        source: String,
        package_manifests_json: String,
    ) -> napi::Result<String> {
        let package_manifests = parse_json::<Vec<OmenaQueryStylePackageManifestV0>>(
            package_manifests_json.as_str(),
            "workspace package manifests",
        )?;
        let workspace = lock_workspace(&self.inner)?;
        let diagnostics = workspace
            .execute_source_diagnostics(
                workspace.snapshot_id(),
                source_path.as_str(),
                source.as_str(),
                package_manifests.as_slice(),
            )
            .map_err(native_error)?;
        to_json(&diagnostics)
    }

    #[napi(js_name = "resolveCssModuleJson")]
    pub fn resolve_css_module_json(
        &self,
        style_path: String,
        package_manifests_json: String,
    ) -> napi::Result<String> {
        let package_manifests = parse_json::<Vec<OmenaQueryStylePackageManifestV0>>(
            package_manifests_json.as_str(),
            "workspace package manifests",
        )?;
        let workspace = lock_workspace(&self.inner)?;
        let response = workspace
            .execute_bundler_resolve(workspace.snapshot_id(), style_path, package_manifests)
            .map_err(native_error)?;
        to_json(&response)
    }
}

#[napi(js_name = "workspaceSessionCacheReportJson")]
pub fn workspace_session_cache_report_json() -> napi::Result<String> {
    let mut cache = lock_cache()?;
    cache.entries.retain(|_, entry| entry.strong_count() > 0);
    to_json(&WorkspaceSessionCacheReport {
        schema_version: "0",
        product: "omena-napi.workspace-session-cache",
        live_entry_count: cache.entries.len(),
        hit_count: cache.hit_count,
        miss_count: cache.miss_count,
    })
}

fn open_cached_workspace(
    workspace_root: &str,
    config_content_digest: &str,
    style_sources: Vec<OmenaQueryStyleSourceInputV0>,
) -> napi::Result<SharedWorkspace> {
    let workspace_root = canonical_workspace_root(workspace_root);
    let key = format!("{workspace_root}\0{config_content_digest}");
    let candidate = OmenaSdkWorkspaceV0::open(
        OmenaSdkSnapshotRequestV0 {
            workspace_root: workspace_root.clone(),
        },
        style_sources.clone(),
    )
    .map_err(native_error)?;

    let (inner, reused) = {
        let mut cache = lock_cache()?;
        cache.entries.retain(|_, entry| entry.strong_count() > 0);
        if let Some(inner) = cache.entries.get(&key).and_then(Weak::upgrade) {
            cache.hit_count = cache.hit_count.saturating_add(1);
            (inner, true)
        } else {
            let inner = Arc::new(Mutex::new(candidate));
            cache.entries.insert(key, Arc::downgrade(&inner));
            cache.miss_count = cache.miss_count.saturating_add(1);
            (inner, false)
        }
    };

    if reused {
        lock_workspace(&inner)?
            .replace_style_sources(style_sources)
            .map_err(native_error)?;
    }
    Ok(inner)
}

fn canonical_workspace_root(workspace_root: &str) -> String {
    fs::canonicalize(Path::new(workspace_root))
        .unwrap_or_else(|_| Path::new(workspace_root).to_path_buf())
        .to_string_lossy()
        .into_owned()
}

fn lock_cache() -> napi::Result<MutexGuard<'static, WorkspaceSessionCache>> {
    WORKSPACE_SESSION_CACHE
        .get_or_init(|| Mutex::new(WorkspaceSessionCache::default()))
        .lock()
        .map_err(|_| workspace_session_lock_error("workspace session cache lock was poisoned"))
}

fn lock_workspace(inner: &SharedWorkspace) -> napi::Result<MutexGuard<'_, OmenaSdkWorkspaceV0>> {
    inner
        .lock()
        .map_err(|_| workspace_session_lock_error("workspace session lock was poisoned"))
}

fn workspace_session_lock_error(message: &'static str) -> napi::Error {
    native_error(OmenaError::new(
        OmenaErrorClassV0::Internal,
        message,
        OmenaErrorContextV0 {
            code: "sdk.workspace-session-lock".to_string(),
            severity: OmenaErrorSeverityV0::Error,
            recoverability: OmenaErrorRecoverabilityV0::Retry,
            evidence: Vec::new(),
        },
    ))
}

fn parse_json<T: DeserializeOwned>(source: &str, label: &str) -> napi::Result<T> {
    serde_json::from_str(source).map_err(|error| {
        native_error(OmenaError::new(
            OmenaErrorClassV0::Input,
            format!("failed to parse {label}: {error}"),
            OmenaErrorContextV0 {
                code: "sdk.request-parse".to_string(),
                severity: OmenaErrorSeverityV0::Error,
                recoverability: OmenaErrorRecoverabilityV0::UserAction,
                evidence: Vec::new(),
            },
        ))
    })
}

fn to_json<T: Serialize>(value: &T) -> napi::Result<String> {
    serde_json::to_string(value).map_err(|error| {
        native_error(OmenaError::new(
            OmenaErrorClassV0::Internal,
            format!("failed to serialize SDK response: {error}"),
            OmenaErrorContextV0 {
                code: "sdk.response-serialization".to_string(),
                severity: OmenaErrorSeverityV0::Error,
                recoverability: OmenaErrorRecoverabilityV0::Retry,
                evidence: Vec::new(),
            },
        ))
    })
}

fn native_error(error: OmenaError) -> napi::Error {
    let envelope = OmenaSdkErrorEnvelopeV0 { error };
    let reason = serde_json::to_string(&envelope).unwrap_or_else(|_| {
        "{\"error\":{\"class\":\"internal\",\"message\":\"failed to serialize SDK error\",\"context\":{\"code\":\"sdk.error-serialization\",\"severity\":\"error\",\"recoverability\":\"retry\"}}}".to_string()
    });
    napi::Error::from_reason(reason)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn style_sources(source: &str) -> Vec<OmenaQueryStyleSourceInputV0> {
        vec![OmenaQueryStyleSourceInputV0 {
            style_path: "Button.module.css".to_string(),
            style_source: source.to_string(),
        }]
    }

    #[test]
    fn cached_workspace_reuses_matching_workspace_and_config_identity() -> napi::Result<()> {
        let root = std::env::temp_dir().join("omena-napi-session-cache-reuse");
        fs::create_dir_all(&root).map_err(|error| napi::Error::from_reason(error.to_string()))?;
        let first = open_cached_workspace(
            root.to_string_lossy().as_ref(),
            "config-a",
            style_sources(".button { color: red; }"),
        )?;
        let second = open_cached_workspace(
            root.to_string_lossy().as_ref(),
            "config-a",
            style_sources(".button { color: red; }"),
        )?;
        assert!(Arc::ptr_eq(&first, &second));
        assert_eq!(lock_workspace(&second)?.snapshot_id().revision().value, 1);

        let third = open_cached_workspace(
            root.to_string_lossy().as_ref(),
            "config-a",
            style_sources(".button { color: blue; }"),
        )?;
        assert!(Arc::ptr_eq(&first, &third));
        assert_eq!(lock_workspace(&third)?.snapshot_id().revision().value, 2);
        Ok(())
    }

    #[test]
    fn cached_workspace_partitions_config_snapshots_and_reuses_bundler_protocol() -> napi::Result<()>
    {
        let root = std::env::temp_dir().join("omena-napi-session-cache-partition");
        fs::create_dir_all(&root).map_err(|error| napi::Error::from_reason(error.to_string()))?;
        let first = open_cached_workspace(
            root.to_string_lossy().as_ref(),
            "config-a",
            style_sources(".button { color: red; }"),
        )?;
        let second = open_cached_workspace(
            root.to_string_lossy().as_ref(),
            "config-b",
            style_sources(".button { color: red; }"),
        )?;
        assert!(!Arc::ptr_eq(&first, &second));

        let workspace = lock_workspace(&first)?;
        let response = workspace
            .execute_bundler_resolve(
                workspace.snapshot_id(),
                "Button.module.css".to_string(),
                Vec::new(),
            )
            .map_err(native_error)?;
        assert!(response.ready);
        assert_eq!(response.protocol_version, "0");
        assert!(response.class_map.contains_key("button"));
        Ok(())
    }
}

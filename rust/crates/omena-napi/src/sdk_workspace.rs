use napi_derive::napi;
use omena_query::{
    OmenaError, OmenaErrorClassV0, OmenaErrorContextV0, OmenaErrorRecoverabilityV0,
    OmenaErrorSeverityV0, OmenaQueryStyleSourceInputV0, OmenaSdkBuildRequestV0,
    OmenaSdkDiagnosticsRequestV0, OmenaSdkErrorEnvelopeV0, OmenaSdkExplainRequestV0,
    OmenaSdkQueryRequestV0, OmenaSdkSnapshotRequestV0, OmenaSdkWorkspaceV0,
};
use serde::{Serialize, de::DeserializeOwned};

#[napi(js_name = "Workspace")]
pub struct OmenaNapiWorkspaceV0 {
    inner: OmenaSdkWorkspaceV0,
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

fn parse_json<T: DeserializeOwned>(source: &str, label: &str) -> napi::Result<T> {
    serde_json::from_str(source).map_err(|error| {
        native_error(OmenaError::new(
            OmenaErrorClassV0::Input,
            format!("failed to parse {label}: {error}"),
            OmenaErrorContextV0 {
                code: "sdk.request-parse".to_string(),
                severity: OmenaErrorSeverityV0::Error,
                recoverability: OmenaErrorRecoverabilityV0::UserAction,
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

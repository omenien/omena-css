use omena_query::{
    OmenaError, OmenaErrorClassV0, OmenaErrorContextV0, OmenaErrorRecoverabilityV0,
    OmenaErrorSeverityV0, OmenaQueryStyleResolutionInputsV0, OmenaQueryStyleSourceInputV0,
    OmenaSdkBuildRequestV0, OmenaSdkDiagnosticsRequestV0, OmenaSdkErrorEnvelopeV0,
    OmenaSdkExplainRequestV0, OmenaSdkQueryRequestV0, OmenaSdkSnapshotRequestV0,
    OmenaSdkWorkspaceV0,
};
use serde::Serialize;
use wasm_bindgen::prelude::*;

#[wasm_bindgen(js_name = Workspace)]
pub struct OmenaWasmWorkspaceV0 {
    inner: OmenaSdkWorkspaceV0,
}

#[wasm_bindgen(js_class = Workspace)]
impl OmenaWasmWorkspaceV0 {
    #[wasm_bindgen(constructor)]
    pub fn new(workspace_root: String, style_sources: JsValue) -> Result<Self, JsValue> {
        let style_sources = parse_value::<Vec<OmenaQueryStyleSourceInputV0>>(
            style_sources,
            "workspace style sources",
        )?;
        let inner =
            OmenaSdkWorkspaceV0::open(OmenaSdkSnapshotRequestV0 { workspace_root }, style_sources)
                .map_err(browser_error)?;
        Ok(Self { inner })
    }

    #[wasm_bindgen(js_name = snapshot)]
    pub fn snapshot(&self) -> Result<JsValue, JsValue> {
        to_value(&self.inner.snapshot())
    }

    #[wasm_bindgen(js_name = replaceStyleSources)]
    pub fn replace_style_sources(&mut self, style_sources: JsValue) -> Result<JsValue, JsValue> {
        let style_sources = parse_value::<Vec<OmenaQueryStyleSourceInputV0>>(
            style_sources,
            "workspace style sources",
        )?;
        let snapshot = self
            .inner
            .replace_style_sources(style_sources)
            .map_err(browser_error)?;
        to_value(&snapshot)
    }

    /// Replaces caller-discovered mappings and disk identities without browser-side filesystem use.
    #[wasm_bindgen(js_name = replaceStyleResolutionInputs)]
    pub fn replace_style_resolution_inputs(
        &mut self,
        resolution_inputs: JsValue,
    ) -> Result<JsValue, JsValue> {
        let resolution_inputs = parse_value::<OmenaQueryStyleResolutionInputsV0>(
            resolution_inputs,
            "style resolution inputs",
        )?;
        to_value(
            &self
                .inner
                .replace_style_resolution_inputs(resolution_inputs),
        )
    }

    #[wasm_bindgen(js_name = query)]
    pub fn query(&self, request: JsValue) -> Result<JsValue, JsValue> {
        let request = parse_value::<OmenaSdkQueryRequestV0>(request, "query request")?;
        to_value(&self.inner.execute_query(request).map_err(browser_error)?)
    }

    #[wasm_bindgen(js_name = diagnostics)]
    pub fn diagnostics(&self, request: JsValue) -> Result<JsValue, JsValue> {
        let request = parse_value::<OmenaSdkDiagnosticsRequestV0>(request, "diagnostics request")?;
        to_value(
            &self
                .inner
                .execute_diagnostics(request)
                .map_err(browser_error)?,
        )
    }

    #[wasm_bindgen(js_name = build)]
    pub fn build(&self, request: JsValue) -> Result<JsValue, JsValue> {
        let request = parse_value::<OmenaSdkBuildRequestV0>(request, "build request")?;
        to_value(&self.inner.execute_build(request).map_err(browser_error)?)
    }

    #[wasm_bindgen(js_name = explain)]
    pub fn explain(&self, request: JsValue) -> Result<JsValue, JsValue> {
        let request = parse_value::<OmenaSdkExplainRequestV0>(request, "explain request")?;
        to_value(&self.inner.execute_explain(request).map_err(browser_error)?)
    }
}

fn parse_value<T: serde::de::DeserializeOwned>(value: JsValue, label: &str) -> Result<T, JsValue> {
    serde_wasm_bindgen::from_value(value).map_err(|error| {
        browser_error(OmenaError::new(
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

fn to_value<T: Serialize>(value: &T) -> Result<JsValue, JsValue> {
    value
        .serialize(&serde_wasm_bindgen::Serializer::json_compatible())
        .map_err(|error| {
            browser_error(OmenaError::new(
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

fn browser_error(error: OmenaError) -> JsValue {
    OmenaSdkErrorEnvelopeV0 { error }
        .serialize(&serde_wasm_bindgen::Serializer::json_compatible())
        .unwrap_or_else(|_| JsValue::from_str("SDK error serialization failed"))
}

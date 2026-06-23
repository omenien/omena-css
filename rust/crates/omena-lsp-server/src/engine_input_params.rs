use omena_query::OmenaQueryEngineInputV2;
use serde_json::{Value, json};

pub(crate) fn query_engine_input_from_params(
    params: Option<&Value>,
) -> Option<OmenaQueryEngineInputV2> {
    if let Some(engine_input) = params.and_then(|value| value.get("engineInput")) {
        return serde_json::from_value(engine_input.clone()).ok();
    }

    serde_json::from_value(json!({
        "version": "2",
        "workspace": {
            "root": "/",
            "classnameTransform": "asIs",
            "settingsKey": "lsp-default",
        },
        "sources": [],
        "styles": [],
        "typeFacts": [],
    }))
    .ok()
}

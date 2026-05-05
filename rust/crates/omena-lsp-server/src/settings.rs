use crate::LspShellState;
use serde_json::Value;

pub(crate) fn apply_feature_settings(state: &mut LspShellState, features: Option<&Value>) {
    let Some(features) = features.and_then(Value::as_object) else {
        return;
    };
    if let Some(value) = features.get("definition").and_then(Value::as_bool) {
        state.features.definition = value;
    }
    if let Some(value) = features.get("hover").and_then(Value::as_bool) {
        state.features.hover = value;
    }
    if let Some(value) = features.get("completion").and_then(Value::as_bool) {
        state.features.completion = value;
    }
    if let Some(value) = features.get("references").and_then(Value::as_bool) {
        state.features.references = value;
    }
    if let Some(value) = features.get("rename").and_then(Value::as_bool) {
        state.features.rename = value;
    }
}

pub(crate) fn apply_diagnostic_settings(state: &mut LspShellState, diagnostics: Option<&Value>) {
    let Some(diagnostics) = diagnostics.and_then(Value::as_object) else {
        return;
    };
    if let Some(value) = diagnostics
        .get("severity")
        .and_then(Value::as_str)
        .and_then(diagnostic_severity_code)
    {
        state.diagnostics.severity = value;
    }
}

fn diagnostic_severity_code(value: &str) -> Option<u8> {
    match value {
        "error" => Some(1),
        "warning" => Some(2),
        "information" => Some(3),
        "hint" => Some(4),
        _ => None,
    }
}

use crate::{LspShellState, file_uri_to_path, normalize_path};
use omena_query::OmenaQueryStylePackageManifestV0;
use serde_json::Value;
use std::{fs, path::PathBuf};

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

pub(crate) fn apply_diagnostic_settings(
    state: &mut LspShellState,
    diagnostics: Option<&Value>,
) -> bool {
    let Some(diagnostics) = diagnostics.and_then(Value::as_object) else {
        return false;
    };
    let mut changed = false;
    if let Some(value) = diagnostics
        .get("severity")
        .and_then(Value::as_str)
        .and_then(diagnostic_severity_code)
    {
        changed |= state.diagnostics.severity != value;
        state.diagnostics.severity = value;
    }
    if let Some(value) = diagnostics.get("deepAnalysis").and_then(Value::as_bool) {
        changed |= state.diagnostics.deep_analysis != value;
        state.diagnostics.deep_analysis = value;
    }
    changed
}

pub(crate) fn apply_resolution_settings(
    state: &mut LspShellState,
    resolution: Option<&Value>,
) -> bool {
    let Some(resolution) = resolution.and_then(Value::as_object) else {
        return false;
    };
    let Some(package_manifest_paths) = resolution
        .get("packageManifestPaths")
        .and_then(Value::as_array)
    else {
        return false;
    };

    let package_manifests = package_manifest_paths
        .iter()
        .filter_map(Value::as_str)
        .filter_map(read_package_manifest_setting)
        .collect::<Vec<_>>();
    let normalized_paths = package_manifests
        .iter()
        .map(|manifest| manifest.package_json_path.clone())
        .collect::<Vec<_>>();

    let changed = state.resolution.package_manifest_paths != normalized_paths
        || state.resolution.package_manifests != package_manifests;
    if changed {
        state.resolution.package_manifest_paths = normalized_paths;
        state.resolution.package_manifests = package_manifests;
    }
    changed
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

fn read_package_manifest_setting(value: &str) -> Option<OmenaQueryStylePackageManifestV0> {
    let path = setting_path(value)?;
    let package_json_path = normalize_path(path).to_string_lossy().to_string();
    let package_json_source = fs::read_to_string(package_json_path.as_str()).ok()?;
    Some(OmenaQueryStylePackageManifestV0 {
        package_json_path,
        package_json_source,
    })
}

fn setting_path(value: &str) -> Option<PathBuf> {
    file_uri_to_path(value).or_else(|| Some(PathBuf::from(value)))
}

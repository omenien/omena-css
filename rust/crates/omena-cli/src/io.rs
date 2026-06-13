use std::{
    fs,
    path::{Path, PathBuf},
};

use omena_query::{
    OmenaQueryEngineInputV2, OmenaQuerySourceDocumentInputV0,
    OmenaQuerySourceMissingSelectorDiagnosticCandidateV0, OmenaQueryStylePackageManifestV0,
    OmenaQueryStyleSourceInputV0, OmenaQueryTransformExecutionContextV0,
};

use crate::paths::path_string;

pub(crate) fn read_source_documents(
    source_document_paths: &[PathBuf],
) -> Result<Vec<OmenaQuerySourceDocumentInputV0>, String> {
    source_document_paths
        .iter()
        .map(|path| {
            Ok(OmenaQuerySourceDocumentInputV0 {
                source_path: path_string(path),
                source_source: read_source(path)?,
                source_syntax_index: None,
                has_unresolved_style_import: false,
            })
        })
        .collect()
}

pub(crate) fn read_style_sources(
    source_paths: &[PathBuf],
) -> Result<Vec<OmenaQueryStyleSourceInputV0>, String> {
    source_paths
        .iter()
        .map(|path| {
            Ok(OmenaQueryStyleSourceInputV0 {
                style_path: path_string(path),
                style_source: read_source(path)?,
            })
        })
        .collect()
}

pub(crate) fn read_package_manifests(
    package_manifest_paths: &[PathBuf],
) -> Result<Vec<OmenaQueryStylePackageManifestV0>, String> {
    package_manifest_paths
        .iter()
        .map(|path| {
            Ok(OmenaQueryStylePackageManifestV0 {
                package_json_path: path_string(path),
                package_json_source: read_source(path)?,
            })
        })
        .collect()
}

pub(crate) fn read_source(path: &Path) -> Result<String, String> {
    fs::read_to_string(path)
        .map_err(|error| format!("failed to read {}: {error}", path_string(path)))
}

pub(crate) fn read_context_json(
    path: Option<&Path>,
) -> Result<OmenaQueryTransformExecutionContextV0, String> {
    let Some(path) = path else {
        return Ok(OmenaQueryTransformExecutionContextV0::default());
    };
    let json = fs::read_to_string(path)
        .map_err(|error| format!("failed to read context JSON {}: {error}", path_string(path)))?;
    serde_json::from_str(&json).map_err(|error| {
        format!(
            "failed to parse context JSON {}: {error}",
            path_string(path)
        )
    })
}

pub(crate) fn read_engine_input_json(path: &Path) -> Result<OmenaQueryEngineInputV2, String> {
    let json = fs::read_to_string(path).map_err(|error| {
        format!(
            "failed to read engine input JSON {}: {error}",
            path_string(path)
        )
    })?;
    serde_json::from_str(&json).map_err(|error| {
        format!(
            "failed to parse engine input JSON {}: {error}",
            path_string(path)
        )
    })
}

pub(crate) fn read_source_diagnostic_candidates_json(
    path: &Path,
) -> Result<Vec<OmenaQuerySourceMissingSelectorDiagnosticCandidateV0>, String> {
    let json = fs::read_to_string(path).map_err(|error| {
        format!(
            "failed to read source diagnostics candidates JSON {}: {error}",
            path_string(path)
        )
    })?;
    serde_json::from_str(&json).map_err(|error| {
        format!(
            "failed to parse source diagnostics candidates JSON {}: {error}",
            path_string(path)
        )
    })
}

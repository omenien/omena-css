use super::*;

pub(super) fn workspace_source_map(
    style_sources: &[omena_query::OmenaQueryStyleSourceInputV0],
    source_documents: &[omena_query::OmenaQuerySourceDocumentInputV0],
) -> BTreeMap<String, String> {
    style_sources
        .iter()
        .map(|source| (source.style_path.clone(), source.style_source.clone()))
        .chain(
            source_documents
                .iter()
                .map(|source| (source.source_path.clone(), source.source_source.clone())),
        )
        .collect()
}

pub(super) fn source_for_uri<'a>(
    sources: &'a BTreeMap<String, String>,
    uri: &str,
) -> Option<&'a str> {
    sources
        .get(uri)
        .or_else(|| {
            cli_file_uri_to_path(uri)
                .as_ref()
                .and_then(|path| sources.get(path_string(path.as_path()).as_str()))
        })
        .map(String::as_str)
}

pub(super) fn resolve_workspace_root(root: Option<&Path>) -> Result<PathBuf, String> {
    let root = match root {
        Some(root) => root.to_path_buf(),
        None => std::env::current_dir()
            .map_err(|error| format!("failed to resolve the current directory: {error}"))?,
    };
    let canonical = fs::canonicalize(root.as_path()).map_err(|error| {
        format!(
            "failed to resolve workspace root {}: {error}",
            root.display()
        )
    })?;
    if canonical.is_file() {
        canonical
            .parent()
            .map(Path::to_path_buf)
            .ok_or_else(|| format!("workspace entry {} has no parent", canonical.display()))
    } else {
        Ok(canonical)
    }
}

pub(super) fn resolve_target_path(root: &Path, target: &Path) -> Result<PathBuf, String> {
    let candidate = if target.is_absolute() {
        target.to_path_buf()
    } else {
        root.join(target)
    };
    fs::canonicalize(candidate.as_path()).map_err(|error| {
        format!(
            "failed to resolve target style {}: {error}",
            candidate.display()
        )
    })
}

pub(super) fn normalize_selector_name(name: &str) -> Result<String, String> {
    let name = name.trim().strip_prefix('.').unwrap_or(name.trim());
    if name.is_empty() || name.chars().any(char::is_whitespace) {
        return Err("selector names must be non-empty and contain no whitespace".to_string());
    }
    Ok(name.to_string())
}

pub(super) fn normalize_custom_property_name(
    name: &str,
) -> Result<CanonicalCustomPropertyNameV0, String> {
    const INVALID_NAME: &str = "custom-property names must be non-empty and contain no whitespace";
    let name = name.trim();
    let normalized = if is_custom_property_name(name) {
        name.to_string()
    } else {
        format!("--{name}")
    };
    let property = PropertyNameV0::from_authored(&normalized);
    let property_key = property.as_custom_key();
    let valid = property_key.as_ref().is_some_and(|property_key| {
        property_key.as_str().len() > 2 && !property_key.as_str().chars().any(char::is_whitespace)
    });
    if !valid {
        return Err(INVALID_NAME.to_string());
    }
    property_key.ok_or_else(|| INVALID_NAME.to_string())
}

pub(super) fn is_css_module_path(path: &Path) -> bool {
    path.file_name()
        .and_then(|name| name.to_str())
        .is_some_and(|name| name.contains(".module."))
}

pub(super) fn exact_workspace_safety() -> FixSafetyEvidenceInputV0 {
    FixSafetyEvidenceInputV0 {
        syntax_preserving: true,
        local_semantics_required: true,
        local_semantics_ready: true,
        closed_world_required: true,
        closed_world_ready: true,
        reference_precision_required: true,
        reference_precision: Some(FactPrecision::Exact),
    }
}

pub(super) fn conservative_workspace_safety() -> FixSafetyEvidenceInputV0 {
    FixSafetyEvidenceInputV0 {
        reference_precision: Some(FactPrecision::Conservative),
        ..exact_workspace_safety()
    }
}

pub(super) fn manual_review_safety() -> FixSafetyEvidenceInputV0 {
    FixSafetyEvidenceInputV0 {
        syntax_preserving: true,
        local_semantics_required: true,
        local_semantics_ready: false,
        closed_world_required: true,
        closed_world_ready: true,
        reference_precision_required: true,
        reference_precision: Some(FactPrecision::Heuristic),
    }
}

use std::{
    fs,
    path::{Component, Path, PathBuf},
};

use serde::Serialize;
use serde_json::Value;

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OmenaBridgeStyleResolutionSummaryV0 {
    pub schema_version: &'static str,
    pub product: &'static str,
    pub owner_crate: &'static str,
    pub resolver_name: &'static str,
    pub supported_specifier_kinds: Vec<&'static str>,
    pub candidate_extensions: Vec<&'static str>,
    pub request_path_policy: Vec<&'static str>,
}

pub fn summarize_omena_bridge_style_resolution_boundary() -> OmenaBridgeStyleResolutionSummaryV0 {
    OmenaBridgeStyleResolutionSummaryV0 {
        schema_version: "0",
        product: "omena-bridge.style-resolution",
        owner_crate: "omena-bridge",
        resolver_name: "style-import-specifier-resolver",
        supported_specifier_kinds: vec!["relative", "tsconfigPaths", "jsconfigPaths"],
        candidate_extensions: vec!["scss", "sass", "css"],
        request_path_policy: vec![
            "resolverConsumesSourceUriWorkspaceUriAndRawSpecifier",
            "relativeSpecifierExpandsStyleModuleCandidates",
            "pathAliasResolutionUsesNearestWorkspaceTsconfigOrJsconfig",
            "lspServerOwnsOnlyDocumentRoutingAndUriRangeMapping",
        ],
    }
}

pub fn resolve_omena_bridge_style_uri_for_specifier(
    source_uri: &str,
    workspace_folder_uri: Option<&str>,
    specifier: &str,
) -> Option<String> {
    if specifier.starts_with('.') {
        let source_path = file_uri_to_path(source_uri)?;
        let imported_path = normalize_path(source_path.parent()?.join(specifier));
        return style_uri_for_style_candidate_base(imported_path.as_path());
    }

    style_uri_for_tsconfig_path_alias(workspace_folder_uri, specifier)
}

fn style_uri_for_tsconfig_path_alias(
    workspace_folder_uri: Option<&str>,
    specifier: &str,
) -> Option<String> {
    let workspace_path = file_uri_to_path(workspace_folder_uri?)?;
    for config_path in [
        workspace_path.join("tsconfig.json"),
        workspace_path.join("jsconfig.json"),
    ] {
        if let Some(style_uri) =
            style_uri_for_tsconfig_path_alias_config(config_path.as_path(), specifier)
        {
            return Some(style_uri);
        }
    }
    None
}

fn style_uri_for_tsconfig_path_alias_config(config_path: &Path, specifier: &str) -> Option<String> {
    let config_text = fs::read_to_string(config_path).ok()?;
    let config = serde_json::from_str::<Value>(config_text.as_str()).ok()?;
    let compiler_options = config.get("compilerOptions")?;
    let paths = compiler_options.get("paths")?.as_object()?;
    let config_dir = config_path.parent()?;
    let base_url = compiler_options
        .get("baseUrl")
        .and_then(Value::as_str)
        .unwrap_or(".");
    let base_path = normalize_path(config_dir.join(base_url));
    let mut candidates = Vec::new();

    for (pattern, targets) in paths {
        let Some((capture, score)) = tsconfig_path_pattern_match(pattern.as_str(), specifier)
        else {
            continue;
        };
        let Some(targets) = targets.as_array() else {
            continue;
        };
        for target in targets.iter().filter_map(Value::as_str) {
            let candidate_path =
                tsconfig_path_target_candidate(base_path.as_path(), target, capture.as_deref());
            for resolved_path in style_candidate_paths(candidate_path.as_path()) {
                candidates.push((score, resolved_path.exists(), resolved_path));
            }
        }
    }

    candidates.sort_by(|left, right| {
        right
            .1
            .cmp(&left.1)
            .then_with(|| right.0.cmp(&left.0))
            .then_with(|| left.2.cmp(&right.2))
    });
    candidates
        .into_iter()
        .map(|(_, _, path)| path_to_file_uri(path.as_path()))
        .next()
}

fn tsconfig_path_pattern_match(pattern: &str, specifier: &str) -> Option<(Option<String>, usize)> {
    let Some(star_index) = pattern.find('*') else {
        return (pattern == specifier).then_some((None, pattern.len()));
    };
    if pattern[star_index + 1..].contains('*') {
        return None;
    }
    let prefix = &pattern[..star_index];
    let suffix = &pattern[star_index + 1..];
    if !specifier.starts_with(prefix) || !specifier.ends_with(suffix) {
        return None;
    }
    let capture_end = specifier.len().saturating_sub(suffix.len());
    if capture_end < prefix.len() {
        return None;
    }
    Some((
        specifier.get(prefix.len()..capture_end).map(str::to_string),
        prefix.len() + suffix.len(),
    ))
}

fn tsconfig_path_target_candidate(
    base_path: &Path,
    target_pattern: &str,
    capture: Option<&str>,
) -> PathBuf {
    let target = if target_pattern.contains('*') {
        target_pattern.replace('*', capture.unwrap_or_default())
    } else {
        target_pattern.to_string()
    };
    let target_path = PathBuf::from(target);
    if target_path.is_absolute() {
        normalize_path(target_path)
    } else {
        normalize_path(base_path.join(target_path))
    }
}

fn style_uri_for_style_candidate_base(base_path: &Path) -> Option<String> {
    let candidates = style_candidate_paths(base_path);
    candidates
        .iter()
        .find(|path| path.exists())
        .or_else(|| candidates.first())
        .map(|path| path_to_file_uri(path.as_path()))
}

fn style_candidate_paths(base_path: &Path) -> Vec<PathBuf> {
    let normalized = normalize_path(base_path.to_path_buf());
    if is_indexable_style_path(normalized.as_path()) {
        return vec![normalized];
    }

    let mut candidates = Vec::new();
    for extension in ["scss", "sass", "css"] {
        candidates.push(normalized.with_extension(extension));
        if let Some(file_name) = normalized.file_name().and_then(|value| value.to_str()) {
            candidates.push(
                normalized
                    .with_file_name(format!("_{file_name}"))
                    .with_extension(extension),
            );
        }
        candidates.push(normalized.join(format!("index.{extension}")));
        candidates.push(normalized.join(format!("_index.{extension}")));
    }
    candidates.sort();
    candidates.dedup();
    candidates
        .into_iter()
        .filter(|path| is_indexable_style_path(path.as_path()))
        .collect()
}

fn is_indexable_style_path(path: &Path) -> bool {
    let path = path.to_string_lossy();
    path.ends_with(".module.css")
        || path.ends_with(".css")
        || path.ends_with(".module.scss")
        || path.ends_with(".scss")
        || path.ends_with(".module.sass")
        || path.ends_with(".sass")
        || path.ends_with(".module.less")
        || path.ends_with(".less")
}

fn file_uri_to_path(uri: &str) -> Option<PathBuf> {
    let raw_path = uri.strip_prefix("file://")?;
    Some(PathBuf::from(percent_decode_uri_path(raw_path)?))
}

fn percent_decode_uri_path(raw_path: &str) -> Option<String> {
    let bytes = raw_path.as_bytes();
    let mut decoded = Vec::with_capacity(bytes.len());
    let mut index = 0usize;
    while index < bytes.len() {
        if bytes[index] == b'%' {
            let high = bytes.get(index + 1).and_then(|byte| hex_value(*byte))?;
            let low = bytes.get(index + 2).and_then(|byte| hex_value(*byte))?;
            decoded.push((high << 4) | low);
            index += 3;
        } else {
            decoded.push(bytes[index]);
            index += 1;
        }
    }
    String::from_utf8(decoded).ok()
}

fn hex_value(byte: u8) -> Option<u8> {
    match byte {
        b'0'..=b'9' => Some(byte - b'0'),
        b'a'..=b'f' => Some(byte - b'a' + 10),
        b'A'..=b'F' => Some(byte - b'A' + 10),
        _ => None,
    }
}

fn path_to_file_uri(path: &Path) -> String {
    format!("file://{}", path.to_string_lossy())
}

fn normalize_path(path: PathBuf) -> PathBuf {
    let mut normalized = PathBuf::new();
    for component in path.components() {
        match component {
            Component::CurDir => {}
            Component::ParentDir => {
                normalized.pop();
            }
            Component::Normal(_) | Component::RootDir | Component::Prefix(_) => {
                normalized.push(component.as_os_str());
            }
        }
    }
    normalized
}

#[cfg(test)]
mod tests {
    use std::{fs, time::SystemTime};

    use super::*;

    #[test]
    fn resolves_relative_style_candidates() -> Result<(), Box<dyn std::error::Error>> {
        let root = temp_dir("omena_bridge_style_relative")?;
        let source = root.join("src/App.tsx");
        let style = root.join("src/Button.module.scss");
        fs::create_dir_all(
            source
                .parent()
                .ok_or_else(|| std::io::Error::other("parent"))?,
        )?;
        fs::write(&source, "")?;
        fs::write(&style, ".root {}")?;

        let uri = resolve_omena_bridge_style_uri_for_specifier(
            path_to_file_uri(source.as_path()).as_str(),
            Some(path_to_file_uri(root.as_path()).as_str()),
            "./Button.module.scss",
        );

        assert_eq!(
            uri.as_deref(),
            Some(path_to_file_uri(style.as_path()).as_str())
        );
        let _ = fs::remove_dir_all(root);
        Ok(())
    }

    #[test]
    fn resolves_tsconfig_path_alias_style_candidates() -> Result<(), Box<dyn std::error::Error>> {
        let root = temp_dir("omena_bridge_style_alias")?;
        let source = root.join("src/App.tsx");
        let style = root.join("src/styles/Button.module.scss");
        fs::create_dir_all(
            style
                .parent()
                .ok_or_else(|| std::io::Error::other("parent"))?,
        )?;
        fs::write(&source, "")?;
        fs::write(&style, ".root {}")?;
        fs::write(
            root.join("tsconfig.json"),
            r#"{"compilerOptions":{"baseUrl":".","paths":{"@styles/*":["src/styles/*"]}}}"#,
        )?;

        let uri = resolve_omena_bridge_style_uri_for_specifier(
            path_to_file_uri(source.as_path()).as_str(),
            Some(path_to_file_uri(root.as_path()).as_str()),
            "@styles/Button.module.scss",
        );

        assert_eq!(
            uri.as_deref(),
            Some(path_to_file_uri(style.as_path()).as_str())
        );
        let _ = fs::remove_dir_all(root);
        Ok(())
    }

    #[test]
    fn resolves_sass_style_candidates_without_legacy_language_filter()
    -> Result<(), Box<dyn std::error::Error>> {
        let root = temp_dir("omena_bridge_style_sass")?;
        let source = root.join("src/App.tsx");
        let style = root.join("src/Button.module.sass");
        fs::create_dir_all(
            source
                .parent()
                .ok_or_else(|| std::io::Error::other("parent"))?,
        )?;
        fs::write(&source, "")?;
        fs::write(&style, ".root\n  color: red\n")?;

        let uri = resolve_omena_bridge_style_uri_for_specifier(
            path_to_file_uri(source.as_path()).as_str(),
            Some(path_to_file_uri(root.as_path()).as_str()),
            "./Button.module.sass",
        );

        assert_eq!(
            uri.as_deref(),
            Some(path_to_file_uri(style.as_path()).as_str())
        );
        let _ = fs::remove_dir_all(root);
        Ok(())
    }

    #[test]
    fn declares_bridge_owned_style_resolution_boundary() {
        let summary = summarize_omena_bridge_style_resolution_boundary();

        assert_eq!(summary.product, "omena-bridge.style-resolution");
        assert_eq!(summary.owner_crate, "omena-bridge");
        assert!(summary.supported_specifier_kinds.contains(&"tsconfigPaths"));
        assert!(
            summary
                .request_path_policy
                .contains(&"lspServerOwnsOnlyDocumentRoutingAndUriRangeMapping")
        );
    }

    fn temp_dir(prefix: &str) -> Result<PathBuf, Box<dyn std::error::Error>> {
        let suffix = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)?
            .as_nanos();
        let path = std::env::temp_dir().join(format!("{prefix}_{suffix}"));
        fs::create_dir_all(path.as_path())?;
        Ok(path)
    }
}

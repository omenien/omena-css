use std::{
    collections::BTreeSet,
    fs,
    path::{Component, Path, PathBuf},
};

use omena_resolver::{
    OmenaResolverStylePackageManifestV0, OmenaResolverTsconfigPathMappingV0,
    collect_omena_resolver_style_module_source_candidates_with_tsconfig_paths,
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
        supported_specifier_kinds: vec![
            "relative",
            "tsconfigPaths",
            "jsconfigPaths",
            "npmPackages",
        ],
        candidate_extensions: vec!["scss", "sass", "css", "less"],
        request_path_policy: vec![
            "resolverConsumesSourceUriWorkspaceUriAndRawSpecifier",
            "relativeSpecifierExpandsStyleModuleCandidates",
            "pathAliasResolutionUsesNearestWorkspaceTsconfigOrJsconfig",
            "packageSpecifierResolutionUsesOmenaResolver",
            "fileUriOutputIsPercentEncoded",
            "lspServerOwnsOnlyDocumentRoutingAndUriRangeMapping",
        ],
    }
}

pub fn resolve_omena_bridge_style_uri_for_specifier(
    source_uri: &str,
    workspace_folder_uri: Option<&str>,
    specifier: &str,
) -> Option<String> {
    let source_path = normalize_path(file_uri_to_path(source_uri)?);
    let source_path_text = source_path.to_string_lossy().to_string();
    let workspace_path = workspace_folder_uri
        .and_then(file_uri_to_path)
        .map(normalize_path);
    let tsconfig_mappings =
        tsconfig_path_mappings_for_workspace(workspace_path.as_deref()).unwrap_or_default();
    let package_manifests =
        package_manifests_for_specifier(source_path.parent(), specifier).unwrap_or_default();
    let requires_existing_candidate = package_name_from_specifier(specifier).is_some()
        && !tsconfig_mappings
            .iter()
            .any(|mapping| tsconfig_path_pattern_matches(mapping.pattern.as_str(), specifier));
    let candidates = collect_omena_resolver_style_module_source_candidates_with_tsconfig_paths(
        source_path_text.as_str(),
        specifier,
        package_manifests.as_slice(),
        tsconfig_mappings.as_slice(),
    );

    style_uri_for_resolver_candidates(candidates.as_slice(), requires_existing_candidate)
}

fn tsconfig_path_mappings_for_workspace(
    workspace_path: Option<&Path>,
) -> Option<Vec<OmenaResolverTsconfigPathMappingV0>> {
    let workspace_path = workspace_path?;
    let mut mappings = Vec::new();
    for config_path in [
        workspace_path.join("tsconfig.json"),
        workspace_path.join("jsconfig.json"),
    ] {
        mappings.extend(tsconfig_path_mappings_for_config(config_path.as_path()));
    }
    Some(mappings)
}

fn tsconfig_path_mappings_for_config(
    config_path: &Path,
) -> Vec<OmenaResolverTsconfigPathMappingV0> {
    let Some(config_text) = fs::read_to_string(config_path).ok() else {
        return Vec::new();
    };
    let Some(config) = serde_json::from_str::<Value>(config_text.as_str()).ok() else {
        return Vec::new();
    };
    tsconfig_path_mappings_from_value(config_path, &config).unwrap_or_default()
}

fn tsconfig_path_mappings_from_value(
    config_path: &Path,
    config: &Value,
) -> Option<Vec<OmenaResolverTsconfigPathMappingV0>> {
    let compiler_options = config.get("compilerOptions")?;
    let paths = compiler_options.get("paths")?.as_object()?;
    let config_dir = config_path.parent()?;
    let base_url = compiler_options
        .get("baseUrl")
        .and_then(Value::as_str)
        .unwrap_or(".");
    let base_path = normalize_path(config_dir.join(base_url));
    let mut mappings = Vec::new();
    for (pattern, targets) in paths {
        let Some(targets) = targets.as_array() else {
            continue;
        };
        let target_patterns = targets
            .iter()
            .filter_map(Value::as_str)
            .map(ToString::to_string)
            .collect::<Vec<_>>();
        if target_patterns.is_empty() {
            continue;
        }
        mappings.push(OmenaResolverTsconfigPathMappingV0 {
            base_path: base_path.to_string_lossy().to_string(),
            pattern: pattern.to_string(),
            target_patterns,
        });
    }
    Some(mappings)
}

fn package_manifests_for_specifier(
    source_dir: Option<&Path>,
    specifier: &str,
) -> Option<Vec<OmenaResolverStylePackageManifestV0>> {
    let package_name = package_name_from_specifier(specifier)?;
    let mut manifests = Vec::new();
    let mut seen = BTreeSet::new();
    let mut current_dir = source_dir;
    while let Some(dir) = current_dir {
        let package_json_path = dir
            .join("node_modules")
            .join(package_name)
            .join("package.json");
        if seen.insert(package_json_path.clone())
            && let Ok(package_json_source) = fs::read_to_string(package_json_path.as_path())
        {
            manifests.push(OmenaResolverStylePackageManifestV0 {
                package_json_path: normalize_path(package_json_path)
                    .to_string_lossy()
                    .to_string(),
                package_json_source,
            });
        }
        current_dir = dir.parent();
    }
    Some(manifests)
}

fn package_name_from_specifier(specifier: &str) -> Option<&str> {
    if specifier.starts_with('.')
        || specifier.starts_with('/')
        || is_external_style_specifier(specifier)
    {
        return None;
    }
    if specifier.starts_with('@') {
        let mut segments = specifier.splitn(3, '/');
        let scope = segments.next()?;
        let package = segments.next()?;
        if scope.len() <= 1 || package.is_empty() {
            return None;
        }
        return specifier.get(..scope.len() + 1 + package.len());
    }
    specifier.split('/').next().filter(|name| !name.is_empty())
}

fn tsconfig_path_pattern_matches(pattern: &str, specifier: &str) -> bool {
    if let Some((prefix, suffix)) = pattern.split_once('*') {
        return !suffix.contains('*')
            && specifier.starts_with(prefix)
            && specifier.ends_with(suffix)
            && specifier.len() >= prefix.len() + suffix.len();
    }
    pattern == specifier
}

fn is_external_style_specifier(specifier: &str) -> bool {
    specifier.starts_with("sass:")
        || specifier.starts_with("http://")
        || specifier.starts_with("https://")
}

fn style_uri_for_resolver_candidates(
    candidates: &[String],
    requires_existing_candidate: bool,
) -> Option<String> {
    candidates
        .iter()
        .map(PathBuf::from)
        .find(|path| path.exists() && is_indexable_style_path(path.as_path()))
        .or_else(|| {
            if requires_existing_candidate {
                return None;
            }
            candidates
                .iter()
                .map(PathBuf::from)
                .find(|path| is_indexable_style_path(path.as_path()))
        })
        .map(|path| path_to_file_uri(normalize_path(path).as_path()))
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
    format!(
        "file://{}",
        percent_encode_uri_path(path.to_string_lossy().as_ref())
    )
}

fn percent_encode_uri_path(path: &str) -> String {
    let mut encoded = String::with_capacity(path.len());
    for byte in path.as_bytes() {
        match *byte {
            b'A'..=b'Z'
            | b'a'..=b'z'
            | b'0'..=b'9'
            | b'-'
            | b'.'
            | b'_'
            | b'~'
            | b'/'
            | b'@'
            | b':'
            | b'!'
            | b'$'
            | b'&'
            | b'\''
            | b'*'
            | b'+'
            | b','
            | b';'
            | b'=' => encoded.push(*byte as char),
            _ => encoded.push_str(format!("%{byte:02X}").as_str()),
        }
    }
    encoded
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
    fn resolves_package_style_candidates_through_omena_resolver()
    -> Result<(), Box<dyn std::error::Error>> {
        let root = temp_dir("omena_bridge_style_package")?;
        let source = root.join("src/App.module.scss");
        let package_root = root.join("node_modules/@design/tokens");
        let style = package_root.join("src/index.scss");
        fs::create_dir_all(
            style
                .parent()
                .ok_or_else(|| std::io::Error::other("parent"))?,
        )?;
        fs::create_dir_all(
            source
                .parent()
                .ok_or_else(|| std::io::Error::other("source parent"))?,
        )?;
        fs::write(&source, "@use \"@design/tokens\";")?;
        fs::write(
            package_root.join("package.json"),
            r#"{"sass":"src/index.scss"}"#,
        )?;
        fs::write(&style, "$gap: 1rem;")?;

        let uri = resolve_omena_bridge_style_uri_for_specifier(
            path_to_file_uri(source.as_path()).as_str(),
            Some(path_to_file_uri(root.as_path()).as_str()),
            "@design/tokens",
        );

        assert_eq!(
            uri.as_deref(),
            Some(path_to_file_uri(style.as_path()).as_str())
        );
        let _ = fs::remove_dir_all(root);
        Ok(())
    }

    #[test]
    fn does_not_fabricate_missing_package_style_candidates()
    -> Result<(), Box<dyn std::error::Error>> {
        let root = temp_dir("omena_bridge_style_missing_package")?;
        let source = root.join("src/App.tsx");
        fs::create_dir_all(
            source
                .parent()
                .ok_or_else(|| std::io::Error::other("parent"))?,
        )?;
        fs::write(&source, "")?;

        let uri = resolve_omena_bridge_style_uri_for_specifier(
            path_to_file_uri(source.as_path()).as_str(),
            Some(path_to_file_uri(root.as_path()).as_str()),
            "@design/tokens",
        );

        assert!(uri.is_none(), "{uri:?}");
        let _ = fs::remove_dir_all(root);
        Ok(())
    }

    #[test]
    fn emits_percent_encoded_file_uris_for_route_group_paths()
    -> Result<(), Box<dyn std::error::Error>> {
        let root = temp_dir("omena_bridge_style_route_group")?;
        let source = root.join("app/(marketing)/page.tsx");
        let style = root.join("app/(marketing)/Card.module.scss");
        fs::create_dir_all(
            source
                .parent()
                .ok_or_else(|| std::io::Error::other("parent"))?,
        )?;
        fs::write(&source, "")?;
        fs::write(&style, ".card {}")?;

        let uri = resolve_omena_bridge_style_uri_for_specifier(
            path_to_file_uri(source.as_path()).as_str(),
            Some(path_to_file_uri(root.as_path()).as_str()),
            "./Card.module.scss",
        )
        .ok_or_else(|| std::io::Error::other("route group style should resolve"))?;

        assert!(uri.contains("%28marketing%29"), "{uri}");
        assert_eq!(uri, path_to_file_uri(style.as_path()));
        let _ = fs::remove_dir_all(root);
        Ok(())
    }

    #[test]
    fn declares_bridge_owned_style_resolution_boundary() {
        let summary = summarize_omena_bridge_style_resolution_boundary();

        assert_eq!(summary.product, "omena-bridge.style-resolution");
        assert_eq!(summary.owner_crate, "omena-bridge");
        assert!(summary.supported_specifier_kinds.contains(&"tsconfigPaths"));
        assert!(summary.supported_specifier_kinds.contains(&"npmPackages"));
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

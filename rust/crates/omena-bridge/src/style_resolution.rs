use std::{
    collections::BTreeSet,
    ffi::OsString,
    fs,
    path::{Component, Path, PathBuf},
};

use crate::bundler_config_alias::load_omena_bridge_workspace_bundler_path_alias_mappings;
use omena_resolver::{
    OmenaResolverBundlerPathAliasMappingV0, OmenaResolverStylePackageManifestV0,
    OmenaResolverTsconfigPathMappingV0,
    collect_omena_resolver_style_module_source_candidates_with_path_mappings,
};
use serde::Serialize;
use serde_json::Value;

const WORKSPACE_PACKAGE_MANIFEST_SCAN_LIMIT: usize = 1024;

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

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OmenaBridgeStyleResolutionInputsV0 {
    pub package_manifests: Vec<OmenaResolverStylePackageManifestV0>,
    pub tsconfig_path_mappings: Vec<OmenaResolverTsconfigPathMappingV0>,
    pub bundler_path_mappings: Vec<OmenaResolverBundlerPathAliasMappingV0>,
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
            "bundlerAliases",
            "npmPackages",
            "packageImports",
        ],
        candidate_extensions: vec!["scss", "sass", "css", "less"],
        request_path_policy: vec![
            "resolverConsumesSourceUriWorkspaceUriAndRawSpecifier",
            "relativeSpecifierExpandsStyleModuleCandidates",
            "pathAliasResolutionUsesNearestWorkspaceTsconfigOrJsconfig",
            "pathAliasResolutionFollowsRelativeTsconfigExtends",
            "bundlerAliasResolutionUsesLiteralViteWebpackConfig",
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
    resolve_omena_bridge_style_uri_for_specifier_with_package_manifests(
        source_uri,
        workspace_folder_uri,
        specifier,
        &[],
    )
}

pub fn resolve_omena_bridge_style_uri_for_specifier_with_package_manifests(
    source_uri: &str,
    workspace_folder_uri: Option<&str>,
    specifier: &str,
    configured_package_manifests: &[OmenaResolverStylePackageManifestV0],
) -> Option<String> {
    let source_path = normalize_path(file_uri_to_path(source_uri)?);
    let workspace_path = workspace_folder_uri
        .and_then(file_uri_to_path)
        .map(normalize_path);
    let package_manifests = merged_package_manifests_for_request(
        source_path.parent(),
        workspace_path.as_deref(),
        specifier,
        configured_package_manifests,
    );
    let inputs = OmenaBridgeStyleResolutionInputsV0 {
        package_manifests,
        tsconfig_path_mappings: tsconfig_path_mappings_for_workspace(workspace_path.as_deref())
            .unwrap_or_default(),
        bundler_path_mappings: load_omena_bridge_workspace_bundler_path_alias_mappings(
            workspace_path.as_deref(),
        ),
    };
    resolve_omena_bridge_style_uri_for_specifier_with_resolution_inputs(
        source_uri,
        workspace_folder_uri,
        specifier,
        &inputs,
    )
}

pub fn resolve_omena_bridge_style_uri_for_specifier_with_resolution_inputs(
    source_uri: &str,
    _workspace_folder_uri: Option<&str>,
    specifier: &str,
    resolution_inputs: &OmenaBridgeStyleResolutionInputsV0,
) -> Option<String> {
    let source_path = normalize_path(file_uri_to_path(source_uri)?);
    let source_path_text = source_path.to_string_lossy().to_string();
    let requires_existing_candidate = (package_name_from_specifier(specifier).is_some()
        || is_package_import_specifier(specifier))
        && !resolution_inputs
            .tsconfig_path_mappings
            .iter()
            .any(|mapping| tsconfig_path_pattern_matches(mapping.pattern.as_str(), specifier))
        && !resolution_inputs
            .bundler_path_mappings
            .iter()
            .any(|mapping| bundler_path_alias_pattern_matches(mapping.pattern.as_str(), specifier));
    let candidates = collect_omena_resolver_style_module_source_candidates_with_path_mappings(
        source_path_text.as_str(),
        specifier,
        resolution_inputs.package_manifests.as_slice(),
        resolution_inputs.bundler_path_mappings.as_slice(),
        resolution_inputs.tsconfig_path_mappings.as_slice(),
    );

    style_uri_for_resolver_candidates(candidates.as_slice(), requires_existing_candidate)
}

pub fn load_omena_bridge_workspace_style_resolution_inputs(
    workspace_folder_uri: Option<&str>,
    configured_package_manifests: &[OmenaResolverStylePackageManifestV0],
) -> OmenaBridgeStyleResolutionInputsV0 {
    let workspace_path = workspace_folder_uri
        .and_then(file_uri_to_path)
        .map(normalize_path);
    load_omena_bridge_workspace_style_resolution_inputs_from_path(
        workspace_path.as_deref(),
        configured_package_manifests,
    )
}

fn load_omena_bridge_workspace_style_resolution_inputs_from_path(
    workspace_path: Option<&Path>,
    configured_package_manifests: &[OmenaResolverStylePackageManifestV0],
) -> OmenaBridgeStyleResolutionInputsV0 {
    OmenaBridgeStyleResolutionInputsV0 {
        package_manifests: merge_package_manifest_lists(
            configured_package_manifests,
            workspace_package_manifests(workspace_path).as_slice(),
        ),
        tsconfig_path_mappings: tsconfig_path_mappings_for_workspace(workspace_path)
            .unwrap_or_default(),
        bundler_path_mappings: load_omena_bridge_workspace_bundler_path_alias_mappings(
            workspace_path,
        ),
    }
}

fn merge_package_manifest_lists(
    primary: &[OmenaResolverStylePackageManifestV0],
    secondary: &[OmenaResolverStylePackageManifestV0],
) -> Vec<OmenaResolverStylePackageManifestV0> {
    let mut manifests = primary.to_vec();
    let mut seen = manifests
        .iter()
        .map(|manifest| manifest.package_json_path.clone())
        .collect::<BTreeSet<_>>();
    for manifest in secondary {
        if seen.insert(manifest.package_json_path.clone()) {
            manifests.push(manifest.clone());
        }
    }
    manifests
}

fn merged_package_manifests_for_specifier(
    source_dir: Option<&Path>,
    specifier: &str,
    configured_package_manifests: &[OmenaResolverStylePackageManifestV0],
) -> Vec<OmenaResolverStylePackageManifestV0> {
    merge_package_manifest_lists(
        configured_package_manifests,
        package_manifests_for_specifier(source_dir, specifier)
            .unwrap_or_default()
            .as_slice(),
    )
}

fn merged_package_manifests_for_request(
    source_dir: Option<&Path>,
    workspace_path: Option<&Path>,
    specifier: &str,
    configured_package_manifests: &[OmenaResolverStylePackageManifestV0],
) -> Vec<OmenaResolverStylePackageManifestV0> {
    let source_manifests =
        merged_package_manifests_for_specifier(source_dir, specifier, configured_package_manifests);
    merge_package_manifest_lists(
        source_manifests.as_slice(),
        workspace_package_manifests(workspace_path).as_slice(),
    )
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
    tsconfig_path_mappings_for_config_with_seen(config_path, &mut BTreeSet::new())
}

fn tsconfig_path_mappings_for_config_with_seen(
    config_path: &Path,
    seen: &mut BTreeSet<PathBuf>,
) -> Vec<OmenaResolverTsconfigPathMappingV0> {
    let normalized_config_path = normalize_path(config_path.to_path_buf());
    if !seen.insert(normalized_config_path.clone()) {
        return Vec::new();
    }
    let Some(config_text) = fs::read_to_string(config_path).ok() else {
        return Vec::new();
    };
    let Some(config) = serde_json::from_str::<Value>(config_text.as_str()).ok() else {
        return Vec::new();
    };
    let own_mappings = tsconfig_path_mappings_from_value(config_path, &config).unwrap_or_default();
    if !own_mappings.is_empty() {
        return own_mappings;
    }
    resolve_tsconfig_extends_path(config_path, &config)
        .map(|extends_path| {
            tsconfig_path_mappings_for_config_with_seen(extends_path.as_path(), seen)
        })
        .unwrap_or_default()
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

fn resolve_tsconfig_extends_path(config_path: &Path, config: &Value) -> Option<PathBuf> {
    let extends = config.get("extends")?.as_str()?;
    if !extends.starts_with('.') {
        return None;
    }
    let config_dir = config_path.parent()?;
    let raw_path = config_dir.join(extends);
    tsconfig_extends_candidates(raw_path)
        .into_iter()
        .find(|candidate| candidate.exists())
}

fn tsconfig_extends_candidates(path: PathBuf) -> Vec<PathBuf> {
    if path.extension().is_some() {
        return vec![path];
    }
    vec![path.with_extension("json"), path.join("tsconfig.json")]
}

fn package_manifests_for_specifier(
    source_dir: Option<&Path>,
    specifier: &str,
) -> Option<Vec<OmenaResolverStylePackageManifestV0>> {
    if is_package_import_specifier(specifier) {
        return Some(package_scope_manifests_for_source_dir(source_dir));
    }
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

fn package_scope_manifests_for_source_dir(
    source_dir: Option<&Path>,
) -> Vec<OmenaResolverStylePackageManifestV0> {
    let mut manifests = Vec::new();
    let mut current_dir = source_dir;
    while let Some(dir) = current_dir {
        push_workspace_package_manifest(dir.join("package.json"), &mut manifests);
        current_dir = dir.parent();
    }
    manifests
}

fn workspace_package_manifests(
    workspace_path: Option<&Path>,
) -> Vec<OmenaResolverStylePackageManifestV0> {
    let Some(workspace_path) = workspace_path else {
        return Vec::new();
    };
    let mut manifests = Vec::new();
    push_workspace_package_manifest(workspace_path.join("package.json"), &mut manifests);

    let node_modules = workspace_path.join("node_modules");
    let Ok(entries) = fs::read_dir(node_modules.as_path()) else {
        return manifests;
    };
    for entry in entries.flatten() {
        if manifests.len() >= WORKSPACE_PACKAGE_MANIFEST_SCAN_LIMIT {
            break;
        }
        let path = entry.path();
        let Some(file_name) = path.file_name().and_then(|value| value.to_str()) else {
            continue;
        };
        if file_name.starts_with('@') {
            push_scoped_workspace_package_manifests(path.as_path(), &mut manifests);
        } else {
            push_workspace_package_manifest(path.join("package.json"), &mut manifests);
        }
    }
    manifests.sort_by(|left, right| left.package_json_path.cmp(&right.package_json_path));
    manifests.dedup_by(|left, right| left.package_json_path == right.package_json_path);
    manifests
}

fn push_scoped_workspace_package_manifests(
    scope_path: &Path,
    manifests: &mut Vec<OmenaResolverStylePackageManifestV0>,
) {
    let Ok(entries) = fs::read_dir(scope_path) else {
        return;
    };
    for entry in entries.flatten() {
        if manifests.len() >= WORKSPACE_PACKAGE_MANIFEST_SCAN_LIMIT {
            return;
        }
        push_workspace_package_manifest(entry.path().join("package.json"), manifests);
    }
}

fn push_workspace_package_manifest(
    package_json_path: PathBuf,
    manifests: &mut Vec<OmenaResolverStylePackageManifestV0>,
) {
    if manifests.len() >= WORKSPACE_PACKAGE_MANIFEST_SCAN_LIMIT {
        return;
    }
    let normalized_package_json_path = normalize_path(package_json_path);
    let package_json_path_text = normalized_package_json_path.to_string_lossy().to_string();
    if manifests
        .iter()
        .any(|manifest| manifest.package_json_path == package_json_path_text)
    {
        return;
    }
    let Ok(package_json_source) = fs::read_to_string(normalized_package_json_path.as_path()) else {
        return;
    };
    manifests.push(OmenaResolverStylePackageManifestV0 {
        package_json_path: package_json_path_text,
        package_json_source,
    });
}

fn package_name_from_specifier(specifier: &str) -> Option<&str> {
    let specifier = specifier.strip_prefix("pkg:").unwrap_or(specifier);
    if specifier.starts_with('.')
        || specifier.starts_with('/')
        || is_package_import_specifier(specifier)
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

fn is_package_import_specifier(specifier: &str) -> bool {
    specifier
        .strip_prefix("pkg:")
        .unwrap_or(specifier)
        .starts_with('#')
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

fn bundler_path_alias_pattern_matches(pattern: &str, specifier: &str) -> bool {
    if pattern.is_empty() {
        return false;
    }
    if let Some(exact_pattern) = pattern.strip_suffix('$') {
        return specifier == exact_pattern;
    }
    if pattern == specifier {
        return true;
    }
    let prefix = if pattern.ends_with('/') {
        pattern.to_string()
    } else {
        format!("{pattern}/")
    };
    specifier.starts_with(prefix.as_str())
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
    let path = normalize_path(path.to_path_buf());
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
    if let Some(canonical) = canonicalize_existing_path_or_parent(path.as_path()) {
        return normalize_path_lexical(canonical);
    }
    normalize_path_lexical(path)
}

fn canonicalize_existing_path_or_parent(path: &Path) -> Option<PathBuf> {
    if let Ok(canonical) = fs::canonicalize(path) {
        return Some(canonical);
    }

    let mut current = path.to_path_buf();
    let mut suffix = Vec::<OsString>::new();
    while let Some(parent) = current.parent() {
        if let Some(file_name) = current.file_name() {
            suffix.push(file_name.to_os_string());
        }
        if let Ok(mut canonical_parent) = fs::canonicalize(parent) {
            for segment in suffix.iter().rev() {
                canonical_parent.push(segment);
            }
            return Some(canonical_parent);
        }
        current = parent.to_path_buf();
    }
    None
}

fn normalize_path_lexical(path: PathBuf) -> PathBuf {
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
    fn resolves_tsconfig_extends_path_alias_style_candidates()
    -> Result<(), Box<dyn std::error::Error>> {
        let root = temp_dir("omena_bridge_style_alias_extends")?;
        let source = root.join("src/App.tsx");
        let style = root.join("src/shared/Button.module.scss");
        let config_dir = root.join("config");
        fs::create_dir_all(
            style
                .parent()
                .ok_or_else(|| std::io::Error::other("parent"))?,
        )?;
        fs::create_dir_all(config_dir.as_path())?;
        fs::write(&source, "")?;
        fs::write(&style, ".root {}")?;
        fs::write(
            config_dir.join("base.json"),
            r#"{"compilerOptions":{"baseUrl":"..","paths":{"$shared/*":["src/shared/*"]}}}"#,
        )?;
        fs::write(root.join("tsconfig.json"), r#"{"extends":"./config/base"}"#)?;

        let uri = resolve_omena_bridge_style_uri_for_specifier(
            path_to_file_uri(source.as_path()).as_str(),
            Some(path_to_file_uri(root.as_path()).as_str()),
            "$shared/Button.module.scss",
        );

        assert_eq!(
            uri.as_deref(),
            Some(path_to_file_uri(style.as_path()).as_str())
        );
        let _ = fs::remove_dir_all(root);
        Ok(())
    }

    #[test]
    fn tsconfig_extends_child_paths_override_parent_paths() -> Result<(), Box<dyn std::error::Error>>
    {
        let root = temp_dir("omena_bridge_style_alias_extends_override")?;
        let source = root.join("src/App.tsx");
        let parent_style = root.join("src/parent/Button.module.scss");
        let child_style = root.join("src/child/Button.module.scss");
        fs::create_dir_all(
            parent_style
                .parent()
                .ok_or_else(|| std::io::Error::other("parent"))?,
        )?;
        fs::create_dir_all(
            child_style
                .parent()
                .ok_or_else(|| std::io::Error::other("child"))?,
        )?;
        fs::write(&source, "")?;
        fs::write(&parent_style, ".root { color: red; }")?;
        fs::write(&child_style, ".root { color: green; }")?;
        fs::write(
            root.join("base.json"),
            r#"{"compilerOptions":{"baseUrl":".","paths":{"$shared/*":["src/parent/*"]}}}"#,
        )?;
        fs::write(
            root.join("tsconfig.json"),
            r#"{"extends":"./base.json","compilerOptions":{"baseUrl":".","paths":{"$shared/*":["src/child/*"]}}}"#,
        )?;

        let uri = resolve_omena_bridge_style_uri_for_specifier(
            path_to_file_uri(source.as_path()).as_str(),
            Some(path_to_file_uri(root.as_path()).as_str()),
            "$shared/Button.module.scss",
        );

        assert_eq!(
            uri.as_deref(),
            Some(path_to_file_uri(child_style.as_path()).as_str())
        );
        let _ = fs::remove_dir_all(root);
        Ok(())
    }

    #[test]
    fn resolves_vite_bundler_alias_style_candidates() -> Result<(), Box<dyn std::error::Error>> {
        let root = temp_dir("omena_bridge_style_bundler_alias")?;
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
            root.join("vite.config.ts"),
            r#"export default { resolve: { alias: { "@styles": "./src/styles" } } };"#,
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
    fn resolves_webpack_exact_bundler_alias_style_candidates()
    -> Result<(), Box<dyn std::error::Error>> {
        let root = temp_dir("omena_bridge_style_bundler_exact_alias")?;
        let source = root.join("src/App.tsx");
        let style = root.join("src/styles/index.module.scss");
        fs::create_dir_all(
            style
                .parent()
                .ok_or_else(|| std::io::Error::other("parent"))?,
        )?;
        fs::write(&source, "")?;
        fs::write(&style, ".root {}")?;
        fs::write(
            root.join("webpack.config.js"),
            r#"module.exports = { resolve: { alias: [{ find: "@theme$", replacement: "./src/styles/index.module.scss" }] } };"#,
        )?;

        let exact_uri = resolve_omena_bridge_style_uri_for_specifier(
            path_to_file_uri(source.as_path()).as_str(),
            Some(path_to_file_uri(root.as_path()).as_str()),
            "@theme",
        );
        let prefix_uri = resolve_omena_bridge_style_uri_for_specifier(
            path_to_file_uri(source.as_path()).as_str(),
            Some(path_to_file_uri(root.as_path()).as_str()),
            "@theme/Button.module.scss",
        );

        assert_eq!(
            exact_uri.as_deref(),
            Some(path_to_file_uri(style.as_path()).as_str())
        );
        assert!(prefix_uri.is_none());
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
    fn resolves_sass_pkg_style_candidates_through_manifest_discovery()
    -> Result<(), Box<dyn std::error::Error>> {
        let root = temp_dir("omena_bridge_style_pkg_manifest")?;
        let source = root.join("src/App.module.scss");
        let package_root = root.join("node_modules/@design/tokens");
        let style = package_root.join("dist/theme.scss");
        fs::create_dir_all(
            style
                .parent()
                .ok_or_else(|| std::io::Error::other("style parent"))?,
        )?;
        fs::create_dir_all(
            source
                .parent()
                .ok_or_else(|| std::io::Error::other("source parent"))?,
        )?;
        fs::write(&source, "@use \"pkg:@design/tokens/theme\";")?;
        fs::write(
            package_root.join("package.json"),
            r#"{"exports":{"./theme":{"sass":"./dist/theme.scss"}}}"#,
        )?;
        fs::write(&style, "$gap: 1rem;")?;

        let uri = resolve_omena_bridge_style_uri_for_specifier(
            path_to_file_uri(source.as_path()).as_str(),
            Some(path_to_file_uri(root.as_path()).as_str()),
            "pkg:@design/tokens/theme",
        );

        assert_eq!(
            uri.as_deref(),
            Some(path_to_file_uri(style.as_path()).as_str())
        );
        let _ = fs::remove_dir_all(root);
        Ok(())
    }

    #[test]
    fn resolves_package_import_style_candidates_through_workspace_manifests()
    -> Result<(), Box<dyn std::error::Error>> {
        let root = temp_dir("omena_bridge_style_package_import_manifest")?;
        let source = root.join("src/App.module.scss");
        let package_root = root.join("node_modules/@design/tokens");
        let style = package_root.join("dist/theme.scss");
        fs::create_dir_all(
            style
                .parent()
                .ok_or_else(|| std::io::Error::other("style parent"))?,
        )?;
        fs::create_dir_all(
            source
                .parent()
                .ok_or_else(|| std::io::Error::other("source parent"))?,
        )?;
        fs::write(&source, "@use \"#theme\" as tokens;")?;
        fs::write(
            root.join("package.json"),
            r##"{"imports":{"#theme":"@design/tokens/theme"}}"##,
        )?;
        fs::write(
            package_root.join("package.json"),
            r#"{"exports":{"./theme":{"sass":"./dist/theme.scss"}}}"#,
        )?;
        fs::write(&style, "$gap: 1rem;")?;

        let uri = resolve_omena_bridge_style_uri_for_specifier(
            path_to_file_uri(source.as_path()).as_str(),
            Some(path_to_file_uri(root.as_path()).as_str()),
            "#theme",
        );

        assert_eq!(
            uri.as_deref(),
            Some(path_to_file_uri(style.as_path()).as_str())
        );
        let _ = fs::remove_dir_all(root);
        Ok(())
    }

    #[cfg(unix)]
    #[test]
    fn resolves_symlinked_package_style_candidates_to_canonical_uri()
    -> Result<(), Box<dyn std::error::Error>> {
        let root = temp_dir("omena_bridge_style_symlinked_package")?;
        let source = root.join("src/App.module.scss");
        let real_package = root.join(".pnpm/@design+tokens@1.0.0/node_modules/@design/tokens");
        let linked_scope = root.join("node_modules/@design");
        let linked_package = linked_scope.join("tokens");
        let style = real_package.join("src/index.scss");
        fs::create_dir_all(
            style
                .parent()
                .ok_or_else(|| std::io::Error::other("style parent"))?,
        )?;
        fs::create_dir_all(
            source
                .parent()
                .ok_or_else(|| std::io::Error::other("source parent"))?,
        )?;
        fs::create_dir_all(linked_scope.as_path())?;
        fs::write(&source, "@use \"@design/tokens\";")?;
        fs::write(
            real_package.join("package.json"),
            r#"{"sass":"src/index.scss"}"#,
        )?;
        fs::write(&style, "$gap: 1rem;")?;
        std::os::unix::fs::symlink(real_package.as_path(), linked_package.as_path())?;

        let uri = resolve_omena_bridge_style_uri_for_specifier(
            path_to_file_uri(source.as_path()).as_str(),
            Some(path_to_file_uri(root.as_path()).as_str()),
            "@design/tokens",
        );
        let expected_uri = path_to_file_uri(fs::canonicalize(style)?.as_path());

        assert_eq!(uri.as_deref(), Some(expected_uri.as_str()));
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
        assert!(
            summary
                .supported_specifier_kinds
                .contains(&"bundlerAliases")
        );
        assert!(summary.supported_specifier_kinds.contains(&"npmPackages"));
        assert!(
            summary
                .request_path_policy
                .contains(&"pathAliasResolutionFollowsRelativeTsconfigExtends")
        );
        assert!(
            summary
                .request_path_policy
                .contains(&"bundlerAliasResolutionUsesLiteralViteWebpackConfig")
        );
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

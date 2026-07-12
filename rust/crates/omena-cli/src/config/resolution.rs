use globset::GlobBuilder;
use serde_json::{Map, Value};
use sha2::{Digest, Sha256};
use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    path::{Path, PathBuf},
};

use super::{
    report::OmenaConfigReport,
    schema::{OmenaConfig, OmenaTranslationValidationMode},
};

const CONFIG_SECTION_NAMES: &[&str] = &[
    "workspace",
    "style",
    "lint",
    "format",
    "minify",
    "modules",
    "sass",
    "intelligence",
    "verify",
    "ci",
    "build",
];

#[derive(Debug, Clone)]
pub(super) struct ResolvedConfigDocument {
    pub(super) config: OmenaConfig,
    pub(super) reports: Vec<OmenaConfigReport>,
    pub(super) config_content_digest: String,
}

#[derive(Debug)]
struct ResolutionInput {
    kind: &'static str,
    path: PathBuf,
    content: Vec<u8>,
}

pub(super) fn resolve_config_document(
    config_path: &Path,
    target_path: &Path,
) -> Result<ResolvedConfigDocument, String> {
    resolve_config_document_with_env(config_path, target_path, &|name| std::env::var(name).ok())
}

fn resolve_config_document_with_env(
    config_path: &Path,
    target_path: &Path,
    env_lookup: &dyn Fn(&str) -> Option<String>,
) -> Result<ResolvedConfigDocument, String> {
    let config_path = canonical_existing_path(config_path)?;
    let target_path = absolute_path(target_path)?;
    let mut inputs = Vec::new();
    let mut env_values = BTreeMap::new();
    let mut stack = Vec::new();
    let mut value = resolve_extends_chain(
        &config_path,
        &mut stack,
        &mut inputs,
        &mut env_values,
        env_lookup,
    )?;

    apply_matching_overrides(&mut value, &config_path, &target_path)?;
    apply_editorconfig_defaults(&mut value, &target_path, &mut inputs)?;

    let (config, unknown_paths) = deserialize_config(&value, &config_path)?;
    let mut reports = unknown_paths
        .into_iter()
        .map(OmenaConfigReport::unknown)
        .collect::<Vec<_>>();
    append_not_yet_consumed_reports(&value, &config, &mut reports);
    reports.sort_by(|left, right| {
        (left.kind.as_str(), left.path.as_str()).cmp(&(right.kind.as_str(), right.path.as_str()))
    });

    let config_content_digest =
        digest_resolution(&config_path, &target_path, &value, &inputs, &env_values)?;
    Ok(ResolvedConfigDocument {
        config,
        reports,
        config_content_digest,
    })
}

fn resolve_extends_chain(
    config_path: &Path,
    stack: &mut Vec<PathBuf>,
    inputs: &mut Vec<ResolutionInput>,
    env_values: &mut BTreeMap<String, String>,
    env_lookup: &dyn Fn(&str) -> Option<String>,
) -> Result<Value, String> {
    let config_path = canonical_existing_path(config_path)?;
    if let Some(index) = stack.iter().position(|path| path == &config_path) {
        let mut cycle = stack[index..].to_vec();
        cycle.push(config_path);
        return Err(format!(
            "Omena config extends cycle: {}",
            cycle
                .iter()
                .map(|path| path.display().to_string())
                .collect::<Vec<_>>()
                .join(" -> ")
        ));
    }

    stack.push(config_path.clone());
    let source = fs::read(&config_path).map_err(|error| {
        format!(
            "failed to read Omena config {}: {error}",
            config_path.display()
        )
    })?;
    inputs.push(ResolutionInput {
        kind: "config",
        path: config_path.clone(),
        content: source.clone(),
    });
    let source_text = std::str::from_utf8(&source).map_err(|error| {
        format!(
            "Omena config {} is not valid UTF-8: {error}",
            config_path.display()
        )
    })?;
    let mut current = parse_config_document(&config_path, source_text)?;
    normalize_legacy_build_document(&config_path, &mut current);
    interpolate_environment(&mut current, env_lookup, env_values)?;

    let extends = extract_extends_paths(&current, &config_path)?;
    let mut merged = Value::Object(Map::new());
    for extends_path in extends {
        if extends_path
            .to_str()
            .is_some_and(|path| path.starts_with("http://") || path.starts_with("https://"))
        {
            return Err(format!(
                "remote Omena config extends are not supported: {}",
                extends_path.display()
            ));
        }
        let base_path = if extends_path.is_absolute() {
            extends_path
        } else {
            config_path
                .parent()
                .unwrap_or_else(|| Path::new("."))
                .join(extends_path)
        };
        let base = resolve_extends_chain(&base_path, stack, inputs, env_values, env_lookup)?;
        merge_config_values(&mut merged, base);
    }
    merge_config_values(&mut merged, current);
    stack.pop();
    Ok(merged)
}

fn parse_config_document(config_path: &Path, source: &str) -> Result<Value, String> {
    match config_path
        .extension()
        .and_then(|extension| extension.to_str())
    {
        Some("toml") => {
            let value = toml::from_str::<toml::Value>(source).map_err(|error| {
                format!(
                    "failed to parse Omena TOML config {}: {error}",
                    config_path.display()
                )
            })?;
            serde_json::to_value(value).map_err(|error| {
                format!(
                    "failed to normalize Omena TOML config {}: {error}",
                    config_path.display()
                )
            })
        }
        Some("json") => serde_json::from_str(source).map_err(|error| {
            format!(
                "failed to parse Omena JSON config {}: {error}",
                config_path.display()
            )
        }),
        _ => Err(format!(
            "unsupported Omena config extension for {}",
            config_path.display()
        )),
    }
}

fn normalize_legacy_build_document(config_path: &Path, value: &mut Value) {
    if config_path.file_name().and_then(|name| name.to_str()) == Some("omena.toml") {
        return;
    }
    let Some(object) = value.as_object() else {
        return;
    };
    if object.iter().any(|(key, value)| {
        matches!(key.as_str(), "extends" | "overrides")
            || (CONFIG_SECTION_NAMES.contains(&key.as_str()) && value.is_object())
    }) {
        return;
    }
    let legacy = std::mem::replace(value, Value::Object(Map::new()));
    let mut root = Map::new();
    root.insert("build".to_string(), legacy);
    *value = Value::Object(root);
}

fn extract_extends_paths(value: &Value, config_path: &Path) -> Result<Vec<PathBuf>, String> {
    let Some(extends) = value.get("extends") else {
        return Ok(Vec::new());
    };
    match extends {
        Value::String(path) => Ok(vec![PathBuf::from(path)]),
        Value::Array(paths) => paths
            .iter()
            .map(|path| {
                path.as_str().map(PathBuf::from).ok_or_else(|| {
                    format!(
                        "Omena config {} extends entries must be strings",
                        config_path.display()
                    )
                })
            })
            .collect(),
        _ => Err(format!(
            "Omena config {} extends must be a string or string array",
            config_path.display()
        )),
    }
}

fn merge_config_values(target: &mut Value, incoming: Value) {
    match (target, incoming) {
        (Value::Object(target), Value::Object(incoming)) => {
            for (key, value) in incoming {
                match target.get_mut(&key) {
                    Some(existing) => merge_config_values(existing, value),
                    None => {
                        target.insert(key, value);
                    }
                }
            }
        }
        (target, incoming) => *target = incoming,
    }
}

fn apply_matching_overrides(
    value: &mut Value,
    config_path: &Path,
    target_path: &Path,
) -> Result<(), String> {
    let overrides = value
        .get("overrides")
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default();
    let config_dir = config_path.parent().unwrap_or_else(|| Path::new("."));
    let relative_target = target_path.strip_prefix(config_dir).unwrap_or(target_path);

    for (index, override_value) in overrides.into_iter().enumerate() {
        let patterns = override_patterns(&override_value).ok_or_else(|| {
            format!("Omena config override {index} must declare pattern or patterns")
        })?;
        let mut matched = false;
        for pattern in &patterns {
            matched |= glob_matches(pattern, relative_target)?;
        }
        if !matched {
            continue;
        }
        let mut overlay = override_value;
        if let Some(object) = overlay.as_object_mut() {
            object.remove("pattern");
            object.remove("patterns");
        }
        merge_config_values(value, overlay);
    }
    Ok(())
}

fn override_patterns(value: &Value) -> Option<Vec<String>> {
    let patterns = value.get("patterns").or_else(|| value.get("pattern"))?;
    match patterns {
        Value::String(pattern) => Some(vec![pattern.clone()]),
        Value::Array(patterns) => patterns
            .iter()
            .map(|pattern| pattern.as_str().map(ToOwned::to_owned))
            .collect(),
        _ => None,
    }
}

fn glob_matches(pattern: &str, path: &Path) -> Result<bool, String> {
    let normalized = path.to_string_lossy().replace('\\', "/");
    let candidate = if pattern.contains('/') {
        normalized.as_str()
    } else {
        path.file_name()
            .and_then(|name| name.to_str())
            .unwrap_or(normalized.as_str())
    };
    let glob = GlobBuilder::new(pattern)
        .literal_separator(true)
        .build()
        .map_err(|error| format!("invalid Omena config glob `{pattern}`: {error}"))?;
    Ok(glob.compile_matcher().is_match(candidate))
}

fn apply_editorconfig_defaults(
    value: &mut Value,
    target_path: &Path,
    inputs: &mut Vec<ResolutionInput>,
) -> Result<(), String> {
    let mut files = Vec::new();
    let mut current = target_path
        .parent()
        .unwrap_or_else(|| Path::new("."))
        .to_path_buf();
    loop {
        let candidate = current.join(".editorconfig");
        if candidate.is_file() {
            let source = fs::read(&candidate)
                .map_err(|error| format!("failed to read {}: {error}", candidate.display()))?;
            let is_root = editorconfig_declares_root(&source);
            files.push((candidate, source));
            if is_root {
                break;
            }
        }
        if !current.pop() {
            break;
        }
    }
    files.reverse();

    let mut indent_width = None;
    let mut line_width = None;
    for (path, source) in files {
        let source_text = std::str::from_utf8(&source)
            .map_err(|error| format!("{} is not valid UTF-8: {error}", path.display()))?;
        let relative_target = target_path
            .strip_prefix(path.parent().unwrap_or_else(|| Path::new(".")))
            .unwrap_or(target_path);
        let values = parse_editorconfig_values(source_text, relative_target)?;
        if values.indent_width.is_some() {
            indent_width = values.indent_width;
        }
        if values.line_width.is_some() {
            line_width = values.line_width;
        }
        inputs.push(ResolutionInput {
            kind: "editorconfig",
            path,
            content: source,
        });
    }

    if indent_width.is_none() && line_width.is_none() {
        return Ok(());
    }

    let root = value
        .as_object_mut()
        .ok_or_else(|| "Omena config root must be a table".to_string())?;
    let format = root
        .entry("format".to_string())
        .or_insert_with(|| Value::Object(Map::new()))
        .as_object_mut()
        .ok_or_else(|| "Omena config format must be a table".to_string())?;
    if !format.contains_key("indentWidth")
        && let Some(width) = indent_width
    {
        format.insert("indentWidth".to_string(), Value::from(width));
    }
    if !format.contains_key("lineWidth")
        && let Some(width) = line_width
    {
        format.insert("lineWidth".to_string(), Value::from(width));
    }
    Ok(())
}

#[derive(Default)]
struct EditorConfigValues {
    indent_width: Option<u8>,
    line_width: Option<u16>,
}

fn parse_editorconfig_values(source: &str, target: &Path) -> Result<EditorConfigValues, String> {
    let mut values = EditorConfigValues::default();
    let mut active = false;
    for raw_line in source.lines() {
        let line = raw_line.trim();
        if line.is_empty() || line.starts_with('#') || line.starts_with(';') {
            continue;
        }
        if let Some(section) = line
            .strip_prefix('[')
            .and_then(|line| line.strip_suffix(']'))
        {
            active = glob_matches(section, target)?;
            continue;
        }
        if !active {
            continue;
        }
        let Some((key, raw_value)) = line.split_once('=') else {
            continue;
        };
        match key.trim() {
            "indent_size" => {
                values.indent_width = raw_value.trim().parse::<u8>().ok();
            }
            "max_line_length" => {
                values.line_width = raw_value.trim().parse::<u16>().ok();
            }
            _ => {}
        }
    }
    Ok(values)
}

fn editorconfig_declares_root(source: &[u8]) -> bool {
    std::str::from_utf8(source).is_ok_and(|source| {
        source.lines().any(|line| {
            line.split_once('=').is_some_and(|(key, value)| {
                key.trim() == "root" && value.trim().eq_ignore_ascii_case("true")
            })
        })
    })
}

fn interpolate_environment(
    value: &mut Value,
    env_lookup: &dyn Fn(&str) -> Option<String>,
    env_values: &mut BTreeMap<String, String>,
) -> Result<(), String> {
    interpolate_value(value, "", env_lookup, env_values)
}

fn interpolate_value(
    value: &mut Value,
    path: &str,
    env_lookup: &dyn Fn(&str) -> Option<String>,
    env_values: &mut BTreeMap<String, String>,
) -> Result<(), String> {
    match value {
        Value::Object(object) => {
            for (key, value) in object {
                let child = if path.is_empty() {
                    key.clone()
                } else {
                    format!("{path}.{key}")
                };
                interpolate_value(value, &child, env_lookup, env_values)?;
            }
        }
        Value::Array(values) => {
            for value in values {
                interpolate_value(value, &format!("{path}[]"), env_lookup, env_values)?;
            }
        }
        Value::String(source) if source.contains("${") => {
            if !environment_path_allowed(path) {
                return Err(format!(
                    "environment interpolation is not allowed for Omena config key `{path}`"
                ));
            }
            *source = interpolate_string(source, env_lookup, env_values)?;
        }
        _ => {}
    }
    Ok(())
}

fn environment_path_allowed(path: &str) -> bool {
    let path = path.strip_prefix("overrides[].").unwrap_or(path);
    matches!(
        path,
        "extends"
            | "extends[]"
            | "workspace.roots[]"
            | "overrides[].pattern"
            | "overrides[].patterns[]"
            | "build.output"
            | "build.sources[]"
            | "build.packageManifests[]"
            | "build.bundleEntries[]"
            | "build.splitOutDir"
            | "build.contextJson"
            | "build.engineInputJson"
    )
}

fn interpolate_string(
    source: &str,
    env_lookup: &dyn Fn(&str) -> Option<String>,
    env_values: &mut BTreeMap<String, String>,
) -> Result<String, String> {
    let mut output = String::with_capacity(source.len());
    let mut remainder = source;
    while let Some(start) = remainder.find("${") {
        output.push_str(&remainder[..start]);
        let after = &remainder[start + 2..];
        let end = after
            .find('}')
            .ok_or_else(|| format!("unterminated environment reference in `{source}`"))?;
        let name = &after[..end];
        if name.is_empty()
            || !name
                .chars()
                .all(|character| character == '_' || character.is_ascii_alphanumeric())
        {
            return Err(format!("invalid environment variable name `{name}`"));
        }
        let resolved = env_lookup(name)
            .ok_or_else(|| format!("environment variable `{name}` is not defined"))?;
        env_values.insert(name.to_string(), resolved.clone());
        output.push_str(&resolved);
        remainder = &after[end + 1..];
    }
    output.push_str(remainder);
    Ok(output)
}

fn deserialize_config(
    value: &Value,
    config_path: &Path,
) -> Result<(OmenaConfig, Vec<String>), String> {
    let source = serde_json::to_string(value).map_err(|error| {
        format!(
            "failed to normalize Omena config {}: {error}",
            config_path.display()
        )
    })?;
    let mut deserializer = serde_json::Deserializer::from_str(&source);
    let mut unknown = BTreeSet::new();
    let config = serde_ignored::deserialize(&mut deserializer, |path| {
        unknown.insert(path.to_string());
    })
    .map_err(|error| {
        format!(
            "failed to decode Omena config {}: {error}",
            config_path.display()
        )
    })?;
    Ok((config, unknown.into_iter().collect()))
}

fn append_not_yet_consumed_reports(
    value: &Value,
    config: &OmenaConfig,
    reports: &mut Vec<OmenaConfigReport>,
) {
    for section in ["minify", "modules", "sass", "intelligence", "verify", "ci"] {
        if value.get(section).is_some() {
            reports.push(OmenaConfigReport::not_yet_consumed(
                section,
                format!(
                    "the `{section}` section is typed and retained but its product semantics are not wired yet"
                ),
            ));
        }
    }
    if config.verify.translation_validation == OmenaTranslationValidationMode::Staged {
        reports.push(OmenaConfigReport::not_yet_consumed(
            "verify.translationValidation",
            format!(
                "{} translation validation awaits an engine-owned observation-equality report arm",
                config.verify.translation_validation.as_str()
            ),
        ));
    }
}

fn digest_resolution(
    config_path: &Path,
    target_path: &Path,
    value: &Value,
    inputs: &[ResolutionInput],
    env_values: &BTreeMap<String, String>,
) -> Result<String, String> {
    let mut hasher = Sha256::new();
    let digest_root = config_path.parent().unwrap_or_else(|| Path::new("."));
    digest_part(
        &mut hasher,
        "selected-config-path",
        stable_digest_path(digest_root, config_path).as_bytes(),
    );
    digest_part(
        &mut hasher,
        "target-path",
        stable_digest_path(digest_root, target_path).as_bytes(),
    );
    for input in inputs {
        digest_part(
            &mut hasher,
            input.kind,
            stable_digest_path(digest_root, &input.path).as_bytes(),
        );
        digest_part(&mut hasher, "content", &input.content);
    }
    for (name, value) in env_values {
        digest_part(&mut hasher, "environment-name", name.as_bytes());
        digest_part(&mut hasher, "environment-value", value.as_bytes());
    }
    let normalized = serde_json::to_vec(value)
        .map_err(|error| format!("failed to serialize resolved Omena config: {error}"))?;
    digest_part(&mut hasher, "resolved-config", &normalized);
    let mut encoded = String::with_capacity(64);
    const HEX: &[u8; 16] = b"0123456789abcdef";
    for byte in hasher.finalize() {
        encoded.push(HEX[(byte >> 4) as usize] as char);
        encoded.push(HEX[(byte & 0x0f) as usize] as char);
    }
    Ok(encoded)
}

fn stable_digest_path(root: &Path, path: &Path) -> String {
    relative_path(root, path)
        .to_string_lossy()
        .replace('\\', "/")
}

fn relative_path(from: &Path, to: &Path) -> PathBuf {
    let from_components = from.components().collect::<Vec<_>>();
    let to_components = to.components().collect::<Vec<_>>();
    let common = from_components
        .iter()
        .zip(&to_components)
        .take_while(|(left, right)| left == right)
        .count();
    let mut relative = PathBuf::new();
    for _ in &from_components[common..] {
        relative.push("..");
    }
    for component in &to_components[common..] {
        relative.push(component.as_os_str());
    }
    if relative.as_os_str().is_empty() {
        relative.push(".");
    }
    relative
}

fn digest_part(hasher: &mut Sha256, label: &str, bytes: &[u8]) {
    hasher.update((label.len() as u64).to_le_bytes());
    hasher.update(label.as_bytes());
    hasher.update((bytes.len() as u64).to_le_bytes());
    hasher.update(bytes);
}

fn canonical_existing_path(path: &Path) -> Result<PathBuf, String> {
    fs::canonicalize(path).map_err(|error| format!("failed to resolve {}: {error}", path.display()))
}

fn absolute_path(path: &Path) -> Result<PathBuf, String> {
    let absolute = if path.is_absolute() {
        path.to_path_buf()
    } else {
        std::env::current_dir()
            .map(|directory| directory.join(path))
            .map_err(|error| format!("failed to read current directory: {error}"))?
    };
    if absolute.exists() {
        fs::canonicalize(&absolute)
            .map_err(|error| format!("failed to resolve {}: {error}", absolute.display()))
    } else {
        Ok(absolute)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{SystemTime, UNIX_EPOCH};

    #[test]
    fn extends_overrides_editorconfig_and_environment_affect_resolution() -> Result<(), String> {
        let root = temp_dir("resolution");
        let source_dir = root.join("src");
        fs::create_dir_all(&source_dir).map_err(|error| error.to_string())?;
        fs::write(
            root.join("base.toml"),
            "[format]\nlineWidth = 80\n[build]\noutput = \"base.css\"\n",
        )
        .map_err(|error| error.to_string())?;
        fs::write(
            root.join("omena.toml"),
            "extends = \"./base.toml\"\n[format]\nmode = \"stable\"\n[build]\noutput = \"${OUT_DIR}/app.css\"\n[[overrides]]\npattern = \"*.scss\"\n[overrides.format]\nlineWidth = 120\n",
        )
        .map_err(|error| error.to_string())?;
        fs::write(
            root.join(".editorconfig"),
            "root = true\n[*]\nindent_size = 4\nmax_line_length = 90\n",
        )
        .map_err(|error| error.to_string())?;
        let target = source_dir.join("app.scss");
        fs::write(&target, ".a {}\n").map_err(|error| error.to_string())?;

        let loaded =
            resolve_config_document_with_env(&root.join("omena.toml"), &target, &|name| {
                (name == "OUT_DIR").then(|| "dist".to_string())
            })?;
        assert_eq!(loaded.config.format.mode.as_deref(), Some("stable"));
        assert_eq!(loaded.config.format.line_width, Some(120));
        assert_eq!(loaded.config.format.indent_width, Some(4));
        assert_eq!(
            loaded.config.build.output,
            Some(PathBuf::from("dist/app.css"))
        );
        assert_eq!(loaded.config_content_digest.len(), 64);
        cleanup(&root);
        Ok(())
    }

    #[test]
    fn extends_cycle_and_undefined_environment_fail_closed() -> Result<(), String> {
        let root = temp_dir("fail-closed");
        fs::create_dir_all(&root).map_err(|error| error.to_string())?;
        let target = root.join("a.css");
        fs::write(&target, ".a {}\n").map_err(|error| error.to_string())?;
        fs::write(root.join("a.toml"), "extends = \"b.toml\"\n")
            .map_err(|error| error.to_string())?;
        fs::write(root.join("b.toml"), "extends = \"a.toml\"\n")
            .map_err(|error| error.to_string())?;
        let cycle = resolve_config_document_with_env(&root.join("a.toml"), &target, &|_| None);
        assert!(cycle.is_err_and(|error| error.contains("extends cycle")));

        fs::write(
            root.join("omena.toml"),
            "[build]\noutput = \"${MISSING}/app.css\"\n",
        )
        .map_err(|error| error.to_string())?;
        let missing =
            resolve_config_document_with_env(&root.join("omena.toml"), &target, &|_| None);
        assert!(
            missing.is_err_and(|error| error.contains("MISSING") && error.contains("not defined"))
        );

        fs::write(
            root.join("omena.toml"),
            "extends = \"https://example.invalid/base.toml\"\n",
        )
        .map_err(|error| error.to_string())?;
        let remote = resolve_config_document_with_env(&root.join("omena.toml"), &target, &|_| None);
        assert!(remote.is_err_and(|error| error.contains("remote Omena config extends")));
        cleanup(&root);
        Ok(())
    }

    #[test]
    fn unknown_and_unconsumed_fields_are_reported() -> Result<(), String> {
        let root = temp_dir("reports");
        fs::create_dir_all(&root).map_err(|error| error.to_string())?;
        let target = root.join("a.css");
        fs::write(&target, ".a {}\n").map_err(|error| error.to_string())?;
        fs::write(
            root.join("omena.toml"),
            "[lint]\nprofile = \"recommended\"\nprofileTypo = true\n[verify]\ntranslationValidation = \"staged\"\n",
        )
        .map_err(|error| error.to_string())?;
        let loaded =
            resolve_config_document_with_env(&root.join("omena.toml"), &target, &|_| None)?;
        assert!(loaded.reports.iter().any(|report| {
            report.kind.as_str() == "unknownKey" && report.path.contains("profileTypo")
        }));
        assert!(
            !loaded.reports.iter().any(|report| {
                report.kind.as_str() == "notYetConsumed" && report.path == "lint"
            })
        );
        assert!(loaded.reports.iter().any(|report| {
            report.kind.as_str() == "notYetConsumed"
                && report.path == "verify.translationValidation"
        }));
        assert!(
            !loaded.reports.iter().any(|report| {
                report.kind.as_str() == "notYetConsumed" && report.path == "format"
            })
        );
        cleanup(&root);
        Ok(())
    }

    #[test]
    fn complete_product_schema_is_typed_without_unknown_fields() -> Result<(), String> {
        let root = temp_dir("complete-schema");
        fs::create_dir_all(&root).map_err(|error| error.to_string())?;
        let target = root.join("a.css");
        fs::write(&target, ".a {}\n").map_err(|error| error.to_string())?;
        fs::write(
            root.join("omena.toml"),
            r#"
[workspace]
roots = ["packages/*"]
[style]
languages = ["css", "scss", "sass", "less"]
sourceLanguages = ["typescriptreact", "vue", "svelte", "astro"]
[lint]
profile = "recommended"
stylelintCompat = true
[format]
mode = "stable"
lineWidth = 100
[minify]
profile = "semantic"
target = "last 2 versions"
[modules]
typedDefinitions = true
hashStrategy = "stable"
include = ["src/**/*.module.css", "src/**/*.module.scss"]
declarationDir = "generated/types"
interfaceFile = "generated/modules.json"
[sass]
oracle = "dart-sass"
sif = true
[intelligence.tailwind]
enabled = true
classFunctions = ["clsx", "cva", "tw"]
[verify]
evidence = "required"
translationValidation = "staged"
externalCorpus = "advisory"
[ci]
precisionRegression = "warn"
transformRejection = "error"
"#,
        )
        .map_err(|error| error.to_string())?;

        let loaded =
            resolve_config_document_with_env(&root.join("omena.toml"), &target, &|_| None)?;
        assert_eq!(loaded.config.workspace.roots, ["packages/*"]);
        assert_eq!(loaded.config.style.languages.len(), 4);
        assert_eq!(loaded.config.lint.profile.as_deref(), Some("recommended"));
        assert_eq!(loaded.config.format.line_width, Some(100));
        assert_eq!(loaded.config.minify.profile.as_deref(), Some("semantic"));
        assert_eq!(loaded.config.modules.typed_definitions, Some(true));
        assert_eq!(loaded.config.modules.include.len(), 2);
        assert_eq!(
            loaded.config.modules.declaration_dir.as_deref(),
            Some(std::path::Path::new("generated/types"))
        );
        assert_eq!(
            loaded.config.modules.interface_file.as_deref(),
            Some(std::path::Path::new("generated/modules.json"))
        );
        assert_eq!(loaded.config.sass.oracle.as_deref(), Some("dart-sass"));
        assert_eq!(loaded.config.intelligence.tailwind.class_functions.len(), 3);
        assert_eq!(
            loaded.config.ci.transform_rejection.as_deref(),
            Some("error")
        );
        assert!(
            !loaded
                .reports
                .iter()
                .any(|report| report.kind.as_str() == "unknownKey")
        );
        cleanup(&root);
        Ok(())
    }

    #[test]
    fn digest_changes_with_every_resolution_input_class() -> Result<(), String> {
        let root = temp_dir("digest-inputs");
        fs::create_dir_all(&root).map_err(|error| error.to_string())?;
        let target = root.join("a.css");
        fs::write(&target, ".a {}\n").map_err(|error| error.to_string())?;
        fs::write(root.join("base.toml"), "[format]\nlineWidth = 80\n")
            .map_err(|error| error.to_string())?;
        fs::write(
            root.join("omena.toml"),
            "extends = \"base.toml\"\n[build]\noutput = \"${OUT_DIR}/a.css\"\n",
        )
        .map_err(|error| error.to_string())?;
        fs::write(root.join(".editorconfig"), "[*]\nindent_size = 2\n")
            .map_err(|error| error.to_string())?;

        let first = resolve_config_document_with_env(&root.join("omena.toml"), &target, &|_| {
            Some("dist-a".to_string())
        })?;
        let env_changed =
            resolve_config_document_with_env(&root.join("omena.toml"), &target, &|_| {
                Some("dist-b".to_string())
            })?;
        assert_ne!(
            first.config_content_digest,
            env_changed.config_content_digest
        );

        fs::write(root.join("base.toml"), "[format]\nlineWidth = 81\n")
            .map_err(|error| error.to_string())?;
        let extends_changed =
            resolve_config_document_with_env(&root.join("omena.toml"), &target, &|_| {
                Some("dist-a".to_string())
            })?;
        assert_ne!(
            first.config_content_digest,
            extends_changed.config_content_digest
        );

        fs::write(root.join(".editorconfig"), "[*]\nindent_size = 4\n")
            .map_err(|error| error.to_string())?;
        let editorconfig_changed =
            resolve_config_document_with_env(&root.join("omena.toml"), &target, &|_| {
                Some("dist-a".to_string())
            })?;
        assert_ne!(
            extends_changed.config_content_digest,
            editorconfig_changed.config_content_digest
        );
        cleanup(&root);
        Ok(())
    }

    #[test]
    fn digest_is_stable_across_checkout_locations() -> Result<(), String> {
        let parent = temp_dir("relocatable-digest");
        let first_root = parent.join("checkout-a");
        let second_root = parent.join("checkout-b");
        for root in [&first_root, &second_root] {
            fs::create_dir_all(root.join("src")).map_err(|error| error.to_string())?;
            fs::write(root.join("base.toml"), "[format]\nlineWidth = 100\n")
                .map_err(|error| error.to_string())?;
            fs::write(
                root.join("omena.toml"),
                "extends = \"base.toml\"\n[build]\noutput = \"dist/app.css\"\n",
            )
            .map_err(|error| error.to_string())?;
            fs::write(root.join("src/app.css"), ".a {}\n").map_err(|error| error.to_string())?;
        }
        let first = resolve_config_document_with_env(
            &first_root.join("omena.toml"),
            &first_root.join("src/app.css"),
            &|_| None,
        )?;
        let second = resolve_config_document_with_env(
            &second_root.join("omena.toml"),
            &second_root.join("src/app.css"),
            &|_| None,
        )?;
        assert_eq!(first.config_content_digest, second.config_content_digest);
        cleanup(&parent);
        Ok(())
    }

    #[test]
    fn invalid_override_glob_fails_closed() -> Result<(), String> {
        let root = temp_dir("invalid-glob");
        fs::create_dir_all(&root).map_err(|error| error.to_string())?;
        let target = root.join("a.css");
        fs::write(&target, ".a {}\n").map_err(|error| error.to_string())?;
        fs::write(
            root.join("omena.toml"),
            "[[overrides]]\npattern = \"[\"\n[overrides.format]\nlineWidth = 100\n",
        )
        .map_err(|error| error.to_string())?;
        let result = resolve_config_document_with_env(&root.join("omena.toml"), &target, &|_| None);
        assert!(result.is_err_and(|error| error.contains("invalid Omena config glob")));
        cleanup(&root);
        Ok(())
    }

    fn temp_dir(label: &str) -> PathBuf {
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_or(0, |duration| duration.as_nanos());
        std::env::temp_dir().join(format!("omena-config-{label}-{nonce}"))
    }

    fn cleanup(path: &Path) {
        let _ = fs::remove_dir_all(path);
    }
}

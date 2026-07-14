use std::{
    collections::VecDeque,
    fs,
    path::{Path, PathBuf},
};

use omena_abstract_value::FactPrecision;
use omena_parser::ParserByteSpanV0;
use serde::Serialize;

use crate::{
    SourceClassValueUniverseEntryV0, SourceClassValueUnresolvedV0,
    SourceDomainClassReferenceFactV0, SourceSyntaxIndexV0,
};

mod config;

const UTILITY_PROVIDER_ID: &str = "tailwind-uno-utility-domain";
const UTILITY_OWNER_NAME: &str = "workspace";
const UTILITY_REFERENCE_DOMAIN: &str = "utility-classes";
const DISCOVERY_DIR_LIMIT: usize = 256;
const DISCOVERY_MAX_DEPTH: usize = 3;
const CONFIG_NAMES: &[&str] = &[
    "tailwind.config.js",
    "tailwind.config.ts",
    "tailwind.config.cjs",
    "tailwind.config.mjs",
    "tailwind.config.cts",
    "tailwind.config.mts",
    "uno.config.js",
    "uno.config.ts",
    "uno.config.cjs",
    "uno.config.mjs",
    "uno.config.cts",
    "uno.config.mts",
];

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum UtilityConfigKindV0 {
    Tailwind,
    UnoCss,
}

impl UtilityConfigKindV0 {
    pub(crate) const fn domain(self) -> &'static str {
        match self {
            Self::Tailwind => "tailwind-utilities",
            Self::UnoCss => "unocss-utilities",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UtilityClassIntelligenceReportV0 {
    pub schema_version: &'static str,
    pub product: &'static str,
    pub config_paths: Vec<String>,
    pub class_value_universes: Vec<SourceClassValueUniverseEntryV0>,
    pub precision: FactPrecision,
}

impl Default for UtilityClassIntelligenceReportV0 {
    fn default() -> Self {
        Self {
            schema_version: "0",
            product: "omena-bridge.utility-class-intelligence",
            config_paths: Vec::new(),
            class_value_universes: Vec::new(),
            precision: FactPrecision::Unknown,
        }
    }
}

impl UtilityClassIntelligenceReportV0 {
    pub fn enumerated_class_count(&self) -> usize {
        self.class_value_universes
            .iter()
            .map(|entry| entry.class_names.len())
            .sum()
    }

    pub fn pattern_count(&self) -> usize {
        self.class_value_universes
            .iter()
            .map(|entry| entry.patterns.len())
            .sum()
    }

    pub fn unresolved_count(&self) -> usize {
        self.class_value_universes
            .iter()
            .map(|entry| entry.unresolved.len())
            .sum()
    }

    pub fn unresolved(&self) -> impl Iterator<Item = &SourceClassValueUnresolvedV0> {
        self.class_value_universes
            .iter()
            .flat_map(|entry| entry.unresolved.iter())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum UtilityClassMembershipKindV0 {
    Enumerated,
    Pattern,
    Outside,
    Indeterminate,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UtilityClassLintSignalV0 {
    pub class_name: String,
    pub membership: UtilityClassMembershipKindV0,
    pub undefined_class_signal: bool,
    pub matched_pattern: Option<String>,
    pub unresolved_count: usize,
}

pub fn summarize_omena_bridge_utility_class_intelligence_for_config(
    config_path: &Path,
    config_source: &str,
) -> UtilityClassIntelligenceReportV0 {
    let kind = config_kind_from_path(config_path);
    let universe = config::summarize_config(config_path, config_source, kind);
    UtilityClassIntelligenceReportV0 {
        config_paths: vec![config_path.to_string_lossy().to_string()],
        class_value_universes: vec![universe],
        ..UtilityClassIntelligenceReportV0::default()
    }
}

pub fn load_omena_bridge_workspace_utility_class_intelligence(
    workspace_root: &Path,
    explicit_config_path: Option<&Path>,
) -> UtilityClassIntelligenceReportV0 {
    let declared_settings = declared_utility_config_settings(workspace_root);
    if explicit_config_path.is_none() && declared_settings.enabled == Some(false) {
        return UtilityClassIntelligenceReportV0::default();
    }
    let declared_config = explicit_config_path
        .map(Path::to_path_buf)
        .or(declared_settings.config_path);
    let config_paths = declared_config.map_or_else(
        || discover_utility_config_paths(workspace_root),
        |path| {
            let path = if path.is_absolute() {
                path
            } else {
                workspace_root.join(path)
            };
            vec![path]
        },
    );

    let mut report = UtilityClassIntelligenceReportV0::default();
    for config_path in config_paths {
        let config_path_text = config_path.to_string_lossy().to_string();
        report.config_paths.push(config_path_text.clone());
        match fs::read_to_string(config_path.as_path()) {
            Ok(source) => report.class_value_universes.push(config::summarize_config(
                config_path.as_path(),
                source.as_str(),
                config_kind_from_path(config_path.as_path()),
            )),
            Err(error) => report
                .class_value_universes
                .push(unreadable_config_universe(
                    config_path_text,
                    error.to_string(),
                )),
        }
    }
    report.config_paths.sort();
    report.config_paths.dedup();
    report
}

pub fn append_omena_bridge_utility_class_intelligence(
    index: &mut SourceSyntaxIndexV0,
    source: &str,
    report: &UtilityClassIntelligenceReportV0,
) {
    index
        .class_value_universes
        .retain(|entry| entry.plugin_id != UTILITY_PROVIDER_ID);
    index
        .domain_class_references
        .retain(|reference| reference.plugin_id != UTILITY_PROVIDER_ID);
    index
        .class_value_universes
        .extend(report.class_value_universes.iter().cloned());

    if report.class_value_universes.is_empty() {
        return;
    }
    for literal_span in index.class_string_literals.clone() {
        let Some(literal) = source.get(literal_span.start..literal_span.end) else {
            continue;
        };
        for (relative_start, relative_end, class_name) in class_tokens(literal) {
            index
                .domain_class_references
                .push(SourceDomainClassReferenceFactV0 {
                    byte_span: ParserByteSpanV0 {
                        start: literal_span.start + relative_start,
                        end: literal_span.start + relative_end,
                    },
                    plugin_id: UTILITY_PROVIDER_ID,
                    domain: UTILITY_REFERENCE_DOMAIN,
                    owner_name: UTILITY_OWNER_NAME.to_string(),
                    axis_name: "class".to_string(),
                    option_name: Some(class_name.to_string()),
                    prefix: None,
                });
        }
    }
    index.domain_class_references.sort_by_key(|reference| {
        (
            reference.byte_span.start,
            reference.byte_span.end,
            reference.plugin_id,
            reference.option_name.clone(),
        )
    });
    index.domain_class_references.dedup();
}

pub fn classify_omena_bridge_utility_class(
    report: &UtilityClassIntelligenceReportV0,
    class_name: &str,
) -> UtilityClassLintSignalV0 {
    let exact = report.class_value_universes.iter().any(|entry| {
        entry
            .class_names
            .iter()
            .any(|candidate| candidate == class_name)
    });
    if exact {
        return utility_signal(
            class_name,
            UtilityClassMembershipKindV0::Enumerated,
            None,
            report,
        );
    }

    let mut has_unresolved_pattern = false;
    for pattern in report
        .class_value_universes
        .iter()
        .flat_map(|entry| entry.patterns.iter())
    {
        match pattern.matches(class_name) {
            Some(true) => {
                return utility_signal(
                    class_name,
                    UtilityClassMembershipKindV0::Pattern,
                    Some(pattern.source.clone()),
                    report,
                );
            }
            Some(false) => {}
            None => has_unresolved_pattern = true,
        }
    }

    let unresolved_count = report.unresolved_count();
    let membership = if unresolved_count > 0 || has_unresolved_pattern {
        UtilityClassMembershipKindV0::Indeterminate
    } else {
        UtilityClassMembershipKindV0::Outside
    };
    utility_signal(class_name, membership, None, report)
}

fn utility_signal(
    class_name: &str,
    membership: UtilityClassMembershipKindV0,
    matched_pattern: Option<String>,
    report: &UtilityClassIntelligenceReportV0,
) -> UtilityClassLintSignalV0 {
    UtilityClassLintSignalV0 {
        class_name: class_name.to_string(),
        membership,
        undefined_class_signal: membership == UtilityClassMembershipKindV0::Outside,
        matched_pattern,
        unresolved_count: report.unresolved_count(),
    }
}

fn unreadable_config_universe(
    config_path: String,
    detail: String,
) -> SourceClassValueUniverseEntryV0 {
    SourceClassValueUniverseEntryV0 {
        plugin_id: UTILITY_PROVIDER_ID,
        domain: config_kind_from_path(Path::new(config_path.as_str())).domain(),
        owner_name: UTILITY_OWNER_NAME.to_string(),
        class_names: Vec::new(),
        axes: Vec::new(),
        patterns: Vec::new(),
        unresolved: vec![SourceClassValueUnresolvedV0 {
            path: config_path,
            reason: "config-read-failed".to_string(),
            detail,
        }],
        byte_span: ParserByteSpanV0 { start: 0, end: 0 },
    }
}

fn config_kind_from_path(path: &Path) -> UtilityConfigKindV0 {
    let name = path
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or_default();
    if name.starts_with("uno.") || name.starts_with("unocss.") {
        UtilityConfigKindV0::UnoCss
    } else {
        UtilityConfigKindV0::Tailwind
    }
}

fn discover_utility_config_paths(workspace_root: &Path) -> Vec<PathBuf> {
    let mut paths = Vec::new();
    let mut queue = VecDeque::from([(workspace_root.to_path_buf(), 0usize)]);
    let mut visited = 0usize;
    while let Some((directory, depth)) = queue.pop_front() {
        if visited >= DISCOVERY_DIR_LIMIT {
            break;
        }
        visited += 1;
        for name in CONFIG_NAMES {
            let candidate = directory.join(name);
            if candidate.is_file() {
                paths.push(candidate);
            }
        }
        if depth >= DISCOVERY_MAX_DEPTH {
            continue;
        }
        let Ok(entries) = fs::read_dir(directory) else {
            continue;
        };
        for entry in entries.flatten() {
            let path = entry.path();
            let Some(name) = path.file_name().and_then(|name| name.to_str()) else {
                continue;
            };
            if path.is_dir() && !should_skip_discovery_directory(name) {
                queue.push_back((path, depth + 1));
            }
        }
    }
    paths.sort();
    paths.dedup();
    paths
}

fn should_skip_discovery_directory(name: &str) -> bool {
    name.starts_with('.')
        || matches!(
            name,
            "coverage" | "dist" | "node_modules" | "target" | "vendor"
        )
}

#[derive(Default)]
struct DeclaredUtilityConfigSettings {
    enabled: Option<bool>,
    config_path: Option<PathBuf>,
}

fn declared_utility_config_settings(workspace_root: &Path) -> DeclaredUtilityConfigSettings {
    for name in ["omena.toml", "omena.config.toml", "omena.config.json"] {
        let path = workspace_root.join(name);
        let Ok(source) = fs::read_to_string(path.as_path()) else {
            continue;
        };
        let value = if name.ends_with(".json") {
            serde_json::from_str::<serde_json::Value>(source.as_str()).ok()
        } else {
            toml::from_str::<toml::Value>(source.as_str())
                .ok()
                .and_then(|value| serde_json::to_value(value).ok())
        };
        let Some(value) = value else {
            continue;
        };
        let tailwind = value.pointer("/intelligence/tailwind");
        return DeclaredUtilityConfigSettings {
            enabled: tailwind
                .and_then(|value| value.get("enabled"))
                .and_then(serde_json::Value::as_bool),
            config_path: tailwind
                .and_then(|value| value.get("configPath"))
                .and_then(serde_json::Value::as_str)
                .map(PathBuf::from),
        };
    }
    DeclaredUtilityConfigSettings::default()
}

fn class_tokens(source: &str) -> Vec<(usize, usize, &str)> {
    let mut tokens = Vec::new();
    let mut start = None;
    for (offset, character) in source.char_indices() {
        if character.is_whitespace() {
            if let Some(token_start) = start.take() {
                tokens.push((token_start, offset, &source[token_start..offset]));
            }
        } else if start.is_none() {
            start = Some(offset);
        }
    }
    if let Some(token_start) = start {
        tokens.push((token_start, source.len(), &source[token_start..]));
    }
    tokens
}

#[cfg(test)]
mod tests;

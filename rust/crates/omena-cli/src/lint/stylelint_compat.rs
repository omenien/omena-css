use std::{collections::BTreeSet, fs, path::Path};

use omena_checker::is_omena_checker_rule_code;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::paths::path_string;

const STYLELINT_COMPAT_CENSUS: &str = include_str!("../../stylelint-compat-census.json");

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
struct StylelintCompatibilityCensusV0 {
    schema_version: String,
    product: String,
    plugin_package: String,
    plugin_peer_range: String,
    mappings: Vec<StylelintRuleMappingV0>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
struct StylelintRuleMappingV0 {
    stylelint_rule: String,
    omena_rule: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct StylelintMappedRuleV0 {
    pub(crate) stylelint_rule: String,
    pub(crate) omena_rule: String,
    pub(crate) enabled: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct StylelintUnsupportedRuleV0 {
    pub(crate) stylelint_rule: String,
    pub(crate) enabled: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct StylelintCompatibilityReportV0 {
    pub(crate) schema_version: &'static str,
    pub(crate) product: &'static str,
    pub(crate) config_path: String,
    pub(crate) mapped_rule_count: usize,
    pub(crate) enabled_mapped_rule_count: usize,
    pub(crate) unsupported_rule_count: usize,
    pub(crate) mapped_rules: Vec<StylelintMappedRuleV0>,
    pub(crate) unsupported_rules: Vec<StylelintUnsupportedRuleV0>,
}

impl StylelintCompatibilityReportV0 {
    pub(crate) fn enabled_omena_rule_ids(&self) -> BTreeSet<&str> {
        self.mapped_rules
            .iter()
            .filter(|mapping| mapping.enabled)
            .map(|mapping| mapping.omena_rule.as_str())
            .collect()
    }
}

pub(crate) fn read_stylelint_compatibility_report(
    path: &Path,
) -> Result<StylelintCompatibilityReportV0, String> {
    validate_stylelint_compatibility_census()?;
    let source = fs::read_to_string(path)
        .map_err(|error| format!("failed to read {}: {error}", path_string(path)))?;
    let document = parse_stylelint_config(path, source.as_str())?;
    let rules = document
        .get("rules")
        .and_then(Value::as_object)
        .ok_or_else(|| {
            format!(
                "Stylelint config {} must contain a rules object",
                path_string(path)
            )
        })?;
    let census = read_census()?;
    let mapping_by_stylelint = census
        .mappings
        .iter()
        .map(|mapping| (mapping.stylelint_rule.as_str(), mapping.omena_rule.as_str()))
        .collect::<std::collections::BTreeMap<_, _>>();
    let mut mapped_rules = Vec::new();
    let mut unsupported_rules = Vec::new();

    for (stylelint_rule, configuration) in rules {
        let enabled = stylelint_rule_enabled(configuration);
        if let Some(omena_rule) = mapping_by_stylelint.get(stylelint_rule.as_str()) {
            mapped_rules.push(StylelintMappedRuleV0 {
                stylelint_rule: stylelint_rule.clone(),
                omena_rule: (*omena_rule).to_string(),
                enabled,
            });
        } else {
            unsupported_rules.push(StylelintUnsupportedRuleV0 {
                stylelint_rule: stylelint_rule.clone(),
                enabled,
            });
        }
    }
    mapped_rules.sort_by(|left, right| left.stylelint_rule.cmp(&right.stylelint_rule));
    unsupported_rules.sort_by(|left, right| left.stylelint_rule.cmp(&right.stylelint_rule));

    Ok(StylelintCompatibilityReportV0 {
        schema_version: "0",
        product: "omena-cli.stylelint-compatibility-report",
        config_path: path_string(path),
        mapped_rule_count: mapped_rules.len(),
        enabled_mapped_rule_count: mapped_rules
            .iter()
            .filter(|mapping| mapping.enabled)
            .count(),
        unsupported_rule_count: unsupported_rules.len(),
        mapped_rules,
        unsupported_rules,
    })
}

pub(crate) fn validate_stylelint_compatibility_census() -> Result<usize, String> {
    let census = read_census()?;
    if census.schema_version != "0"
        || census.product != "omena-cli.stylelint-compat-census"
        || census.plugin_package != "@omena/stylelint-plugin"
        || census.plugin_peer_range != "^17.0.0"
    {
        return Err("Stylelint compatibility census metadata is invalid".to_string());
    }
    let mut stylelint_rules = BTreeSet::new();
    let mut omena_rules = BTreeSet::new();
    for mapping in &census.mappings {
        if !stylelint_rules.insert(mapping.stylelint_rule.as_str()) {
            return Err(format!(
                "duplicate Stylelint compatibility rule {}",
                mapping.stylelint_rule
            ));
        }
        if !omena_rules.insert(mapping.omena_rule.as_str()) {
            return Err(format!(
                "duplicate Omena compatibility rule {}",
                mapping.omena_rule
            ));
        }
        if !is_omena_checker_rule_code(mapping.omena_rule.as_str()) {
            return Err(format!(
                "Stylelint mapping references unknown Omena rule {}",
                mapping.omena_rule
            ));
        }
    }
    Ok(census.mappings.len())
}

fn read_census() -> Result<StylelintCompatibilityCensusV0, String> {
    serde_json::from_str(STYLELINT_COMPAT_CENSUS)
        .map_err(|error| format!("failed to parse Stylelint compatibility census: {error}"))
}

fn parse_stylelint_config(path: &Path, source: &str) -> Result<Value, String> {
    let extension = path.extension().and_then(|value| value.to_str());
    match extension {
        Some("yaml" | "yml") => serde_yaml_ng::from_str(source).map_err(|error| {
            format!("failed to parse Stylelint YAML {}: {error}", path_string(path))
        }),
        _ => serde_json::from_str(source).or_else(|json_error| {
            serde_yaml_ng::from_str(source).map_err(|yaml_error| {
                format!(
                    "failed to parse Stylelint config {} as JSON ({json_error}) or YAML ({yaml_error})",
                    path_string(path)
                )
            })
        }),
    }
}

fn stylelint_rule_enabled(configuration: &Value) -> bool {
    let primary = configuration
        .as_array()
        .and_then(|items| items.first())
        .unwrap_or(configuration);
    match primary {
        Value::Null => false,
        Value::Bool(value) => *value,
        Value::Number(value) => value.as_i64() != Some(0),
        Value::String(value) => value != "off",
        Value::Array(_) | Value::Object(_) => true,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::{
        path::PathBuf,
        sync::atomic::{AtomicU64, Ordering},
    };

    static NEXT_FIXTURE_ID: AtomicU64 = AtomicU64::new(0);

    #[test]
    fn census_maps_the_eight_plugin_rules_to_registered_checker_rules() -> Result<(), String> {
        assert_eq!(validate_stylelint_compatibility_census()?, 8);
        Ok(())
    }

    #[test]
    fn json_reports_mapped_disabled_and_unsupported_rules() -> Result<(), String> {
        let path = fixture_path("stylelintrc.json");
        fs::write(
            &path,
            r#"{"rules":{"omena/unused-selector":[true],"omena/missing-keyframes":"off","color-no-invalid-hex":true}}"#,
        )
        .map_err(|error| error.to_string())?;
        let report = read_stylelint_compatibility_report(&path)?;
        assert_eq!(report.mapped_rule_count, 2);
        assert_eq!(report.enabled_mapped_rule_count, 1);
        assert_eq!(report.unsupported_rule_count, 1);
        assert_eq!(
            report.unsupported_rules[0].stylelint_rule,
            "color-no-invalid-hex"
        );
        fs::remove_file(path).map_err(|error| error.to_string())?;
        Ok(())
    }

    #[test]
    fn yaml_uses_the_same_mapping_and_passthrough_contract() -> Result<(), String> {
        let path = fixture_path("stylelintrc.yaml");
        fs::write(
            &path,
            "rules:\n  omena/missing-sass-symbol: true\n  declaration-no-important: true\n",
        )
        .map_err(|error| error.to_string())?;
        let report = read_stylelint_compatibility_report(&path)?;
        assert_eq!(
            report.enabled_omena_rule_ids(),
            BTreeSet::from(["missing-sass-symbol"])
        );
        assert_eq!(
            report.unsupported_rules[0].stylelint_rule,
            "declaration-no-important"
        );
        fs::remove_file(path).map_err(|error| error.to_string())?;
        Ok(())
    }

    fn fixture_path(file_name: &str) -> PathBuf {
        let id = NEXT_FIXTURE_ID.fetch_add(1, Ordering::Relaxed);
        std::env::temp_dir().join(format!(
            "omena-stylelint-compat-{}-{id}-{file_name}",
            std::process::id()
        ))
    }
}

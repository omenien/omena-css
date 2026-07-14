use std::{path::PathBuf, sync::Arc};

use omena_query::{
    OmenaQuerySourceClassValueUnresolvedV0, load_omena_query_workspace_utility_class_intelligence,
};
use serde::Serialize;

use crate::{
    config::{find_omena_config_for_path, resolve_config_path},
    output::{CliOutputMetadataV0, print_json},
};

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct IntelReportV0 {
    pub(crate) provider_id: &'static str,
    pub(crate) enabled: bool,
    pub(crate) workspace_root: String,
    pub(crate) config_paths: Vec<String>,
    pub(crate) enumerated_class_count: usize,
    pub(crate) pattern_count: usize,
    pub(crate) unresolved_count: usize,
    pub(crate) unresolved: Vec<OmenaQuerySourceClassValueUnresolvedV0>,
    pub(crate) config_diagnostics: Vec<String>,
}

pub(crate) fn intel_workspace(root: Option<PathBuf>, json: bool) -> Result<(), String> {
    let root = root.map_or_else(
        || std::env::current_dir().map_err(|error| error.to_string()),
        Ok,
    )?;
    let (report, config_digest) = build_intel_report(root)?;
    if json {
        print_json(
            CliOutputMetadataV0::new("omena-cli.intel")
                .with_config_content_digest(config_digest.as_deref()),
            &report,
        )?;
        return Ok(());
    }

    println!("provider: {}", report.provider_id);
    println!("enabled: {}", report.enabled);
    println!("configs: {}", report.config_paths.len());
    println!("enumerated classes: {}", report.enumerated_class_count);
    println!("patterns: {}", report.pattern_count);
    println!("unresolved: {}", report.unresolved_count);
    for item in &report.unresolved {
        println!("- [{}] {}: {}", item.reason, item.path, item.detail);
    }
    Ok(())
}

fn build_intel_report(root: PathBuf) -> Result<(IntelReportV0, Option<Arc<str>>), String> {
    let loaded = find_omena_config_for_path(root.as_path())?;
    let enabled = loaded
        .as_ref()
        .and_then(|config| config.config.intelligence.tailwind.enabled)
        .unwrap_or(true);
    let explicit_config_path = loaded.as_ref().and_then(|config| {
        config
            .config
            .intelligence
            .tailwind
            .config_path
            .as_deref()
            .map(|path| resolve_config_path(config.directory.as_path(), path))
    });
    let config_diagnostics = loaded
        .as_ref()
        .map(|config| {
            config
                .reports
                .iter()
                .map(|report| report.render_warning())
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();
    let config_digest = loaded
        .as_ref()
        .map(|config| Arc::clone(&config.config_content_digest));
    let utility = if enabled {
        load_omena_query_workspace_utility_class_intelligence(
            root.as_path(),
            explicit_config_path.as_deref(),
        )
    } else {
        Default::default()
    };
    let enumerated_class_count = utility.enumerated_class_count();
    let pattern_count = utility.pattern_count();
    let unresolved = utility.unresolved().cloned().collect::<Vec<_>>();
    Ok((
        IntelReportV0 {
            provider_id: "tailwind-uno-utility-domain",
            enabled,
            workspace_root: root.to_string_lossy().to_string(),
            config_paths: utility.config_paths,
            enumerated_class_count,
            pattern_count,
            unresolved_count: unresolved.len(),
            unresolved,
            config_diagnostics,
        },
        config_digest,
    ))
}

#[cfg(test)]
mod tests {
    use std::{fs, time::SystemTime};

    use super::*;

    #[test]
    fn configured_path_wins_and_summary_retains_all_three_counts() -> Result<(), String> {
        let suffix = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .map_err(|error| error.to_string())?
            .as_nanos();
        let root = std::env::temp_dir().join(format!("omena-intel-{suffix}"));
        fs::create_dir_all(root.join("config")).map_err(|error| error.to_string())?;
        fs::write(
            root.join("omena.toml"),
            "[intelligence.tailwind]\nenabled = true\nconfigPath = 'config/utility.ts'\n",
        )
        .map_err(|error| error.to_string())?;
        fs::write(
            root.join("tailwind.config.ts"),
            "export default { safelist: ['discovered'] }",
        )
        .map_err(|error| error.to_string())?;
        fs::write(
            root.join("config/utility.ts"),
            "export default { safelist: ['explicit'], plugins: [plugin] }",
        )
        .map_err(|error| error.to_string())?;

        let (report, digest) = build_intel_report(root.clone())?;
        assert!(report.enabled);
        assert_eq!(
            report.config_paths,
            vec![
                fs::canonicalize(root.join("config/utility.ts"))
                    .map_err(|error| error.to_string())?
                    .display()
                    .to_string()
            ]
        );
        assert_eq!(report.enumerated_class_count, 1);
        assert_eq!(report.pattern_count, 0);
        assert!(report.unresolved_count >= 2);
        assert!(digest.is_some());
        fs::remove_dir_all(root).map_err(|error| error.to_string())?;
        Ok(())
    }

    #[test]
    fn disabled_provider_does_not_discover_config() -> Result<(), String> {
        let suffix = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .map_err(|error| error.to_string())?
            .as_nanos();
        let root = std::env::temp_dir().join(format!("omena-intel-disabled-{suffix}"));
        fs::create_dir_all(root.as_path()).map_err(|error| error.to_string())?;
        fs::write(
            root.join("omena.toml"),
            "[intelligence.tailwind]\nenabled = false\n",
        )
        .map_err(|error| error.to_string())?;
        fs::write(
            root.join("tailwind.config.ts"),
            "export default { safelist: ['hidden'] }",
        )
        .map_err(|error| error.to_string())?;
        let (report, _) = build_intel_report(root.clone())?;
        assert!(!report.enabled);
        assert_eq!(report.enumerated_class_count, 0);
        assert!(report.config_paths.is_empty());
        fs::remove_dir_all(root).map_err(|error| error.to_string())?;
        Ok(())
    }
}

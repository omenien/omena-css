use std::{fs, path::PathBuf};

use omena_checker::{FixSafetyEvidenceInputV0, compute_fix_safety};
use omena_query::{
    OmenaQueryPrettyFormatOptionsV0, OmenaQueryTransformPrintMode,
    OmenaQueryTransformPrintOptionsV0, OmenaQueryTransformStyleDialect,
    print_omena_query_transform_source_with_pretty_options,
};
use serde::Serialize;

use crate::{
    commands::FormatMode,
    config::find_omena_config_for_path,
    lint::discover_style_paths,
    output::{CliOutputMetadataV0, print_json},
    paths::path_string,
    write_safety::{
        SourceWriteErrorV0, SourceWriteEvidenceV0, SourceWriteModeV0, apply_write_with_safety,
    },
};

const DEFAULT_LINE_WIDTH: usize = 100;
const DEFAULT_INDENT_WIDTH: usize = 2;

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct FormatFileReportV0 {
    path: String,
    mode: &'static str,
    line_width: usize,
    indent_width: usize,
    changed: bool,
    idempotent: bool,
    written: bool,
    fallback_node_count: usize,
    fallback_reasons: Vec<&'static str>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct FormatReportV0 {
    schema_version: &'static str,
    product: &'static str,
    root: String,
    check: bool,
    pub(crate) file_count: usize,
    pub(crate) changed_file_count: usize,
    written_file_count: usize,
    pub(crate) non_idempotent_file_count: usize,
    files: Vec<FormatFileReportV0>,
}

struct FormatPlanV0 {
    path: PathBuf,
    output: String,
    report: FormatFileReportV0,
}

pub(crate) fn format_sources(
    path: Option<PathBuf>,
    mode: Option<FormatMode>,
    check: bool,
    json: bool,
) -> Result<(), String> {
    let report = build_format_report(path, mode, check)?;
    if json {
        print_json(CliOutputMetadataV0::new("omena-cli.format"), &report)?;
    } else {
        println!(
            "formatted {} file(s): {} changed, {} written",
            report.file_count, report.changed_file_count, report.written_file_count
        );
    }

    if report.non_idempotent_file_count > 0 {
        return Err(format!(
            "formatting was not idempotent for {} file(s); no changes were written",
            report.non_idempotent_file_count
        ));
    }
    if check && report.changed_file_count > 0 {
        return Err(format!(
            "{} file(s) require formatting",
            report.changed_file_count
        ));
    }
    Ok(())
}

pub(crate) fn build_format_report(
    path: Option<PathBuf>,
    mode: Option<FormatMode>,
    check: bool,
) -> Result<FormatReportV0, String> {
    let root = path.unwrap_or_else(|| PathBuf::from("."));
    let absolute_root = fs::canonicalize(&root).map_err(|error| {
        format!(
            "failed to resolve format root {}: {error}",
            path_string(root.as_path())
        )
    })?;
    let style_paths = discover_style_paths(absolute_root.as_path())?;
    if style_paths.is_empty() {
        return Err(format!(
            "no CSS-family sources found under {}",
            path_string(absolute_root.as_path())
        ));
    }

    let mut plans = style_paths
        .iter()
        .map(|style_path| build_format_plan(style_path, mode))
        .collect::<Result<Vec<_>, _>>()?;
    let non_idempotent_file_count = plans.iter().filter(|plan| !plan.report.idempotent).count();

    if !check && non_idempotent_file_count == 0 {
        for plan in &mut plans {
            if !plan.report.changed {
                continue;
            }
            apply_format_write(plan.path.as_path(), plan.output.as_str(), true)
                .map_err(|error| error.to_string())?;
            plan.report.written = true;
        }
    } else if !check && let Some(plan) = plans.iter().find(|plan| !plan.report.idempotent) {
        let Err(error) = apply_format_write(plan.path.as_path(), plan.output.as_str(), false)
        else {
            return Err("non-idempotent formatting unexpectedly passed the write gate".to_string());
        };
        if !matches!(error, SourceWriteErrorV0::Rejected(_)) {
            return Err(error.to_string());
        }
    }

    let files = plans
        .into_iter()
        .map(|plan| plan.report)
        .collect::<Vec<_>>();
    Ok(FormatReportV0 {
        schema_version: "0",
        product: "omena-cli.format-report",
        root: path_string(absolute_root.as_path()),
        check,
        file_count: files.len(),
        changed_file_count: files.iter().filter(|file| file.changed).count(),
        written_file_count: files.iter().filter(|file| file.written).count(),
        non_idempotent_file_count,
        files,
    })
}

fn build_format_plan(path: &PathBuf, cli_mode: Option<FormatMode>) -> Result<FormatPlanV0, String> {
    let source = fs::read_to_string(path)
        .map_err(|error| format!("failed to read {}: {error}", path_string(path)))?;
    let loaded_config = find_omena_config_for_path(path.as_path())?;
    let configured_mode = loaded_config
        .as_ref()
        .and_then(|loaded| loaded.config.format.mode.as_deref());
    let mode = resolve_format_mode(cli_mode, configured_mode)?;
    let line_width = loaded_config
        .as_ref()
        .and_then(|loaded| loaded.config.format.line_width)
        .map_or(DEFAULT_LINE_WIDTH, usize::from);
    let indent_width = loaded_config
        .as_ref()
        .and_then(|loaded| loaded.config.format.indent_width)
        .map_or(DEFAULT_INDENT_WIDTH, usize::from);
    let print_mode = match mode {
        FormatMode::Pretty => OmenaQueryTransformPrintMode::Pretty,
        FormatMode::Stable => OmenaQueryTransformPrintMode::Identity,
    };
    let pretty_options = OmenaQueryPrettyFormatOptionsV0 {
        line_width,
        indent_width,
    };
    let source_path = path_string(path);
    let first = print_omena_query_transform_source_with_pretty_options(
        source_path.as_str(),
        source.as_str(),
        dialect_for_path(path),
        format!("format:{source_path}"),
        &[],
        OmenaQueryTransformPrintOptionsV0 {
            mode: print_mode,
            include_source_map: false,
        },
        pretty_options,
    );
    let second = print_omena_query_transform_source_with_pretty_options(
        source_path.as_str(),
        first.css.as_str(),
        dialect_for_path(path),
        format!("format-observation:{source_path}"),
        &[],
        OmenaQueryTransformPrintOptionsV0 {
            mode: print_mode,
            include_source_map: false,
        },
        pretty_options,
    );
    let idempotent = first.css == second.css;
    let (fallback_node_count, fallback_reasons) = first
        .pretty_format_report
        .as_ref()
        .map_or((0, Vec::new()), |report| {
            (report.fallback_node_count, report.fallback_reasons.clone())
        });
    Ok(FormatPlanV0 {
        path: path.clone(),
        output: first.css.clone(),
        report: FormatFileReportV0 {
            path: source_path,
            mode: mode.as_str(),
            line_width,
            indent_width,
            changed: first.css != source,
            idempotent,
            written: false,
            fallback_node_count,
            fallback_reasons,
        },
    })
}

fn apply_format_write(
    path: &std::path::Path,
    output: &str,
    idempotent: bool,
) -> Result<(), SourceWriteErrorV0> {
    let assessment = compute_fix_safety(FixSafetyEvidenceInputV0 {
        syntax_preserving: true,
        local_semantics_required: false,
        local_semantics_ready: false,
        closed_world_required: false,
        closed_world_ready: false,
        reference_precision_required: false,
        reference_precision: None,
    });
    apply_write_with_safety(
        path,
        output.as_bytes(),
        &assessment,
        SourceWriteModeV0::SafeOnly,
        SourceWriteEvidenceV0::Formatting { idempotent },
    )?;
    Ok(())
}

fn resolve_format_mode(
    cli_mode: Option<FormatMode>,
    configured_mode: Option<&str>,
) -> Result<FormatMode, String> {
    if let Some(mode) = cli_mode {
        return Ok(mode);
    }
    match configured_mode {
        None | Some("pretty") => Ok(FormatMode::Pretty),
        Some("stable") => Ok(FormatMode::Stable),
        Some(mode) => Err(format!(
            "unsupported [format].mode `{mode}`; expected `pretty` or `stable`"
        )),
    }
}

fn dialect_for_path(path: &std::path::Path) -> OmenaQueryTransformStyleDialect {
    match path.extension().and_then(|extension| extension.to_str()) {
        Some("scss") => OmenaQueryTransformStyleDialect::Scss,
        Some("sass") => OmenaQueryTransformStyleDialect::Sass,
        Some("less") => OmenaQueryTransformStyleDialect::Less,
        _ => OmenaQueryTransformStyleDialect::Css,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicU64, Ordering};

    static NEXT_FIXTURE_ID: AtomicU64 = AtomicU64::new(0);

    #[test]
    fn pretty_check_observes_idempotence_without_writing() -> Result<(), String> {
        let root = fixture_root();
        fs::create_dir_all(&root).map_err(|error| error.to_string())?;
        let path = root.join("app.css");
        let source = ".app,.panel{color:red;}";
        fs::write(&path, source).map_err(|error| error.to_string())?;

        let report = build_format_report(Some(path.clone()), Some(FormatMode::Pretty), true)?;
        assert_eq!(report.changed_file_count, 1);
        assert_eq!(report.written_file_count, 0);
        assert_eq!(report.non_idempotent_file_count, 0);
        assert!(report.files[0].idempotent);
        assert_eq!(
            fs::read_to_string(&path).map_err(|error| error.to_string())?,
            source
        );

        fs::remove_dir_all(root).map_err(|error| error.to_string())?;
        Ok(())
    }

    #[test]
    fn pretty_write_uses_config_and_shared_safety_gate() -> Result<(), String> {
        let root = fixture_root();
        fs::create_dir_all(&root).map_err(|error| error.to_string())?;
        let path = root.join("app.css");
        fs::write(&path, ".app,.panel{color:red;}").map_err(|error| error.to_string())?;
        fs::write(
            root.join("omena.toml"),
            "[format]\nmode = \"pretty\"\nlineWidth = 80\nindentWidth = 4\n",
        )
        .map_err(|error| error.to_string())?;

        let report = build_format_report(Some(path.clone()), None, false)?;
        assert_eq!(report.written_file_count, 1);
        assert!(report.files[0].idempotent);
        assert_eq!(report.files[0].line_width, 80);
        assert_eq!(report.files[0].indent_width, 4);
        let formatted = fs::read_to_string(&path).map_err(|error| error.to_string())?;
        assert!(formatted.contains("\n    color: red;"));

        let stable = build_format_report(Some(path), Some(FormatMode::Stable), true)?;
        assert_eq!(stable.changed_file_count, 0);
        fs::remove_dir_all(root).map_err(|error| error.to_string())?;
        Ok(())
    }

    #[test]
    fn non_idempotent_formatting_is_rejected_before_write() -> Result<(), String> {
        let root = fixture_root();
        fs::create_dir_all(&root).map_err(|error| error.to_string())?;
        let path = root.join("app.css");
        fs::write(&path, ".app {}").map_err(|error| error.to_string())?;

        let Err(error) = apply_format_write(path.as_path(), ".changed {}", false) else {
            return Err(
                "non-idempotent formatting unexpectedly reached the filesystem".to_string(),
            );
        };
        assert!(matches!(error, SourceWriteErrorV0::Rejected(_)));
        assert_eq!(
            fs::read_to_string(&path).map_err(|error| error.to_string())?,
            ".app {}"
        );

        fs::remove_dir_all(root).map_err(|error| error.to_string())?;
        Ok(())
    }

    fn fixture_root() -> PathBuf {
        let id = NEXT_FIXTURE_ID.fetch_add(1, Ordering::Relaxed);
        std::env::temp_dir().join(format!("omena-format-{}-{id}", std::process::id()))
    }
}

mod manifest;
mod report;
mod workspace;

use std::path::PathBuf;

use crate::output::{CliOutputMetadataV0, print_json};

pub(crate) use manifest::ci_adapters;
pub(crate) use workspace::verify_workspace;

pub(crate) fn verify_command(
    root: Option<PathBuf>,
    engine_self: bool,
    json: bool,
) -> Result<(), String> {
    let execution = verify_workspace(root, engine_self)?;
    for warning in &execution.warnings {
        eprintln!("warning: {warning}");
    }
    let report = execution.report;
    if json {
        print_json(
            CliOutputMetadataV0::new("omena-cli.verify")
                .with_config_content_digest(execution.config_content_digest.as_deref()),
            &report,
        )?;
    } else {
        println!(
            "verification: {} passed, {} failed, {} indeterminate, {} not yet available, {} skipped",
            report.passed_count,
            report.failed_count,
            report.indeterminate_count,
            report.not_yet_available_count,
            report.skipped_count
        );
        for item in &report.items {
            println!("{:?}\t{}\t{}", item.outcome, item.id, item.summary);
        }
    }
    if report.blocking_failure_count > 0 {
        return Err(format!(
            "workspace verification failed closed with {} blocking item(s)",
            report.blocking_failure_count
        ));
    }
    Ok(())
}

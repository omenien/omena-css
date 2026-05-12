use clap::{Parser, Subcommand};
use omena_parser::{StyleDialect, dialect_for_path, parse, summarize_omena_parser_style_facts};
use omena_transform_cst::{TransformPassKind, all_transform_pass_kinds};
use omena_transform_passes::{
    TransformExecutionSummaryV0, execute_transform_passes_on_source_with_dialect,
};
use serde::Serialize;
use std::{
    fs,
    path::{Path, PathBuf},
    process::ExitCode,
};

#[derive(Debug, Parser)]
#[command(
    name = "omena",
    about = "Check and transform CSS-family sources with the Omena CSS workspace"
)]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Debug, Subcommand)]
enum Command {
    /// Parse a CSS-family source and report parser-owned facts.
    Check {
        /// CSS, SCSS, Sass, Less, or CSS Modules file to check.
        path: PathBuf,
        /// Print machine-readable JSON.
        #[arg(long)]
        json: bool,
    },
    /// Run the conservative transform pipeline.
    Build {
        /// CSS, SCSS, Sass, Less, or CSS Modules file to transform.
        path: PathBuf,
        /// Optional output file. Prints to stdout when omitted.
        #[arg(short, long)]
        output: Option<PathBuf>,
        /// Transform pass id. Repeat to choose specific passes.
        #[arg(long = "pass")]
        passes: Vec<String>,
        /// Print a machine-readable execution summary.
        #[arg(long)]
        json: bool,
    },
    /// List transform pass ids accepted by `omena build --pass`.
    Passes {
        /// Print machine-readable JSON.
        #[arg(long)]
        json: bool,
    },
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct CheckSummary {
    schema_version: &'static str,
    product: &'static str,
    file: String,
    dialect: &'static str,
    token_count: usize,
    parser_error_count: usize,
    class_selector_count: usize,
    custom_property_count: usize,
    keyframe_count: usize,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct PassSummary {
    id: &'static str,
    title: &'static str,
    reads_semantic_graph: bool,
    reads_cascade_model: bool,
}

fn main() -> ExitCode {
    match run(Cli::parse()) {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("{error}");
            ExitCode::FAILURE
        }
    }
}

fn run(cli: Cli) -> Result<(), String> {
    match cli.command {
        Command::Check { path, json } => check_file(path, json),
        Command::Build {
            path,
            output,
            passes,
            json,
        } => build_file(path, output, passes, json),
        Command::Passes { json } => list_passes(json),
    }
}

fn check_file(path: PathBuf, json: bool) -> Result<(), String> {
    let source = read_source(&path)?;
    let dialect = dialect_for_path(path_string(&path).as_str());
    let parse_result = parse(&source, dialect);
    let style_facts = summarize_omena_parser_style_facts(&source, dialect);
    let summary = CheckSummary {
        schema_version: "0",
        product: "omena-cli.check",
        file: path_string(&path),
        dialect: dialect_label(dialect),
        token_count: parse_result.token_count(),
        parser_error_count: parse_result.errors().len(),
        class_selector_count: style_facts.class_selector_names.len(),
        custom_property_count: style_facts.custom_property_names.len(),
        keyframe_count: style_facts.keyframe_names.len(),
    };

    if json {
        print_json(&summary)?;
        return Ok(());
    }

    println!("file: {}", summary.file);
    println!("dialect: {}", summary.dialect);
    println!("tokens: {}", summary.token_count);
    println!("parse errors: {}", summary.parser_error_count);
    println!("class selectors: {}", summary.class_selector_count);
    println!("custom properties: {}", summary.custom_property_count);
    println!("keyframes: {}", summary.keyframe_count);
    Ok(())
}

fn build_file(
    path: PathBuf,
    output: Option<PathBuf>,
    pass_ids: Vec<String>,
    json: bool,
) -> Result<(), String> {
    let source = read_source(&path)?;
    let dialect = dialect_for_path(path_string(&path).as_str());
    let passes = parse_pass_ids(&pass_ids)?;
    let execution = execute_transform_passes_on_source_with_dialect(&source, dialect, &passes);

    if let Some(output_path) = output {
        fs::write(&output_path, &execution.output_css).map_err(|error| {
            format!(
                "failed to write transformed CSS to {}: {error}",
                path_string(&output_path)
            )
        })?;
    } else if !json {
        print!("{}", execution.output_css);
    }

    if json {
        print_json(&execution_summary_for_cli(&execution, &path, dialect))?;
        return Ok(());
    }

    eprintln!(
        "executed passes: {}",
        execution.executed_pass_ids.join(", ")
    );
    eprintln!(
        "planned-only passes: {}",
        execution.planned_only_pass_ids.join(", ")
    );
    eprintln!("mutations: {}", execution.mutation_count);
    Ok(())
}

fn list_passes(json: bool) -> Result<(), String> {
    let passes = all_transform_pass_kinds()
        .into_iter()
        .map(|kind| PassSummary {
            id: kind.id(),
            title: kind.title(),
            reads_semantic_graph: kind.reads_semantic_graph(),
            reads_cascade_model: kind.reads_cascade_model(),
        })
        .collect::<Vec<_>>();

    if json {
        print_json(&passes)?;
        return Ok(());
    }

    for pass in passes {
        println!("{}\t{}", pass.id, pass.title);
    }
    Ok(())
}

fn parse_pass_ids(pass_ids: &[String]) -> Result<Vec<TransformPassKind>, String> {
    if pass_ids.is_empty() {
        return Ok(all_transform_pass_kinds().to_vec());
    }

    pass_ids
        .iter()
        .map(|pass_id| {
            all_transform_pass_kinds()
                .into_iter()
                .find(|kind| kind.id() == pass_id)
                .ok_or_else(|| format!("unknown transform pass id: {pass_id}"))
        })
        .collect()
}

fn read_source(path: &Path) -> Result<String, String> {
    fs::read_to_string(path)
        .map_err(|error| format!("failed to read {}: {error}", path_string(path)))
}

fn print_json<T: Serialize>(value: &T) -> Result<(), String> {
    let json = serde_json::to_string_pretty(value)
        .map_err(|error| format!("failed to serialize JSON: {error}"))?;
    println!("{json}");
    Ok(())
}

fn execution_summary_for_cli(
    execution: &TransformExecutionSummaryV0,
    path: &Path,
    dialect: StyleDialect,
) -> serde_json::Value {
    serde_json::json!({
        "schemaVersion": "0",
        "product": "omena-cli.build",
        "file": path_string(path),
        "dialect": dialect_label(dialect),
        "inputByteLen": execution.input_byte_len,
        "outputByteLen": execution.output_byte_len,
        "requestedPassIds": execution.requested_pass_ids,
        "orderedPassIds": execution.ordered_pass_ids,
        "executedPassIds": execution.executed_pass_ids,
        "plannedOnlyPassIds": execution.planned_only_pass_ids,
        "mutationCount": execution.mutation_count,
        "provenancePreserved": execution.provenance_preserved,
        "outputCss": execution.output_css,
    })
}

fn path_string(path: &Path) -> String {
    path.to_string_lossy().into_owned()
}

fn dialect_label(dialect: StyleDialect) -> &'static str {
    match dialect {
        StyleDialect::Css => "css",
        StyleDialect::Scss => "scss",
        StyleDialect::Sass => "sass",
        StyleDialect::Less => "less",
    }
}

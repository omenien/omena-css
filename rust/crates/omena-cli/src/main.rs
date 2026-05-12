use clap::{Parser, Subcommand};
use omena_query::{
    OmenaQueryEngineInputV2, OmenaQueryStylePackageManifestV0, OmenaQueryStyleSourceInputV0,
    OmenaQueryTargetTransformOptionsV0, OmenaQueryTransformExecutionContextV0,
    execute_omena_query_consumer_build_style_source_for_target_query_with_context_and_options,
    execute_omena_query_consumer_build_style_source_with_context,
    execute_omena_query_consumer_build_style_sources_for_target_query_with_context_and_options,
    execute_omena_query_consumer_build_style_sources_with_context,
    list_omena_query_transform_pass_summaries, summarize_omena_query_consumer_check_style_source,
    summarize_omena_query_transform_context_from_engine_input,
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
        /// Browserslist query or named target profile used to plan target-sensitive passes.
        #[arg(long)]
        target_query: Option<String>,
        /// Allow logical property lowering when target query says it is needed.
        #[arg(long)]
        allow_logical_to_physical: bool,
        /// Allow @scope flattening when target query says it is needed.
        #[arg(long)]
        allow_scope_flatten: bool,
        /// Allow cascade layer flattening when target query says it is needed.
        #[arg(long)]
        allow_layer_flatten: bool,
        /// Enable static @supports branch evaluation.
        #[arg(long)]
        enable_supports_static_eval: bool,
        /// Enable static @media branch evaluation.
        #[arg(long)]
        enable_media_static_eval: bool,
        /// JSON file containing a TransformExecutionContextV0 evaluator/provenance bridge.
        #[arg(long)]
        context_json: Option<PathBuf>,
        /// JSON file containing EngineInputV2 source/style/type facts for semantic reachability.
        #[arg(long)]
        engine_input_json: Option<PathBuf>,
        /// Treat the provided context/engine input as a closed style world for tree shaking.
        #[arg(long)]
        closed_style_world: bool,
        /// Additional workspace style source used to derive import/composes build context.
        #[arg(long = "source")]
        source_paths: Vec<PathBuf>,
        /// package.json file used to resolve package style exports for workspace sources.
        #[arg(long = "package-manifest")]
        package_manifest_paths: Vec<PathBuf>,
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
            target_query,
            allow_logical_to_physical,
            allow_scope_flatten,
            allow_layer_flatten,
            enable_supports_static_eval,
            enable_media_static_eval,
            context_json,
            engine_input_json,
            closed_style_world,
            source_paths,
            package_manifest_paths,
            json,
        } => build_file(BuildFileOptions {
            path,
            output,
            pass_ids: passes,
            target_query,
            context_json,
            engine_input_json,
            closed_style_world,
            source_paths,
            package_manifest_paths,
            target_options: OmenaQueryTargetTransformOptionsV0 {
                allow_logical_to_physical,
                allow_scope_flatten,
                allow_layer_flatten,
                enable_supports_static_eval,
                enable_media_static_eval,
            },
            json,
        }),
        Command::Passes { json } => list_passes(json),
    }
}

fn check_file(path: PathBuf, json: bool) -> Result<(), String> {
    let source = read_source(&path)?;
    let summary = summarize_omena_query_consumer_check_style_source(&path_string(&path), &source);

    if json {
        print_json(&summary)?;
        return Ok(());
    }

    println!("file: {}", summary.style_path);
    println!("dialect: {}", summary.dialect);
    println!("tokens: {}", summary.token_count);
    println!("parse errors: {}", summary.parser_error_count);
    println!("class selectors: {}", summary.class_selector_count);
    println!("custom properties: {}", summary.custom_property_count);
    println!("keyframes: {}", summary.keyframe_count);
    Ok(())
}

struct BuildFileOptions {
    path: PathBuf,
    output: Option<PathBuf>,
    pass_ids: Vec<String>,
    target_query: Option<String>,
    context_json: Option<PathBuf>,
    engine_input_json: Option<PathBuf>,
    closed_style_world: bool,
    source_paths: Vec<PathBuf>,
    package_manifest_paths: Vec<PathBuf>,
    target_options: OmenaQueryTargetTransformOptionsV0,
    json: bool,
}

fn build_file(options: BuildFileOptions) -> Result<(), String> {
    let BuildFileOptions {
        path,
        output,
        pass_ids,
        target_query,
        context_json,
        engine_input_json,
        closed_style_world,
        source_paths,
        package_manifest_paths,
        target_options,
        json,
    } = options;

    if target_query.is_some() && !pass_ids.is_empty() {
        return Err("cannot combine --target-query with explicit --pass values".to_string());
    }

    let source = read_source(&path)?;
    let style_path = path_string(&path);
    let mut context = read_context_json(context_json.as_deref())?;
    if closed_style_world {
        context.closed_style_world = true;
    }
    if let Some(engine_input_path) = engine_input_json.as_deref() {
        let engine_input = read_engine_input_json(engine_input_path)?;
        let engine_context = summarize_omena_query_transform_context_from_engine_input(
            &engine_input,
            &style_path,
            context.closed_style_world,
        )
        .context;
        context = merge_cli_transform_context(context, &engine_context);
    }
    let workspace_sources = read_workspace_sources(&path, &source, &source_paths)?;
    let package_manifests = read_package_manifests(&package_manifest_paths)?;
    let summary = if let Some(target_query) = target_query {
        if workspace_sources.len() > 1 {
            execute_omena_query_consumer_build_style_sources_for_target_query_with_context_and_options(
                &style_path,
                &workspace_sources,
                &target_query,
                &context,
                target_options,
                &package_manifests,
            )?
        } else {
            execute_omena_query_consumer_build_style_source_for_target_query_with_context_and_options(
                &style_path,
                &source,
                &target_query,
                &context,
                target_options,
            )
        }
    } else if workspace_sources.len() > 1 {
        execute_omena_query_consumer_build_style_sources_with_context(
            &style_path,
            &workspace_sources,
            &pass_ids,
            &context,
            &package_manifests,
        )?
    } else {
        execute_omena_query_consumer_build_style_source_with_context(
            &style_path,
            &source,
            &pass_ids,
            &context,
        )
    };

    if !summary.unknown_pass_ids.is_empty() {
        return Err(format!(
            "unknown transform pass id: {}",
            summary.unknown_pass_ids.join(", ")
        ));
    }

    if let Some(output_path) = output {
        fs::write(&output_path, &summary.execution.output_css).map_err(|error| {
            format!(
                "failed to write transformed CSS to {}: {error}",
                path_string(&output_path)
            )
        })?;
    } else if !json {
        print!("{}", summary.execution.output_css);
    }

    if json {
        print_json(&summary)?;
        return Ok(());
    }

    eprintln!(
        "executed passes: {}",
        summary.execution.executed_pass_ids.join(", ")
    );
    eprintln!(
        "planned-only passes: {}",
        summary.execution.planned_only_pass_ids.join(", ")
    );
    eprintln!("mutations: {}", summary.execution.mutation_count);
    Ok(())
}

fn read_workspace_sources(
    target_path: &Path,
    target_source: &str,
    additional_paths: &[PathBuf],
) -> Result<Vec<OmenaQueryStyleSourceInputV0>, String> {
    let target_path_string = path_string(target_path);
    let mut sources = vec![OmenaQueryStyleSourceInputV0 {
        style_path: target_path_string.clone(),
        style_source: target_source.to_string(),
    }];

    for source_path in additional_paths {
        let source_path_string = path_string(source_path);
        if source_path_string == target_path_string {
            continue;
        }
        sources.push(OmenaQueryStyleSourceInputV0 {
            style_path: source_path_string,
            style_source: read_source(source_path)?,
        });
    }

    Ok(sources)
}

fn read_package_manifests(
    package_manifest_paths: &[PathBuf],
) -> Result<Vec<OmenaQueryStylePackageManifestV0>, String> {
    package_manifest_paths
        .iter()
        .map(|path| {
            Ok(OmenaQueryStylePackageManifestV0 {
                package_json_path: path_string(path),
                package_json_source: read_source(path)?,
            })
        })
        .collect()
}

fn list_passes(json: bool) -> Result<(), String> {
    let passes = list_omena_query_transform_pass_summaries();

    if json {
        print_json(&passes)?;
        return Ok(());
    }

    for pass in passes {
        println!("{}\t{}", pass.id, pass.title);
    }
    Ok(())
}

fn read_source(path: &Path) -> Result<String, String> {
    fs::read_to_string(path)
        .map_err(|error| format!("failed to read {}: {error}", path_string(path)))
}

fn read_context_json(path: Option<&Path>) -> Result<OmenaQueryTransformExecutionContextV0, String> {
    let Some(path) = path else {
        return Ok(OmenaQueryTransformExecutionContextV0::default());
    };
    let json = fs::read_to_string(path)
        .map_err(|error| format!("failed to read context JSON {}: {error}", path_string(path)))?;
    serde_json::from_str(&json).map_err(|error| {
        format!(
            "failed to parse context JSON {}: {error}",
            path_string(path)
        )
    })
}

fn read_engine_input_json(path: &Path) -> Result<OmenaQueryEngineInputV2, String> {
    let json = fs::read_to_string(path).map_err(|error| {
        format!(
            "failed to read engine input JSON {}: {error}",
            path_string(path)
        )
    })?;
    serde_json::from_str(&json).map_err(|error| {
        format!(
            "failed to parse engine input JSON {}: {error}",
            path_string(path)
        )
    })
}

fn merge_cli_transform_context(
    mut base: OmenaQueryTransformExecutionContextV0,
    additional: &OmenaQueryTransformExecutionContextV0,
) -> OmenaQueryTransformExecutionContextV0 {
    base.closed_style_world = base.closed_style_world || additional.closed_style_world;
    merge_cli_context_list(
        &mut base.reachable_class_names,
        &additional.reachable_class_names,
    );
    merge_cli_context_list(
        &mut base.reachable_keyframe_names,
        &additional.reachable_keyframe_names,
    );
    merge_cli_context_list(
        &mut base.reachable_value_names,
        &additional.reachable_value_names,
    );
    merge_cli_context_list(
        &mut base.reachable_custom_property_names,
        &additional.reachable_custom_property_names,
    );
    base
}

fn merge_cli_context_list(target: &mut Vec<String>, additional: &[String]) {
    for item in additional {
        if !target.contains(item) {
            target.push(item.clone());
        }
    }
    target.sort();
}

fn print_json<T: Serialize>(value: &T) -> Result<(), String> {
    let json = serde_json::to_string_pretty(value)
        .map_err(|error| format!("failed to serialize JSON: {error}"))?;
    println!("{json}");
    Ok(())
}

fn path_string(path: &Path) -> String {
    path.to_string_lossy().into_owned()
}

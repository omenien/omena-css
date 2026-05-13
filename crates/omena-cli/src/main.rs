use clap::{Parser, Subcommand};
use omena_query::{
    OmenaQueryEngineInputV2, OmenaQueryExpressionDomainFlowRuntimeV0,
    OmenaQueryStylePackageManifestV0, OmenaQueryStyleSourceInputV0,
    OmenaQueryTargetTransformOptionsV0, OmenaQueryTransformExecutionContextV0, ParserPositionV0,
    execute_omena_query_consumer_build_style_source_for_target_query_with_context_and_options,
    execute_omena_query_consumer_build_style_source_with_context,
    execute_omena_query_consumer_build_style_sources_for_target_query_with_context_and_options,
    execute_omena_query_consumer_build_style_sources_with_context,
    list_omena_query_transform_pass_summaries, read_omena_query_cascade_at_position,
    read_omena_query_style_context_index, summarize_omena_query_consumer_check_style_source,
    summarize_omena_query_expression_domain_incremental_flow_analysis,
    summarize_omena_query_expression_domain_selector_projection,
    summarize_omena_query_style_diagnostics_for_file, summarize_omena_query_style_hover_candidates,
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
        /// Drop dark color-scheme media branches when workspace policy proves no dark-mode runtime.
        #[arg(long)]
        drop_dark_mode_media_queries: bool,
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
    /// Derive transform context from EngineInputV2 semantic reachability.
    Context {
        /// Target CSS, SCSS, Sass, Less, or CSS Modules path.
        path: PathBuf,
        /// JSON file containing EngineInputV2 source/style/type facts.
        #[arg(long)]
        engine_input_json: PathBuf,
        /// Treat the engine input as a closed style world for tree shaking.
        #[arg(long)]
        closed_style_world: bool,
        /// Print machine-readable JSON.
        #[arg(long)]
        json: bool,
    },
    /// Analyze cross-language class-value flow from EngineInputV2.
    ExpressionFlow {
        /// JSON file containing EngineInputV2 source/style/type facts.
        #[arg(long)]
        engine_input_json: PathBuf,
        /// Print machine-readable JSON.
        #[arg(long)]
        json: bool,
    },
    /// Project expression-domain flow values to target style selectors.
    SelectorProjection {
        /// JSON file containing EngineInputV2 source/style/type facts.
        #[arg(long)]
        engine_input_json: PathBuf,
        /// Print machine-readable JSON.
        #[arg(long)]
        json: bool,
    },
    /// Read cascade and custom-property LFP information at a source position.
    Cascade {
        /// CSS, SCSS, Sass, Less, or CSS Modules file to inspect.
        path: PathBuf,
        /// Zero-based line number.
        #[arg(long)]
        line: usize,
        /// Zero-based UTF-16-like character offset used by the query protocol.
        #[arg(long)]
        character: usize,
        /// Optional EngineInputV2 JSON file for source/type context.
        #[arg(long)]
        engine_input_json: Option<PathBuf>,
        /// Print machine-readable JSON.
        #[arg(long)]
        json: bool,
    },
    /// Read @layer, @container, and @scope context indexes.
    ContextIndex {
        /// CSS, SCSS, Sass, Less, or CSS Modules file to inspect.
        path: PathBuf,
        /// Optional EngineInputV2 JSON file for source/type context.
        #[arg(long)]
        engine_input_json: Option<PathBuf>,
        /// Print machine-readable JSON.
        #[arg(long)]
        json: bool,
    },
    /// Read query-owned style diagnostics for a CSS-family file.
    StyleDiagnostics {
        /// CSS, SCSS, Sass, Less, or CSS Modules file to inspect.
        path: PathBuf,
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
            drop_dark_mode_media_queries,
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
                drop_dark_mode_media_queries,
            },
            json,
        }),
        Command::Passes { json } => list_passes(json),
        Command::Context {
            path,
            engine_input_json,
            closed_style_world,
            json,
        } => context_from_engine_input(path, engine_input_json, closed_style_world, json),
        Command::ExpressionFlow {
            engine_input_json,
            json,
        } => expression_flow(engine_input_json, json),
        Command::SelectorProjection {
            engine_input_json,
            json,
        } => selector_projection(engine_input_json, json),
        Command::Cascade {
            path,
            line,
            character,
            engine_input_json,
            json,
        } => cascade_at_position(path, line, character, engine_input_json, json),
        Command::ContextIndex {
            path,
            engine_input_json,
            json,
        } => context_index(path, engine_input_json, json),
        Command::StyleDiagnostics { path, json } => style_diagnostics(path, json),
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
    let used_engine_input = engine_input_json.is_some();
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
    let mut summary = if let Some(target_query) = target_query {
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
    if used_engine_input {
        push_ready_surface(
            &mut summary.ready_surfaces,
            "semanticReachabilityTransformContext",
        );
        push_ready_surface(
            &mut summary.ready_surfaces,
            "expressionDomainSelectorProjection",
        );
    }

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

fn context_from_engine_input(
    path: PathBuf,
    engine_input_json: PathBuf,
    closed_style_world: bool,
    json: bool,
) -> Result<(), String> {
    let style_path = path_string(&path);
    let engine_input = read_engine_input_json(&engine_input_json)?;
    let summary = summarize_omena_query_transform_context_from_engine_input(
        &engine_input,
        &style_path,
        closed_style_world,
    );

    if json {
        print_json(&summary)?;
        return Ok(());
    }

    println!("target: {}", summary.target_style_path);
    println!("closed style world: {}", summary.closed_style_world);
    println!("projections: {}", summary.projection_count);
    println!(
        "selected projections: {}",
        summary.selected_projection_count
    );
    println!("reachable classes: {}", summary.reachable_class_name_count);
    for class_name in &summary.context.reachable_class_names {
        println!("  {class_name}");
    }
    Ok(())
}

fn expression_flow(engine_input_json: PathBuf, json: bool) -> Result<(), String> {
    let engine_input = read_engine_input_json(&engine_input_json)?;
    let mut runtime = OmenaQueryExpressionDomainFlowRuntimeV0::default();
    let summary = summarize_omena_query_expression_domain_incremental_flow_analysis(
        &engine_input,
        &mut runtime,
    );

    if json {
        print_json(&summary)?;
        return Ok(());
    }

    println!("product: {}", summary.product);
    println!("revision: {}", summary.revision);
    println!("graphs: {}", summary.graph_count);
    println!("dirty graphs: {}", summary.dirty_graph_count);
    println!("reused graphs: {}", summary.reused_graph_count);
    for entry in &summary.analyses {
        println!(
            "{}\tnodes={}\tdirty={}\treused={}",
            entry.graph_id,
            entry.analysis.analysis.nodes.len(),
            entry.analysis.incremental_plan.dirty_node_count,
            entry.analysis.reused_previous_analysis
        );
    }
    Ok(())
}

fn selector_projection(engine_input_json: PathBuf, json: bool) -> Result<(), String> {
    let engine_input = read_engine_input_json(&engine_input_json)?;
    let summary = summarize_omena_query_expression_domain_selector_projection(&engine_input);

    if json {
        print_json(&summary)?;
        return Ok(());
    }

    println!("product: {}", summary.product);
    println!("projections: {}", summary.projection_count);
    for projection in &summary.projections {
        println!(
            "{}\t{}\t{:?}\t{}",
            projection.graph_id,
            projection.node_id,
            projection.certainty,
            projection.selector_names.join(",")
        );
    }
    Ok(())
}

fn cascade_at_position(
    path: PathBuf,
    line: usize,
    character: usize,
    engine_input_json: Option<PathBuf>,
    json: bool,
) -> Result<(), String> {
    let source = read_source(&path)?;
    let style_path = path_string(&path);
    let engine_input = if let Some(engine_input_path) = engine_input_json.as_deref() {
        read_engine_input_json(engine_input_path)?
    } else {
        empty_engine_input()
    };
    let position = ParserPositionV0 { line, character };
    let Some(summary) =
        read_omena_query_cascade_at_position(&style_path, &source, &engine_input, position)
    else {
        return Err(format!(
            "failed to read cascade information for {style_path}:{line}:{character}",
        ));
    };

    if json {
        print_json(&summary)?;
        return Ok(());
    }

    println!("file: {}", summary.style_path);
    println!("status: {}", summary.status);
    println!(
        "reference: {}",
        summary.reference_name.as_deref().unwrap_or("-")
    );
    println!(
        "computed status: {}",
        summary
            .referenced_declaration_computed_value_status
            .unwrap_or("-")
    );
    println!(
        "computed value: {}",
        summary
            .referenced_declaration_computed_value
            .as_deref()
            .unwrap_or("-")
    );
    println!(
        "lfp status: {}",
        summary
            .reference_custom_property_fixed_point_status
            .unwrap_or("-")
    );
    println!(
        "lfp value: {}",
        summary
            .reference_custom_property_fixed_point_value
            .as_deref()
            .unwrap_or("-")
    );
    println!(
        "lfp iterations: {}",
        summary.custom_property_fixed_point_iteration_count
    );
    println!(
        "lfp guaranteed-invalid count: {}",
        summary.custom_property_fixed_point_guaranteed_invalid_count
    );
    Ok(())
}

fn context_index(
    path: PathBuf,
    engine_input_json: Option<PathBuf>,
    json: bool,
) -> Result<(), String> {
    let source = read_source(&path)?;
    let style_path = path_string(&path);
    let engine_input = if let Some(engine_input_path) = engine_input_json.as_deref() {
        read_engine_input_json(engine_input_path)?
    } else {
        empty_engine_input()
    };
    let Some(summary) = read_omena_query_style_context_index(&style_path, &source, &engine_input)
    else {
        return Err(format!("failed to read context index for {style_path}"));
    };

    if json {
        print_json(&summary)?;
        return Ok(());
    }

    println!("file: {}", summary.style_path);
    println!("source: {}", summary.context_index_source);
    println!(
        "layer blocks: {}",
        summary.context_index.layer_index.block_layers.len()
    );
    println!(
        "layer statements: {}",
        summary.context_index.layer_index.statement_layers.len()
    );
    println!(
        "containers: {}",
        summary.context_index.container_index.containers.len()
    );
    println!("scopes: {}", summary.context_index.scope_index.scopes.len());
    println!(
        "selector context memberships: {}",
        summary.context_index.selector_context_count
    );
    Ok(())
}

fn style_diagnostics(path: PathBuf, json: bool) -> Result<(), String> {
    let source = read_source(&path)?;
    let style_path = path_string(&path);
    let Some(candidates) = summarize_omena_query_style_hover_candidates(&style_path, &source)
    else {
        return Err(format!(
            "failed to read style candidates for {}",
            path_string(&path)
        ));
    };
    let summary = summarize_omena_query_style_diagnostics_for_file(
        &style_path,
        &source,
        candidates.candidates.as_slice(),
    );

    if json {
        print_json(&summary)?;
        return Ok(());
    }

    println!("file: {}", summary.file_uri);
    println!("diagnostics: {}", summary.diagnostic_count);
    for diagnostic in &summary.diagnostics {
        println!("{}\t{}", diagnostic.code, diagnostic.message);
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

fn empty_engine_input() -> OmenaQueryEngineInputV2 {
    OmenaQueryEngineInputV2 {
        version: "2".to_string(),
        sources: Vec::new(),
        styles: Vec::new(),
        type_facts: Vec::new(),
    }
}

fn merge_cli_transform_context(
    mut base: OmenaQueryTransformExecutionContextV0,
    additional: &OmenaQueryTransformExecutionContextV0,
) -> OmenaQueryTransformExecutionContextV0 {
    base.closed_style_world = base.closed_style_world || additional.closed_style_world;
    base.drop_dark_mode_media_queries =
        base.drop_dark_mode_media_queries || additional.drop_dark_mode_media_queries;
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

fn push_ready_surface(surfaces: &mut Vec<&'static str>, surface: &'static str) {
    if !surfaces.contains(&surface) {
        surfaces.push(surface);
    }
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{SystemTime, UNIX_EPOCH};

    #[test]
    fn build_command_writes_query_owned_transform_output() {
        let source_path = temp_path("input.css");
        let output_path = temp_path("output.css");
        fs::write(&source_path, ".card { color: #ffffff; }")
            .expect("fixture source should be writable");

        let result = run(Cli {
            command: Command::Build {
                path: source_path.clone(),
                output: Some(output_path.clone()),
                passes: vec![
                    "whitespace-strip".to_string(),
                    "color-compression".to_string(),
                ],
                target_query: None,
                allow_logical_to_physical: false,
                allow_scope_flatten: false,
                allow_layer_flatten: false,
                enable_supports_static_eval: false,
                enable_media_static_eval: false,
                drop_dark_mode_media_queries: false,
                context_json: None,
                engine_input_json: None,
                closed_style_world: false,
                source_paths: Vec::new(),
                package_manifest_paths: Vec::new(),
                json: false,
            },
        });

        assert!(result.is_ok(), "{result:?}");
        let output = fs::read_to_string(&output_path).expect("build output should be written");
        assert!(output.contains("#fff"));
        assert!(!output.contains("#ffffff"));

        cleanup(&source_path);
        cleanup(&output_path);
    }

    #[test]
    fn cascade_and_context_index_commands_read_query_surfaces() {
        let source_path = temp_path("input.module.css");
        fs::write(
            &source_path,
            "@layer components { :root { --brand: #2563eb; } }\n.button { color: var(--brand); }\n",
        )
        .expect("fixture source should be writable");

        let cascade_result = run(Cli {
            command: Command::Cascade {
                path: source_path.clone(),
                line: 1,
                character: 24,
                engine_input_json: None,
                json: true,
            },
        });
        assert!(cascade_result.is_ok(), "{cascade_result:?}");

        let context_result = run(Cli {
            command: Command::ContextIndex {
                path: source_path.clone(),
                engine_input_json: None,
                json: true,
            },
        });
        assert!(context_result.is_ok(), "{context_result:?}");

        cleanup(&source_path);
    }

    #[test]
    fn style_diagnostics_command_reads_query_owned_diagnostics() {
        let source_path = temp_path("diagnostics.module.css");
        fs::write(
            &source_path,
            ":root { --known: #2563eb; }\n.button { color: var(--missing); }\n",
        )
        .expect("fixture source should be writable");

        let result = run(Cli {
            command: Command::StyleDiagnostics {
                path: source_path.clone(),
                json: true,
            },
        });

        assert!(result.is_ok(), "{result:?}");

        cleanup(&source_path);
    }

    fn temp_path(name: &str) -> PathBuf {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system time should be after unix epoch")
            .as_nanos();
        std::env::temp_dir().join(format!("omena-cli-{nanos}-{name}"))
    }

    fn cleanup(path: &Path) {
        let _ = fs::remove_file(path);
    }
}

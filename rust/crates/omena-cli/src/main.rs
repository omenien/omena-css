use clap::{Parser, Subcommand};
#[cfg(feature = "mdl")]
use omena_query::summarize_omena_query_design_system_minimum_description;
use omena_query::{
    OmenaQueryEngineInputV2, OmenaQueryExpressionDomainFlowRuntimeV0,
    OmenaQuerySourceDiagnosticsForFileV0, OmenaQuerySourceDocumentInputV0,
    OmenaQuerySourceMissingSelectorDiagnosticCandidateV0, OmenaQueryStylePackageManifestV0,
    OmenaQueryStyleSourceInputV0, OmenaQueryTargetTransformOptionsV0,
    OmenaQueryTransformExecutionContextV0, ParserPositionV0,
    execute_omena_query_consumer_build_style_source_for_target_query_with_context_and_options,
    execute_omena_query_consumer_build_style_source_with_context,
    execute_omena_query_consumer_build_style_sources_for_target_query_with_context_and_options,
    execute_omena_query_consumer_build_style_sources_with_context,
    list_omena_query_transform_pass_summaries, read_omena_query_cascade_at_position,
    read_omena_query_style_context_index, summarize_omena_query_consumer_check_style_source,
    summarize_omena_query_expression_domain_incremental_flow_analysis,
    summarize_omena_query_expression_domain_selector_projection,
    summarize_omena_query_source_diagnostics_for_file,
    summarize_omena_query_source_diagnostics_for_workspace_file,
    summarize_omena_query_style_completion_at_position,
    summarize_omena_query_style_diagnostics_for_file,
    summarize_omena_query_style_diagnostics_for_workspace_file,
    summarize_omena_query_style_document, summarize_omena_query_style_hover_candidates,
    summarize_omena_query_transform_context_from_engine_input,
};
#[cfg(feature = "zk-audit")]
use omena_zk_audit::{
    CascadeZKAuditV0, ZKAuditCiMatrixV0, cascade_zk_audit_v0, zk_audit_ci_matrix_v0,
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
    /// Estimate an MDL minimum-description summary for a style source.
    #[cfg(feature = "mdl")]
    Compress {
        /// CSS, SCSS, Sass, Less, or CSS Modules file to summarize.
        path: PathBuf,
        /// Fail when the estimated description length exceeds this bit budget.
        #[arg(long)]
        budget_bits: Option<f64>,
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
        /// Additional workspace style source used to resolve CSS Modules imports.
        #[arg(long = "source")]
        source_paths: Vec<PathBuf>,
        /// Additional source document used to resolve selector usage.
        #[arg(long = "source-document")]
        source_document_paths: Vec<PathBuf>,
        /// package.json file used to resolve package style exports for workspace sources.
        #[arg(long = "package-manifest")]
        package_manifest_paths: Vec<PathBuf>,
        /// Print machine-readable JSON.
        #[arg(long)]
        json: bool,
    },
    /// Read query-owned style hover candidates for a CSS-family file.
    StyleHoverCandidates {
        /// CSS, SCSS, Sass, Less, or CSS Modules file to inspect.
        path: PathBuf,
        /// Print machine-readable JSON.
        #[arg(long)]
        json: bool,
    },
    /// Read query-owned style completions at a source position.
    StyleCompletion {
        /// CSS, SCSS, Sass, Less, or CSS Modules file to inspect.
        path: PathBuf,
        /// Zero-based line number.
        #[arg(long)]
        line: usize,
        /// Zero-based UTF-16-like character offset used by the query protocol.
        #[arg(long)]
        character: usize,
        /// Print machine-readable JSON.
        #[arg(long)]
        json: bool,
    },
    /// Read query-owned source diagnostics from precomputed missing-selector candidates.
    SourceDiagnostics {
        /// Source document URI used in the diagnostics result.
        source_uri: String,
        /// JSON file containing source missing-selector diagnostic candidates.
        #[arg(long)]
        candidates_json: Option<PathBuf>,
        /// Source document path used to compute workspace diagnostics directly.
        #[arg(long)]
        source_path: Option<PathBuf>,
        /// Workspace style source used to resolve CSS Module selectors.
        #[arg(long = "source")]
        source_paths: Vec<PathBuf>,
        /// package.json file used to resolve package style exports for workspace sources.
        #[arg(long = "package-manifest")]
        package_manifest_paths: Vec<PathBuf>,
        /// Print machine-readable JSON.
        #[arg(long)]
        json: bool,
    },
    /// Emit the #72 downstream perceptual-check JSON scaffold from Omena facts.
    PerceptualCheck {
        /// CSS, SCSS, Sass, Less, or CSS Modules file to inspect.
        path: PathBuf,
        /// Print machine-readable JSON.
        #[arg(long)]
        json: bool,
    },
    /// Run feature-gated audit surfaces.
    #[cfg(feature = "zk-audit")]
    Audit {
        #[command(subcommand)]
        command: AuditCommand,
    },
}

#[cfg(feature = "zk-audit")]
#[derive(Debug, Subcommand)]
enum AuditCommand {
    /// Run zero-knowledge cascade audit commands.
    Zk {
        #[command(subcommand)]
        command: ZkAuditCommand,
    },
}

#[cfg(feature = "zk-audit")]
#[derive(Debug, Subcommand)]
enum ZkAuditCommand {
    /// Produce a default-off ZK cascade audit contract.
    Prove {
        /// Stable audit identifier.
        #[arg(long, default_value = "cli-zk-audit")]
        audit_id: String,
        /// Print machine-readable JSON.
        #[arg(long)]
        json: bool,
    },
    /// Verify the default-off ZK cascade audit contract shape.
    Verify {
        /// Stable audit identifier.
        #[arg(long, default_value = "cli-zk-audit")]
        audit_id: String,
        /// Print machine-readable JSON.
        #[arg(long)]
        json: bool,
    },
    /// Report the default Halo2+IPA setup status and opt-in CI cells.
    SetupStatus {
        /// Print machine-readable JSON.
        #[arg(long)]
        json: bool,
    },
}

#[cfg(feature = "zk-audit")]
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct ZkAuditCliResultV0 {
    schema_version: &'static str,
    product: &'static str,
    layer_marker: &'static str,
    feature_gate: &'static str,
    command: &'static str,
    audit: Option<CascadeZKAuditV0>,
    ci_matrix: Option<ZKAuditCiMatrixV0>,
    verified: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
struct PerceptualCheckCliReportV0 {
    schema_version: &'static str,
    product: &'static str,
    command: &'static str,
    claim_level: &'static str,
    style_path: String,
    language: &'static str,
    fact_source_products: Vec<&'static str>,
    selector_count: usize,
    custom_property_declaration_count: usize,
    custom_property_reference_count: usize,
    diagnostic_count: usize,
    color_machinery_source: &'static str,
    json_schema_ready: bool,
    downstream_tool_scaffold_ready: bool,
    consumes_omena_facts: bool,
    wcag_algorithm_ready: bool,
    apca_algorithm_ready: bool,
    oklab_perceptual_operator_ready: bool,
    full_perceptual_algorithm_ready: bool,
    public_safety_claim_ready: bool,
    supported_claims: Vec<&'static str>,
    deferred_claims: Vec<&'static str>,
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
        #[cfg(feature = "mdl")]
        Command::Compress {
            path,
            budget_bits,
            json,
        } => compress_file(path, budget_bits, json),
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
        Command::StyleDiagnostics {
            path,
            source_paths,
            source_document_paths,
            package_manifest_paths,
            json,
        } => style_diagnostics(
            path,
            source_paths,
            source_document_paths,
            package_manifest_paths,
            json,
        ),
        Command::StyleHoverCandidates { path, json } => style_hover_candidates(path, json),
        Command::StyleCompletion {
            path,
            line,
            character,
            json,
        } => style_completion(path, line, character, json),
        Command::SourceDiagnostics {
            source_uri,
            candidates_json,
            source_path,
            source_paths,
            package_manifest_paths,
            json,
        } => source_diagnostics(
            source_uri,
            candidates_json,
            source_path,
            source_paths,
            package_manifest_paths,
            json,
        ),
        Command::PerceptualCheck { path, json } => perceptual_check(path, json),
        #[cfg(feature = "zk-audit")]
        Command::Audit { command } => audit_command(command),
    }
}

#[cfg(feature = "zk-audit")]
fn audit_command(command: AuditCommand) -> Result<(), String> {
    match command {
        AuditCommand::Zk { command } => zk_audit_command(command),
    }
}

#[cfg(feature = "zk-audit")]
fn zk_audit_command(command: ZkAuditCommand) -> Result<(), String> {
    match command {
        ZkAuditCommand::Prove { audit_id, json } => {
            let result = ZkAuditCliResultV0 {
                schema_version: "0",
                product: "omena-cli.audit.zk.prove",
                layer_marker: "cryptographic-implementation",
                feature_gate: "zk-audit",
                command: "prove",
                audit: Some(cascade_zk_audit_v0(audit_id)),
                ci_matrix: None,
                verified: true,
            };
            print_zk_audit_result(&result, json)
        }
        ZkAuditCommand::Verify { audit_id, json } => {
            let audit = cascade_zk_audit_v0(audit_id);
            let verified = audit.schema_version == "0"
                && audit.feature_gate == "zk-audit"
                && audit.recursion_overhead == "O(1)";
            let result = ZkAuditCliResultV0 {
                schema_version: "0",
                product: "omena-cli.audit.zk.verify",
                layer_marker: "cryptographic-implementation",
                feature_gate: "zk-audit",
                command: "verify",
                audit: Some(audit),
                ci_matrix: None,
                verified,
            };
            print_zk_audit_result(&result, json)
        }
        ZkAuditCommand::SetupStatus { json } => {
            let result = ZkAuditCliResultV0 {
                schema_version: "0",
                product: "omena-cli.audit.zk.setup-status",
                layer_marker: "cryptographic-implementation",
                feature_gate: "zk-audit",
                command: "setup-status",
                audit: None,
                ci_matrix: Some(zk_audit_ci_matrix_v0()),
                verified: true,
            };
            print_zk_audit_result(&result, json)
        }
    }
}

#[cfg(feature = "zk-audit")]
fn print_zk_audit_result(result: &ZkAuditCliResultV0, json: bool) -> Result<(), String> {
    if json {
        print_json(result)?;
        return Ok(());
    }

    println!("product: {}", result.product);
    println!("command: {}", result.command);
    println!("feature gate: {}", result.feature_gate);
    println!("verified: {}", result.verified);
    if let Some(audit) = &result.audit {
        println!("audit: {}", audit.audit_id);
        println!("setup: {:?}", audit.setup_kind);
        println!("recursion overhead: {}", audit.recursion_overhead);
    }
    if let Some(ci_matrix) = &result.ci_matrix {
        println!("ci cells: {}", ci_matrix.cells.join(","));
        println!(
            "heavy dependencies default off: {}",
            ci_matrix.heavy_dependencies_default_off
        );
    }
    Ok(())
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

fn read_source_documents(
    source_document_paths: &[PathBuf],
) -> Result<Vec<OmenaQuerySourceDocumentInputV0>, String> {
    source_document_paths
        .iter()
        .map(|path| {
            Ok(OmenaQuerySourceDocumentInputV0 {
                source_path: path_string(path),
                source_source: read_source(path)?,
            })
        })
        .collect()
}

fn read_style_sources(
    source_paths: &[PathBuf],
) -> Result<Vec<OmenaQueryStyleSourceInputV0>, String> {
    source_paths
        .iter()
        .map(|path| {
            Ok(OmenaQueryStyleSourceInputV0 {
                style_path: path_string(path),
                style_source: read_source(path)?,
            })
        })
        .collect()
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

#[cfg(feature = "mdl")]
fn compress_file(path: PathBuf, budget_bits: Option<f64>, json: bool) -> Result<(), String> {
    let source = read_source(&path)?;
    let rule_count = source.matches('{').count();
    let observation_count = source
        .lines()
        .filter(|line| !line.trim().is_empty())
        .count();
    let source_hash = format!(
        "len{}-sum{}",
        source.len(),
        source.bytes().map(u64::from).sum::<u64>()
    );
    let summary = summarize_omena_query_design_system_minimum_description(
        path_string(&path),
        source_hash,
        rule_count,
        observation_count,
    );
    if let Some(budget_bits) = budget_bits
        && summary.total_bits > budget_bits
    {
        return Err(format!(
            "MDL budget exceeded: total_bits={} budget_bits={budget_bits}",
            summary.total_bits
        ));
    }

    if json {
        print_json(&summary)?;
        return Ok(());
    }

    println!("total bits: {}", summary.total_bits);
    println!("model bits: {}", summary.model_bits);
    println!("residual bits: {}", summary.residual_bits);
    println!("unit: {}", summary.unit);
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

fn style_diagnostics(
    path: PathBuf,
    source_paths: Vec<PathBuf>,
    source_document_paths: Vec<PathBuf>,
    package_manifest_paths: Vec<PathBuf>,
    json: bool,
) -> Result<(), String> {
    let source = read_source(&path)?;
    let style_path = path_string(&path);
    let package_manifests = read_package_manifests(&package_manifest_paths)?;
    let summary = if source_paths.is_empty()
        && source_document_paths.is_empty()
        && package_manifests.is_empty()
    {
        let Some(candidates) = summarize_omena_query_style_hover_candidates(&style_path, &source)
        else {
            return Err(format!(
                "failed to read style candidates for {}",
                path_string(&path)
            ));
        };
        summarize_omena_query_style_diagnostics_for_file(
            &style_path,
            &source,
            candidates.candidates.as_slice(),
        )
    } else {
        let workspace_sources = read_workspace_sources(&path, &source, &source_paths)?;
        let source_documents = read_source_documents(&source_document_paths)?;
        summarize_omena_query_style_diagnostics_for_workspace_file(
            &style_path,
            workspace_sources.as_slice(),
            source_documents.as_slice(),
            package_manifests.as_slice(),
            None,
        )
        .ok_or_else(|| format!("failed to read workspace style diagnostics for {style_path}"))?
    };

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

fn style_hover_candidates(path: PathBuf, json: bool) -> Result<(), String> {
    let source = read_source(&path)?;
    let style_path = path_string(&path);
    let Some(summary) = summarize_omena_query_style_hover_candidates(&style_path, &source) else {
        return Err(format!(
            "failed to read style candidates for {}",
            path_string(&path)
        ));
    };

    if json {
        print_json(&summary)?;
        return Ok(());
    }

    println!("file: {style_path}");
    println!("language: {}", summary.language);
    println!("candidates: {}", summary.candidates.len());
    for candidate in &summary.candidates {
        println!(
            "{}\t{}\t{}",
            candidate.kind, candidate.name, candidate.source
        );
    }
    Ok(())
}

fn style_completion(
    path: PathBuf,
    line: usize,
    character: usize,
    json: bool,
) -> Result<(), String> {
    let source = read_source(&path)?;
    let style_path = path_string(&path);
    let Some(candidates) = summarize_omena_query_style_hover_candidates(&style_path, &source)
    else {
        return Err(format!(
            "failed to read style candidates for {}",
            path_string(&path)
        ));
    };
    let summary = summarize_omena_query_style_completion_at_position(
        &style_path,
        &source,
        ParserPositionV0 { line, character },
        candidates.candidates.as_slice(),
    );

    if json {
        print_json(&summary)?;
        return Ok(());
    }

    println!("file: {}", summary.file_uri);
    println!("context: {}", summary.context_kind);
    println!("items: {}", summary.item_count);
    for item in &summary.items {
        println!("{}\t{}\t{}", item.label, item.detail, item.source);
    }
    Ok(())
}

fn source_diagnostics(
    source_uri: String,
    candidates_json: Option<PathBuf>,
    source_path: Option<PathBuf>,
    source_paths: Vec<PathBuf>,
    package_manifest_paths: Vec<PathBuf>,
    json: bool,
) -> Result<(), String> {
    let summary = source_diagnostics_summary(
        source_uri,
        candidates_json,
        source_path,
        source_paths,
        package_manifest_paths,
    )?;

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

fn perceptual_check(path: PathBuf, json: bool) -> Result<(), String> {
    let report = perceptual_check_summary(&path)?;

    if json {
        print_json(&report)?;
        return Ok(());
    }

    println!("product: {}", report.product);
    println!("file: {}", report.style_path);
    println!("language: {}", report.language);
    println!("claim level: {}", report.claim_level);
    println!("selectors: {}", report.selector_count);
    println!(
        "custom property declarations: {}",
        report.custom_property_declaration_count
    );
    println!(
        "custom property references: {}",
        report.custom_property_reference_count
    );
    println!("diagnostics: {}", report.diagnostic_count);
    println!(
        "downstream scaffold ready: {}",
        report.downstream_tool_scaffold_ready
    );
    println!(
        "full perceptual algorithm ready: {}",
        report.full_perceptual_algorithm_ready
    );
    Ok(())
}

fn perceptual_check_summary(path: &Path) -> Result<PerceptualCheckCliReportV0, String> {
    let source = read_source(path)?;
    let style_path = path_string(path);
    let style_document = summarize_omena_query_style_document(&style_path, &source)
        .ok_or_else(|| format!("failed to read style document facts for {style_path}"))?;
    let check = summarize_omena_query_consumer_check_style_source(&style_path, &source);

    Ok(PerceptualCheckCliReportV0 {
        schema_version: "0",
        product: "omena-cli.perceptual-check.scaffold",
        command: "perceptual-check",
        claim_level: "m6DownstreamToolScaffoldOnly",
        style_path,
        language: style_document.language,
        fact_source_products: vec![style_document.product, check.product],
        selector_count: style_document.selector_names.len(),
        custom_property_declaration_count: style_document.custom_property_decl_names.len(),
        custom_property_reference_count: style_document.custom_property_ref_names.len(),
        diagnostic_count: style_document
            .diagnostic_count
            .max(check.parser_error_count),
        color_machinery_source: "omena-transform-passes.domains.color",
        json_schema_ready: true,
        downstream_tool_scaffold_ready: true,
        consumes_omena_facts: true,
        wcag_algorithm_ready: false,
        apca_algorithm_ready: false,
        oklab_perceptual_operator_ready: false,
        full_perceptual_algorithm_ready: false,
        public_safety_claim_ready: false,
        supported_claims: vec![
            "perceptual-check CLI scaffold",
            "JSON output schema",
            "Omena fact-level input consumption",
        ],
        deferred_claims: vec![
            "WCAG luminance algorithm",
            "APCA algorithm",
            "OKLab perceptual operator",
            "full perceptual algorithm",
            "public safety claim",
        ],
    })
}

fn source_diagnostics_summary(
    source_uri: String,
    candidates_json: Option<PathBuf>,
    source_path: Option<PathBuf>,
    source_paths: Vec<PathBuf>,
    package_manifest_paths: Vec<PathBuf>,
) -> Result<OmenaQuerySourceDiagnosticsForFileV0, String> {
    if let Some(candidates_json) = candidates_json {
        let candidates = read_source_diagnostic_candidates_json(&candidates_json)?;
        Ok(summarize_omena_query_source_diagnostics_for_file(
            source_uri.as_str(),
            candidates.as_slice(),
        ))
    } else {
        let source_path = source_path.ok_or_else(|| {
            "source-diagnostics requires either --candidates-json or --source-path".to_string()
        })?;
        let source_source = read_source(&source_path)?;
        let style_sources = read_style_sources(&source_paths)?;
        let package_manifests = read_package_manifests(&package_manifest_paths)?;
        Ok(summarize_omena_query_source_diagnostics_for_workspace_file(
            source_uri.as_str(),
            source_source.as_str(),
            style_sources.as_slice(),
            package_manifests.as_slice(),
        ))
    }
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

fn read_source_diagnostic_candidates_json(
    path: &Path,
) -> Result<Vec<OmenaQuerySourceMissingSelectorDiagnosticCandidateV0>, String> {
    let json = fs::read_to_string(path).map_err(|error| {
        format!(
            "failed to read source diagnostics candidates JSON {}: {error}",
            path_string(path)
        )
    })?;
    serde_json::from_str(&json).map_err(|error| {
        format!(
            "failed to parse source diagnostics candidates JSON {}: {error}",
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
    use clap::CommandFactory;
    use std::time::{SystemTime, UNIX_EPOCH};

    #[test]
    fn build_command_writes_query_owned_transform_output() -> Result<(), String> {
        let source_path = temp_path("input.css");
        let output_path = temp_path("output.css");
        fs::write(&source_path, ".card { color: #ffffff; }")
            .map_err(|error| format!("fixture source should be writable: {error}"))?;

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
        let output = fs::read_to_string(&output_path)
            .map_err(|error| format!("build output should be written: {error}"))?;
        assert!(output.contains("#fff"));
        assert!(!output.contains("#ffffff"));

        cleanup(&source_path);
        cleanup(&output_path);
        Ok(())
    }

    #[cfg(feature = "mdl")]
    #[test]
    fn compress_command_enforces_budget_bits() -> Result<(), String> {
        let source_path = temp_path("compress.css");
        fs::write(&source_path, ".card { color: red; }\n")
            .map_err(|error| format!("fixture source should be writable: {error}"))?;

        let result = run(Cli {
            command: Command::Compress {
                path: source_path.clone(),
                budget_bits: Some(1.0),
                json: true,
            },
        });

        assert!(result.is_err(), "{result:?}");

        cleanup(&source_path);
        Ok(())
    }

    #[test]
    fn cascade_and_context_index_commands_read_query_surfaces() -> Result<(), String> {
        let source_path = temp_path("input.module.css");
        fs::write(
            &source_path,
            "@layer components { :root { --brand: #2563eb; } }\n.button { color: var(--brand); }\n",
        )
        .map_err(|error| format!("fixture source should be writable: {error}"))?;

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
        Ok(())
    }

    #[test]
    fn style_diagnostics_command_reads_query_owned_diagnostics() -> Result<(), String> {
        let source_path = temp_path("diagnostics.module.css");
        fs::write(
            &source_path,
            ":root { --known: #2563eb; }\n.button { color: var(--missing); animation: fade 1s; }\n",
        )
        .map_err(|error| format!("fixture source should be writable: {error}"))?;

        let result = run(Cli {
            command: Command::StyleDiagnostics {
                path: source_path.clone(),
                source_paths: Vec::new(),
                source_document_paths: Vec::new(),
                package_manifest_paths: Vec::new(),
                json: true,
            },
        });

        assert!(result.is_ok(), "{result:?}");

        cleanup(&source_path);
        Ok(())
    }

    #[test]
    fn style_hover_and_completion_commands_read_query_owned_surfaces() -> Result<(), String> {
        let source_path = temp_path("hover.module.css");
        fs::write(
            &source_path,
            ":root { --brand: #2563eb; }\n.button { color: var(--); }\n",
        )
        .map_err(|error| format!("fixture source should be writable: {error}"))?;

        let hover_result = run(Cli {
            command: Command::StyleHoverCandidates {
                path: source_path.clone(),
                json: true,
            },
        });
        assert!(hover_result.is_ok(), "{hover_result:?}");

        let completion_result = run(Cli {
            command: Command::StyleCompletion {
                path: source_path.clone(),
                line: 1,
                character: 23,
                json: true,
            },
        });
        assert!(completion_result.is_ok(), "{completion_result:?}");

        cleanup(&source_path);
        Ok(())
    }

    #[test]
    fn source_diagnostics_command_reads_query_owned_diagnostics() -> Result<(), String> {
        let candidates_path = temp_path("source-diagnostics.json");
        fs::write(
            &candidates_path,
            r#"[
              {
                "targetStyleUri": "file:///workspace/src/App.module.css",
                "targetStyleSource": ".root {\n}\n",
                "selectorName": "missing",
                "sourceReferenceRange": {
                  "start": { "line": 2, "character": 18 },
                  "end": { "line": 2, "character": 25 }
                }
              }
            ]"#,
        )
        .map_err(|error| format!("fixture candidates should be writable: {error}"))?;

        let result = run(Cli {
            command: Command::SourceDiagnostics {
                source_uri: "file:///workspace/src/App.tsx".to_string(),
                candidates_json: Some(candidates_path.clone()),
                source_path: None,
                source_paths: Vec::new(),
                package_manifest_paths: Vec::new(),
                json: true,
            },
        });

        assert!(result.is_ok(), "{result:?}");

        cleanup(&candidates_path);
        Ok(())
    }

    #[test]
    fn source_diagnostics_command_reads_workspace_query_owned_diagnostics() -> Result<(), String> {
        let source_path = temp_path("App.tsx");
        let style_path = temp_path("App.module.scss");
        fs::write(
            &source_path,
            r#"import bind from "classnames/bind";
import styles from "./App.module.scss";
const cx = bind.bind(styles);
const variant = Math.random() > 0.5 ? "chip" : "ghost";
export function App() {
  return <div className={cx(variant)} />;
}
"#,
        )
        .map_err(|error| format!("fixture source should be writable: {error}"))?;
        fs::write(&style_path, ".chip {}\n")
            .map_err(|error| format!("fixture style should be writable: {error}"))?;

        let result = run(Cli {
            command: Command::SourceDiagnostics {
                source_uri: path_string(&source_path),
                candidates_json: None,
                source_path: Some(source_path.clone()),
                source_paths: vec![style_path.clone()],
                package_manifest_paths: Vec::new(),
                json: true,
            },
        });

        assert!(result.is_ok(), "{result:?}");

        cleanup(&source_path);
        cleanup(&style_path);
        Ok(())
    }

    #[test]
    fn source_diagnostics_command_uses_package_manifest_override_paths() -> Result<(), String> {
        let workspace_path = temp_dir("package-manifest-override");
        let source_dir = workspace_path.join("src");
        let package_dir = workspace_path.join("node_modules/@design/tokens");
        let style_dir = package_dir.join("dist");
        fs::create_dir_all(&source_dir)
            .map_err(|error| format!("fixture source dir should be writable: {error}"))?;
        fs::create_dir_all(&style_dir)
            .map_err(|error| format!("fixture package dir should be writable: {error}"))?;

        let source_path = source_dir.join("App.tsx");
        let style_path = style_dir.join("theme.module.css");
        let package_manifest_path = package_dir.join("package.json");
        fs::write(
            &source_path,
            r#"import bind from "classnames/bind";
import styles from "@design/tokens/theme.module.css";
const cx = bind.bind(styles);
export function App() {
  return <div className={cx("ghost")} />;
}
"#,
        )
        .map_err(|error| format!("fixture source should be writable: {error}"))?;
        fs::write(&style_path, ".chip {}\n")
            .map_err(|error| format!("fixture style should be writable: {error}"))?;
        fs::write(
            &package_manifest_path,
            r#"{"exports":{"./theme.module.css":{"style":"./dist/theme.module.css"}}}"#,
        )
        .map_err(|error| format!("fixture package manifest should be writable: {error}"))?;

        let summary = source_diagnostics_summary(
            path_string(&source_path),
            None,
            Some(source_path.clone()),
            vec![style_path.clone()],
            vec![package_manifest_path.clone()],
        )?;

        assert!(
            summary
                .diagnostics
                .iter()
                .all(|diagnostic| diagnostic.code != "missingModule"),
            "{summary:?}"
        );
        let diagnostic = summary
            .diagnostics
            .iter()
            .find(|diagnostic| diagnostic.code == "missingStaticClass")
            .ok_or_else(|| format!("expected missingStaticClass diagnostic: {summary:?}"))?;
        let create_selector = diagnostic
            .create_selector
            .as_ref()
            .ok_or_else(|| format!("expected create selector action: {diagnostic:?}"))?;
        assert_eq!(create_selector.uri, path_string(&style_path));
        assert_eq!(create_selector.selector_name, "ghost");

        let result = run(Cli {
            command: Command::SourceDiagnostics {
                source_uri: path_string(&source_path),
                candidates_json: None,
                source_path: Some(source_path.clone()),
                source_paths: vec![style_path],
                package_manifest_paths: vec![package_manifest_path],
                json: true,
            },
        });
        assert!(result.is_ok(), "{result:?}");

        cleanup_dir(&workspace_path);
        Ok(())
    }

    #[test]
    fn source_diagnostics_command_resolves_package_import_array_fallbacks() -> Result<(), String> {
        let workspace_path = temp_dir("package-import-array-fallback");
        let source_dir = workspace_path.join("src");
        fs::create_dir_all(&source_dir)
            .map_err(|error| format!("fixture source dir should be writable: {error}"))?;

        let source_path = source_dir.join("App.tsx");
        let style_path = source_dir.join("theme.module.css");
        let package_manifest_path = workspace_path.join("package.json");
        fs::write(
            &source_path,
            r##"import bind from "classnames/bind";
import styles from "#theme.module.css";
const cx = bind.bind(styles);
export function App() {
  return <div className={cx("ghost")} />;
}
"##,
        )
        .map_err(|error| format!("fixture source should be writable: {error}"))?;
        fs::write(&style_path, ".chip {}\n")
            .map_err(|error| format!("fixture style should be writable: {error}"))?;
        fs::write(
            &package_manifest_path,
            r##"{"imports":{"#theme.module.css":[{"node":"./src/theme.js"},{"style":"./src/theme.module.css"}]}}"##,
        )
        .map_err(|error| format!("fixture package manifest should be writable: {error}"))?;

        let summary = source_diagnostics_summary(
            path_string(&source_path),
            None,
            Some(source_path.clone()),
            vec![style_path.clone()],
            vec![package_manifest_path.clone()],
        )?;

        assert!(
            summary
                .diagnostics
                .iter()
                .all(|diagnostic| diagnostic.code != "missingModule"),
            "{summary:?}"
        );
        let diagnostic = summary
            .diagnostics
            .iter()
            .find(|diagnostic| diagnostic.code == "missingStaticClass")
            .ok_or_else(|| format!("expected missingStaticClass diagnostic: {summary:?}"))?;
        let create_selector = diagnostic
            .create_selector
            .as_ref()
            .ok_or_else(|| format!("expected create selector action: {diagnostic:?}"))?;
        assert_eq!(create_selector.uri, path_string(&style_path));
        assert_eq!(create_selector.selector_name, "ghost");

        let result = run(Cli {
            command: Command::SourceDiagnostics {
                source_uri: path_string(&source_path),
                candidates_json: None,
                source_path: Some(source_path.clone()),
                source_paths: vec![style_path],
                package_manifest_paths: vec![package_manifest_path],
                json: true,
            },
        });
        assert!(result.is_ok(), "{result:?}");

        cleanup_dir(&workspace_path);
        Ok(())
    }

    #[test]
    fn perceptual_check_command_emits_scaffold_schema_from_query_facts() -> Result<(), String> {
        let source_path = temp_path("perceptual.module.css");
        fs::write(
            &source_path,
            ":root { --fg: #000; }\n.button { color: var(--fg); background: #fff; }\n",
        )
        .map_err(|error| format!("fixture source should be writable: {error}"))?;

        let report = perceptual_check_summary(&source_path)?;
        assert_eq!(report.product, "omena-cli.perceptual-check.scaffold");
        assert_eq!(report.claim_level, "m6DownstreamToolScaffoldOnly");
        assert_eq!(report.command, "perceptual-check");
        assert!(report.json_schema_ready);
        assert!(report.downstream_tool_scaffold_ready);
        assert!(report.consumes_omena_facts);
        assert_eq!(report.selector_count, 1);
        assert_eq!(report.custom_property_declaration_count, 1);
        assert_eq!(report.custom_property_reference_count, 1);
        assert!(!report.wcag_algorithm_ready);
        assert!(!report.apca_algorithm_ready);
        assert!(!report.oklab_perceptual_operator_ready);
        assert!(!report.full_perceptual_algorithm_ready);
        assert!(!report.public_safety_claim_ready);
        assert!(
            report
                .fact_source_products
                .contains(&"omena-query.style-document-summary")
        );
        assert!(
            report
                .fact_source_products
                .contains(&"omena-query.consumer-check-style-source")
        );

        let result = run(Cli {
            command: Command::PerceptualCheck {
                path: source_path.clone(),
                json: true,
            },
        });
        assert!(result.is_ok(), "{result:?}");

        cleanup(&source_path);
        Ok(())
    }

    #[test]
    fn perceptual_check_help_is_available() {
        let help = Cli::command().render_long_help().to_string();
        assert!(help.contains("perceptual-check"));
        assert!(help.contains("downstream perceptual-check JSON scaffold"));
    }

    #[cfg(feature = "zk-audit")]
    #[test]
    fn audit_zk_commands_are_feature_gated_surfaces() {
        let prove_result = run(Cli {
            command: Command::Audit {
                command: AuditCommand::Zk {
                    command: ZkAuditCommand::Prove {
                        audit_id: "test-audit".to_string(),
                        json: true,
                    },
                },
            },
        });
        assert!(prove_result.is_ok(), "{prove_result:?}");

        let verify_result = run(Cli {
            command: Command::Audit {
                command: AuditCommand::Zk {
                    command: ZkAuditCommand::Verify {
                        audit_id: "test-audit".to_string(),
                        json: true,
                    },
                },
            },
        });
        assert!(verify_result.is_ok(), "{verify_result:?}");

        let setup_result = run(Cli {
            command: Command::Audit {
                command: AuditCommand::Zk {
                    command: ZkAuditCommand::SetupStatus { json: true },
                },
            },
        });
        assert!(setup_result.is_ok(), "{setup_result:?}");
    }

    fn temp_path(name: &str) -> PathBuf {
        let nanos = match SystemTime::now().duration_since(UNIX_EPOCH) {
            Ok(duration) => duration.as_nanos(),
            Err(_) => 0,
        };
        std::env::temp_dir().join(format!("omena-cli-{nanos}-{name}"))
    }

    fn temp_dir(name: &str) -> PathBuf {
        temp_path(name)
    }

    fn cleanup(path: &Path) {
        let _ = fs::remove_file(path);
    }

    fn cleanup_dir(path: &Path) {
        let _ = fs::remove_dir_all(path);
    }
}

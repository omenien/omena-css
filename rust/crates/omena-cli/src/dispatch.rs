use omena_query::OmenaQueryTargetTransformOptionsV0;

use crate::{
    build::{BuildFileOptions, build_file, list_passes},
    commands::{Cli, Command},
    diagnostics::{dynamic_classname_diagnostics, source_diagnostics, style_diagnostics},
    facts::facts_file,
    lint::lint_workspace,
    lock::lock_command,
    perceptual::perceptual_check,
    product_verb::{CliExit, ProductVerb},
    provenance::provenance_command,
    query::{
        cascade_at_position, context_from_engine_input, context_index, expression_flow,
        selector_projection, style_completion, style_hover_candidates,
    },
    reports::report_command,
    sif::sif_command,
};

#[cfg(feature = "mdl")]
use crate::mdl::compress_file;

#[cfg(feature = "zk-audit")]
use crate::audit::audit_command;

#[cfg(test)]
pub(crate) fn run(cli: Cli) -> Result<(), String> {
    run_with_exit(cli).map_err(|error| error.to_string())
}

pub(crate) fn run_with_exit(cli: Cli) -> Result<(), CliExit> {
    let result = match cli.command {
        Command::Check { path, write, json } => {
            return run_reserved_facts_alias(path, write, json);
        }
        Command::Facts { path, json } => facts_file(path, json),
        Command::Lint {
            root,
            profile,
            stylelint_config,
            write,
            json,
        } => lint_workspace(root, profile, stylelint_config, write, json),
        Command::Fmt { .. } => return Err(CliExit::not_yet_wired(ProductVerb::Fmt)),
        Command::Minify { .. } => return Err(CliExit::not_yet_wired(ProductVerb::Minify)),
        Command::Bundle { .. } => return Err(CliExit::not_yet_wired(ProductVerb::Bundle)),
        Command::Modules => return Err(CliExit::not_yet_wired(ProductVerb::Modules)),
        Command::Sass => return Err(CliExit::not_yet_wired(ProductVerb::Sass)),
        Command::Intel => return Err(CliExit::not_yet_wired(ProductVerb::Intel)),
        Command::Migrate { .. } => return Err(CliExit::not_yet_wired(ProductVerb::Migrate)),
        Command::Verify => return Err(CliExit::not_yet_wired(ProductVerb::Verify)),
        Command::Ci => return Err(CliExit::not_yet_wired(ProductVerb::Ci)),
        Command::Explain { .. } => return Err(CliExit::not_yet_wired(ProductVerb::Explain)),
        Command::Build {
            path,
            output,
            passes,
            minify,
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
            tree_shake,
            bundle,
            split_out_dir,
            bundle_entry_paths,
            source_paths,
            package_manifest_paths,
            source_map,
            input_source_maps,
            json,
        } => build_file(BuildFileOptions {
            path,
            output,
            pass_ids: passes,
            minify,
            target_query,
            context_json,
            engine_input_json,
            closed_style_world,
            tree_shake,
            bundle,
            split_out_dir,
            bundle_entry_paths,
            source_paths,
            package_manifest_paths,
            source_map,
            input_source_maps,
            target_options: OmenaQueryTargetTransformOptionsV0 {
                allow_logical_to_physical,
                allow_scope_flatten,
                allow_layer_flatten,
                enable_supports_static_eval,
                enable_media_static_eval,
                enable_container_static_eval: false,
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
            categorical_evidence,
            json,
        } => cascade_at_position(
            path,
            line,
            character,
            engine_input_json,
            categorical_evidence,
            json,
        ),
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
            sif_paths,
            lockfile,
            external,
            deep_analysis,
            json,
        } => style_diagnostics(
            path,
            source_paths,
            source_document_paths,
            package_manifest_paths,
            sif_paths,
            lockfile,
            external,
            deep_analysis,
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
        Command::DynamicClassnameDiagnostics { input_json, json } => {
            dynamic_classname_diagnostics(input_json, json)
        }
        Command::PerceptualCheck { path, json } => perceptual_check(path, json),
        Command::Lock {
            lockfile,
            json,
            command,
        } => lock_command(lockfile, json, command),
        Command::Sif { command } => sif_command(command),
        Command::Provenance { command } => provenance_command(command),
        Command::Report { command } => report_command(command),
        #[cfg(feature = "zk-audit")]
        Command::Audit { command } => audit_command(command),
    };
    result.map_err(CliExit::failure)
}

fn run_reserved_facts_alias(
    path: Option<std::path::PathBuf>,
    write: bool,
    json: bool,
) -> Result<(), CliExit> {
    let Some(path) = path else {
        return Err(CliExit::not_yet_wired(ProductVerb::Check));
    };
    if write {
        return Err(CliExit::not_yet_wired(ProductVerb::Check));
    }
    eprintln!("warning: `omena check <file>` is deprecated; use `omena facts <file>`");
    facts_file(path, json).map_err(CliExit::failure)
}

use omena_query::OmenaQueryTargetTransformOptionsV0;

use crate::{
    build::{BuildFileOptions, build_file, list_passes},
    check_file,
    commands::{Cli, Command},
    diagnostics::{dynamic_classname_diagnostics, source_diagnostics, style_diagnostics},
    lock::lock_command,
    perceptual::perceptual_check,
    provenance::provenance_command,
    query::{
        cascade_at_position, context_from_engine_input, context_index, expression_flow,
        selector_projection, style_completion, style_hover_candidates,
    },
    reports::report_command,
    sif::sif_command,
};

#[cfg(feature = "mdl")]
use crate::compress_file;

#[cfg(feature = "zk-audit")]
use crate::audit::audit_command;

pub(crate) fn run(cli: Cli) -> Result<(), String> {
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
    }
}

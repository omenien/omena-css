use clap::Parser;
use std::process::ExitCode;

#[cfg(feature = "zk-audit")]
mod audit;
mod build;
mod bundle;
mod ci;
mod commands;
mod config;
mod diagnostics;
mod dispatch;
mod explain;
mod facts;
mod format;
mod intel;
mod io;
mod lint;
mod lock;
#[cfg(feature = "mdl")]
mod mdl;
mod migrate;
mod minify;
mod minify_backend;
mod modules;
mod output;
mod paths;
mod perceptual;
mod postcss_compat;
mod product_verb;
mod provenance;
mod query;
mod reports;
mod sass;
mod sif;
mod text_edit;
mod verification;
mod write_safety;

use commands::Cli;
use dispatch::run_with_exit;

fn main() -> ExitCode {
    match run_with_exit(Cli::parse()) {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("{error}");
            ExitCode::from(error.code())
        }
    }
}

#[cfg(test)]
mod tests;

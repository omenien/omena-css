use clap::Parser;
use std::process::ExitCode;

#[cfg(feature = "zk-audit")]
mod audit;
mod build;
mod commands;
mod config;
mod diagnostics;
mod dispatch;
mod facts;
mod format;
mod io;
mod lint;
mod lock;
#[cfg(feature = "mdl")]
mod mdl;
mod output;
mod paths;
mod perceptual;
mod product_verb;
mod provenance;
mod query;
mod reports;
mod sif;
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

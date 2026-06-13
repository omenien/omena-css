use clap::Parser;
use std::process::ExitCode;

#[cfg(feature = "zk-audit")]
mod audit;
mod build;
mod check;
mod commands;
mod diagnostics;
mod dispatch;
mod io;
mod lock;
#[cfg(feature = "mdl")]
mod mdl;
mod output;
mod paths;
mod perceptual;
mod provenance;
mod query;
mod reports;
mod sif;

use commands::Cli;
use dispatch::run;

fn main() -> ExitCode {
    match run(Cli::parse()) {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("{error}");
            ExitCode::FAILURE
        }
    }
}

#[cfg(test)]
mod tests;

use std::process::ExitCode;

fn main() -> ExitCode {
    omena_cli::daemon::run_omenad_from_env()
}

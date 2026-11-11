//! Smarter git workflow commands: `smart-pull`, `smart-merge` and `smart-log`.
//!
//! Each command explains what it decides and echoes every git command it runs.

pub mod commands;
pub mod error;
pub mod git;
pub mod ui;

use error::{Error, Result};
use git::Repo;
use std::io::{IsTerminal, Write};
use std::path::Path;
use std::process::ExitCode;
use ui::Ui;

pub const COMMANDS: [&str; 3] = ["smart-log", "smart-merge", "smart-pull"];

/// Runs `command` against the repository containing `dir`.
pub fn run<W: Write>(command: &str, args: &[String], dir: &Path, ui: &mut Ui<W>) -> Result<()> {
    if !COMMANDS.contains(&command) {
        return Err(error::failed(format!(
            "No command {command:?} defined! Available commands are {COMMANDS:?}"
        )));
    }
    let repo = Repo::discover(dir)?;
    match command {
        "smart-log" => commands::log::run(&repo, args, ui),
        "smart-merge" => commands::merge::run(&repo, args, ui),
        _ => commands::pull::run(&repo, args, ui),
    }
}

/// Entry point shared by the `git-smart-*` executables.
pub fn main(command: &str) -> ExitCode {
    let stdout = std::io::stdout();
    let color = stdout.is_terminal() && std::env::var_os("NO_COLOR").is_none();
    let mut ui = Ui::new(stdout.lock(), color);
    let args: Vec<String> = std::env::args().skip(1).collect();
    let dir = std::env::current_dir().unwrap_or_else(|_| ".".into());

    match run(command, &args, &dir, &mut ui) {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            let _ = report(&mut ui, &e);
            ExitCode::FAILURE
        }
    }
}

fn report<W: Write>(ui: &mut Ui<W>, e: &Error) -> std::io::Result<()> {
    match e {
        Error::Failed(msg) => ui.failure(msg),
        other => ui.warn(&other.to_string()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn unknown_command_is_rejected() {
        let mut ui = Ui::new(Vec::new(), false);
        let err = run("smart-nope", &[], Path::new("."), &mut ui).unwrap_err();
        assert!(err.to_string().contains("No command \"smart-nope\""));
    }

    #[test]
    fn outside_a_repo_is_a_meaningful_error() {
        let dir = std::env::temp_dir().join(format!("gsr-norepo-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let mut ui = Ui::new(Vec::new(), false);
        let err = run("smart-pull", &[], &dir, &mut ui).unwrap_err();
        std::fs::remove_dir_all(&dir).unwrap();
        let msg = err.to_string();
        assert!(msg.contains("You need to run this from within a Git directory"));
        assert!(msg.contains("Current working directory:"));
    }
}

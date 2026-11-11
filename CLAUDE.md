# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Overview

Rust port of the Ruby `git-smart` gem: three git subcommands (`smart-pull`, `smart-merge`, `smart-log`) installed as `git-smart-*` executables so git runs them as `git smart-pull` etc. Zero dependencies beyond `std` and the `git` executable (keep it that way). Edition 2024.

## Commands

- `cargo build` / `cargo install --path .` (installs the three binaries)
- `cargo test` — unit tests plus integration tests in `tests/`
- Single test: `cargo test --test smart_pull <test_name>` (or `cargo test <name>` for unit tests in `src/`)
- `cargo clippy` / `cargo fmt`

## Architecture

- `src/lib.rs` (crate `git_smart`) holds everything shared. Each `src/bin/git-smart-*.rs` is a 3-line shim calling `git_smart::main("<command>")`. `main` handles colour detection (TTY and no `NO_COLOR`), arg collection and exit codes; `run(command, args, dir, ui)` dispatches and is the testable entry point (takes an explicit directory and a generic `Ui<W: Write>`, so tests can capture output).
- Adding a command means: a module in `src/commands/`, an entry in `COMMANDS` and the `match` in `lib.rs`, and a `[[bin]]` in `Cargo.toml`.
- `src/git.rs`: `Repo` wraps the `git` executable (no libgit2). `Repo::run(ui, args)` echoes the command to the user before executing it — use it for anything that modifies the repo; `query` is for silent read-only lookups.
- `src/ui.rs`: all user-facing output (notes, warnings, failures, colour) goes through `Ui`; don't `println!` directly.
- `src/error.rs`: `Error::Failed(msg)` is a user-facing failure (printed via `ui.failure`); other variants are reported as warnings. Create failures with `error::failed(...)`.
- `src/commands/mod.rs`: `with_stash` stashes dirty tracked changes around an operation and pops afterwards; on failure it deliberately leaves the stash and tells the user how to recover. Used by pull and merge.

## Testing

Integration tests (`tests/smart_*.rs`) use `tests/common/mod.rs`: `Sandbox` creates a temp dir deleted on drop (canonicalized for macOS `/var` symlinks), and `git()` runs git with global/system config disabled and `protocol.file.allow=always` so tests are hermetic and can use local-path remotes. Tests build real throwaway repos rather than mocking git.

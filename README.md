# git-smart-rust

A Rust port of [git-smart](https://github.com/kevwil/git-smart): git commands with some smarts. Each one explains its
decisions and echoes every git command that modifies the repository. No dependencies beyond `std`
and the `git` executable.

| Command | What it does |
| --- | --- |
| `git smart-pull` | Fetches the tracked branch, then fast-forwards, or rebases with `--rebase-merges` if both sides moved. Stashes and restores tracked local changes. |
| `git smart-merge <ref>` | `git merge --no-ff` wrapped in a stash/pop, with commit counts. |
| `git smart-log [args]` | A compact, coloured `git log --graph`; arguments pass through. |

## Install

    cargo install --path .

Git runs `git-smart-pull`, `git-smart-merge` and `git-smart-log` from `PATH` as `git smart-pull` etc.

Colour is used when stdout is a terminal (disable with `NO_COLOR`). Exit status is non-zero on failure.

## Development

```bash
make
make audit
make release
```

Integration tests build throwaway repositories in the system temp directory.

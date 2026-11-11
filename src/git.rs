use crate::error::{Error, Result, failed};
use crate::ui::Ui;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Command, Output, Stdio};

/// A git working tree, driven through the `git` executable.
pub struct Repo {
    dir: PathBuf,
}

impl Repo {
    /// Finds the repository containing `dir`.
    pub fn discover(dir: &Path) -> Result<Self> {
        let probe = Self {
            dir: dir.to_path_buf(),
        };
        match probe.query(&["rev-parse", "--show-toplevel"]) {
            Some(top) => Ok(Self {
                dir: PathBuf::from(top),
            }),
            None => Err(failed(format!(
                "You need to run this from within a Git directory.\nCurrent working directory: {}",
                dir.display()
            ))),
        }
    }

    pub fn current_branch(&self) -> String {
        self.query(&["symbolic-ref", "--short", "-q", "HEAD"])
            .unwrap_or_else(|| "HEAD".into())
    }

    /// Resolves `rev` to a full commit SHA, if git recognises it.
    pub fn sha(&self, rev: &str) -> Option<String> {
        self.query(&[
            "rev-parse",
            "--verify",
            "--quiet",
            &format!("{rev}^{{commit}}"),
        ])
    }

    pub fn tracking_remote(&self) -> Option<String> {
        self.config(&format!("branch.{}.remote", self.current_branch()))
    }

    pub fn tracking_branch(&self) -> Result<Option<String>> {
        let key = format!("branch.{}.merge", self.current_branch());
        self.config(&key)
            .map(|v| parse_tracking_ref(&key, &v))
            .transpose()
    }

    pub fn merge_base(&self, a: &str, b: &str) -> Result<String> {
        self.query(&["merge-base", a, b])
            .ok_or_else(|| failed(format!("'{a}' and '{b}' have no common ancestor!")))
    }

    /// Number of commits reachable from `to` but not from `from`.
    pub fn count_commits(&self, from: &str, to: &str) -> Result<usize> {
        self.query(&["rev-list", "--count", &format!("{from}..{to}")])
            .and_then(|n| n.parse().ok())
            .ok_or_else(|| failed(format!("Unable to count commits in {from}..{to}")))
    }

    /// True if tracked files have changes (untracked files are ignored).
    pub fn is_dirty(&self) -> bool {
        self.query(&["status", "--porcelain"])
            .is_some_and(|s| has_tracked_changes(&s))
    }

    /// Runs a git command, showing the user what is being executed and what it said.
    pub fn run<W: Write>(&self, ui: &mut Ui<W>, args: &[&str]) -> Result<()> {
        ui.say(&format!("Executing: git {}", args.join(" ")))?;
        let out = self.output(args)?;
        let text = indent(&combined_output(&out));
        if !out.status.success() {
            return Err(Error::Git(text));
        }
        if !text.is_empty() {
            ui.say(&text)?;
        }
        Ok(())
    }

    /// Like `run`, but lets git write straight to the terminal (colours, pager).
    pub fn run_inherited<W: Write>(&self, ui: &mut Ui<W>, args: &[&str]) -> Result<()> {
        ui.say(&format!("Executing: git {}", args.join(" ")))?;
        ui.flush()?;
        Command::new("git")
            .args(args)
            .current_dir(&self.dir)
            .status()?;
        Ok(())
    }

    fn config(&self, key: &str) -> Option<String> {
        self.query(&["config", key])
    }

    /// Stdout of a successful git command, trimmed; `None` on failure or empty output.
    fn query(&self, args: &[&str]) -> Option<String> {
        let out = Command::new("git")
            .args(args)
            .current_dir(&self.dir)
            .stderr(Stdio::null())
            .output()
            .ok()?;
        let text = String::from_utf8_lossy(&out.stdout).trim().to_owned();
        (out.status.success() && !text.is_empty()).then_some(text)
    }

    fn output(&self, args: &[&str]) -> std::io::Result<Output> {
        Command::new("git")
            .args(args)
            .current_dir(&self.dir)
            .output()
    }
}

/// First seven characters of a SHA, as git displays them.
pub fn short(sha: &str) -> &str {
    sha.get(..7).unwrap_or(sha)
}

fn parse_tracking_ref(key: &str, value: &str) -> Result<String> {
    value
        .strip_prefix("refs/heads/")
        .map(str::to_owned)
        .ok_or_else(|| {
            Error::Failed(format!(
                "Expected the config of '{key}' to be refs/heads/branchname, got '{value}'"
            ))
        })
}

/// Parses `git status --porcelain` output: anything not untracked (`??`) counts.
fn has_tracked_changes(porcelain: &str) -> bool {
    porcelain
        .lines()
        .any(|l| !l.is_empty() && !l.starts_with("??"))
}

fn combined_output(out: &Output) -> String {
    let mut s = String::from_utf8_lossy(&out.stdout).into_owned();
    s.push_str(&String::from_utf8_lossy(&out.stderr));
    s
}

fn indent(text: &str) -> String {
    text.lines()
        .map(|l| format!("  {l}"))
        .collect::<Vec<_>>()
        .join("\n")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn untracked_files_are_not_dirty() {
        assert!(!has_tracked_changes(""));
        assert!(!has_tracked_changes("?? noob\n?? other"));
    }

    #[test]
    fn tracked_changes_are_dirty() {
        assert!(has_tracked_changes(" M lib/codes.rb"));
        assert!(has_tracked_changes("A  noob\n?? other"));
        assert!(has_tracked_changes("UU README"));
        assert!(has_tracked_changes("D  gone"));
    }

    #[test]
    fn tracking_ref_strips_heads_prefix() {
        assert_eq!(
            parse_tracking_ref("k", "refs/heads/feature/x").unwrap(),
            "feature/x"
        );
    }

    #[test]
    fn tracking_ref_rejects_other_shapes() {
        let err = parse_tracking_ref("branch.a.merge", "refs/tags/v1").unwrap_err();
        assert!(err.to_string().contains("branch.a.merge"));
    }

    #[test]
    fn short_truncates_to_seven() {
        assert_eq!(short("0123456789abcdef"), "0123456");
        assert_eq!(short("abc"), "abc");
    }

    #[test]
    fn indent_prefixes_each_line() {
        assert_eq!(indent("a\nb"), "  a\n  b");
        assert_eq!(indent(""), "");
    }
}

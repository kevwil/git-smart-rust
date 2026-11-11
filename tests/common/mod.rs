#![allow(dead_code)]
use git_smart::{error::Result, ui::Ui};
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::atomic::{AtomicUsize, Ordering};

static NEXT: AtomicUsize = AtomicUsize::new(0);

/// A scratch directory that is deleted on drop.
pub struct Sandbox(pub PathBuf);

impl Sandbox {
    pub fn new() -> Self {
        let n = NEXT.fetch_add(1, Ordering::Relaxed);
        let dir = std::env::temp_dir().join(format!("git-smart-test-{}-{n}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        // Resolve symlinks (/var -> /private/var on macOS) so paths compare equal.
        Self(dir.canonicalize().unwrap())
    }

    pub fn path(&self, name: &str) -> PathBuf {
        self.0.join(name)
    }
}

impl Drop for Sandbox {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

pub fn git(dir: &Path, args: &[&str]) -> String {
    let out = Command::new("git")
        .args(["-c", "protocol.file.allow=always"])
        .args(args)
        .current_dir(dir)
        .env("GIT_CONFIG_GLOBAL", "/dev/null")
        .env("GIT_CONFIG_SYSTEM", "/dev/null")
        .output()
        .unwrap();
    assert!(
        out.status.success(),
        "git {args:?} failed: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    String::from_utf8_lossy(&out.stdout).into_owned()
}

/// Creates a repo on branch `main` with one commit containing README and lib/codes.rb.
pub fn init_repo(dir: &Path) {
    std::fs::create_dir_all(dir).unwrap();
    git(dir, &["init", "-q", "-b", "main"]);
    git(dir, &["config", "user.name", "Maxwell Smart"]);
    git(dir, &["config", "user.email", "agent86@control.gov"]);
    git(dir, &["config", "core.pager", "cat"]);
    write(dir, "README", "hurr durr\n");
    write(dir, "lib/codes.rb", "puts \"pro hax\"\n");
    commit_all(dir, "first");
}

pub fn clone_repo(from: &Path, to: &Path) {
    let parent = to.parent().unwrap();
    git(
        parent,
        &[
            "clone",
            "-q",
            from.to_str().unwrap(),
            to.file_name().unwrap().to_str().unwrap(),
        ],
    );
    git(to, &["config", "user.name", "Agent 99"]);
    git(to, &["config", "user.email", "agent99@control.gov"]);
    git(to, &["config", "core.pager", "cat"]);
}

pub fn write(dir: &Path, file: &str, contents: &str) {
    let path = dir.join(file);
    std::fs::create_dir_all(path.parent().unwrap()).unwrap();
    std::fs::write(path, contents).unwrap();
}

pub fn append(dir: &Path, file: &str, contents: &str) {
    let existing = std::fs::read_to_string(dir.join(file)).unwrap_or_default();
    write(dir, file, &format!("{existing}{contents}"));
}

pub fn commit_all(dir: &Path, msg: &str) {
    git(dir, &["add", "."]);
    git(dir, &["commit", "-q", "-m", msg]);
}

pub fn status(dir: &Path) -> String {
    git(dir, &["status", "--porcelain"])
}

pub fn last_commits(dir: &Path, n: usize) -> Vec<String> {
    git(dir, &["log", "--format=%s", "-n", &n.to_string()])
        .lines()
        .map(String::from)
        .collect()
}

/// Output lines of a command run, plus its result.
pub struct Run {
    pub lines: Vec<String>,
    pub result: Result<()>,
}

pub fn run(dir: &Path, command: &str, args: &[&str]) -> Run {
    let args: Vec<String> = args.iter().map(|s| s.to_string()).collect();
    let mut ui = Ui::new(Vec::new(), false);
    let result = git_smart::run(command, &args, dir, &mut ui);
    let text = String::from_utf8(ui.into_inner()).unwrap();
    Run {
        lines: text.lines().map(String::from).collect(),
        result,
    }
}

impl Run {
    pub fn reports(&self, needle: &str) -> bool {
        self.lines.iter().any(|l| l.contains(needle))
    }

    #[track_caller]
    pub fn assert_reports(&self, needle: &str) {
        assert!(
            self.reports(needle),
            "expected {needle:?} in:\n{}",
            self.lines.join("\n")
        );
    }

    #[track_caller]
    pub fn assert_not_reports(&self, needle: &str) {
        assert!(
            !self.reports(needle),
            "did not expect {needle:?} in:\n{}",
            self.lines.join("\n")
        );
    }

    #[track_caller]
    pub fn ok(&self) -> &Self {
        assert!(
            self.result.is_ok(),
            "command failed: {:?}\n{}",
            self.result,
            self.lines.join("\n")
        );
        self
    }
}

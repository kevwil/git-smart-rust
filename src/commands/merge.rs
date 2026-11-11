use super::with_stash;
use crate::error::{Result, failed};
use crate::git::{Repo, short};
use crate::ui::{Ui, count};
use std::io::Write;

/// A `git merge --no-ff` wrapped in a stash, with helpful commentary.
pub fn run<W: Write>(repo: &Repo, args: &[String], ui: &mut Ui<W>) -> Result<()> {
    let branch = repo.current_branch();
    ui.start(&format!("Starting: smart-merge on branch '{branch}'"))?;

    let target = args
        .first()
        .ok_or_else(|| failed("Usage: git smart-merge ref"))?;
    let target_sha = repo
        .sha(target)
        .ok_or_else(|| failed(format!("Branch to merge '{target}' not recognised by git!")))?;
    let head = repo
        .sha("HEAD")
        .ok_or_else(|| failed("There are no commits on the current branch to merge into!"))?;

    if target_sha == head {
        ui.note(&format!(
            "Branch '{target}' has no new commits. Nothing to merge in."
        ))?;
        return ui.success("Already up-to-date.").map_err(Into::into);
    }

    let base = repo.merge_base(&head, &target_sha)?;
    let incoming = repo.count_commits(&base, target)?;
    ui.say(&format!(
        "Branch '{target}' has diverged by {}. Merging in.",
        count(incoming, "commit")
    ))?;

    if head == base {
        // A fast-forward would make the branch vanish from history, so --no-ff it is.
        ui.note(&format!(
            "Branch '{branch}' has not moved on since '{target}' diverged. \
             Running with --no-ff anyway, since a fast-forward is unexpected behaviour."
        ))?;
    } else {
        let local = repo.count_commits(&base, &head)?;
        ui.say(&format!(
            "Branch '{branch}' has {} since '{target}' diverged.",
            count(local, "new commit")
        ))?;
    }

    with_stash(repo, ui, |ui| repo.run(ui, &["merge", "--no-ff", target]))?;

    let new_head = repo.sha("HEAD").unwrap_or_default();
    ui.success(&format!(
        "All good. Created merge commit {}.",
        short(&new_head)
    ))?;
    Ok(())
}

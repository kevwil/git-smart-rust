use super::{log, with_stash};
use crate::error::{Result, failed};
use crate::git::{Repo, short};
use crate::ui::{Ui, count};
use std::io::Write;

/// Fetches the tracked branch and brings the local branch up to date, by
/// fast-forward if possible and `rebase --rebase-merges` otherwise.
pub fn run<W: Write>(repo: &Repo, args: &[String], ui: &mut Ui<W>) -> Result<()> {
    let branch = repo.current_branch();
    ui.start(&format!("Starting: smart-pull on branch '{branch}'"))?;
    if !args.is_empty() {
        ui.warn(&format!("Ignoring arguments: {args:?}"))?;
    }

    let remote = match repo.tracking_remote() {
        Some(r) => r,
        None => {
            ui.note("No tracking remote configured, assuming 'origin'")?;
            "origin".to_owned()
        }
    };
    repo.run(ui, &["fetch", &remote])?;

    let tracking = match repo.tracking_branch()? {
        Some(b) => b,
        None => {
            ui.note(&format!(
                "No tracking branch configured, assuming '{branch}'"
            ))?;
            branch.clone()
        }
    };
    let upstream = format!("{remote}/{tracking}");
    let remote_sha = repo
        .sha(&upstream)
        .ok_or_else(|| failed(format!("Upstream branch '{upstream}' doesn't exist!")))?;
    let head = repo
        .sha("HEAD")
        .ok_or_else(|| failed("There are no commits on the current branch yet!"))?;

    if head == remote_sha {
        ui.say(&format!(
            "Neither your local branch '{branch}', nor the remote branch '{upstream}' have moved on."
        ))?;
        return ui.success("Already up-to-date").map_err(Into::into);
    }

    let base = repo.merge_base(&head, &remote_sha)?;

    let local = repo.count_commits(&base, &head)?;
    if local > 0 {
        ui.note(&format!(
            "You have {} on '{branch}'.",
            count(local, "new commit")
        ))?;
    }

    if base == remote_sha {
        ui.say(&format!("Remote branch '{upstream}' has not moved on."))?;
        return ui.success("Already up-to-date").map_err(Into::into);
    }

    let incoming = repo.count_commits(&base, &remote_sha)?;
    let verb = if incoming == 1 { "is" } else { "are" };
    ui.note(&format!(
        "There {verb} {} on '{upstream}'.",
        count(incoming, "new commit")
    ))?;

    let summary = with_stash(repo, ui, |ui| {
        if base == head {
            ui.say(&format!(
                "Local branch '{branch}' has not moved on. Fast-forwarding..."
            ))?;
            repo.run(ui, &["merge", "--ff-only", &upstream])?;
            Ok(format!(
                "Fast forwarded from {} to {}",
                short(&head),
                short(&remote_sha)
            ))
        } else {
            ui.note(&format!(
                "Both local and remote branches have moved on. Branch '{branch}' needs to be rebased onto '{upstream}'"
            ))?;
            repo.run(ui, &["rebase", "--rebase-merges", &upstream])?;
            let new_head = repo.sha("HEAD").unwrap_or_default();
            Ok(format!(
                "HEAD moved from {} to {}.",
                short(&head),
                short(&new_head)
            ))
        }
    })?;

    log::run(repo, &[format!("{base}..{upstream}")], ui)?;
    ui.success(&format!("All good. {summary}"))?;
    Ok(())
}
